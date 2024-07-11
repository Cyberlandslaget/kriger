pub mod config;

use crate::config::Config;
use anyhow::{bail, Context, Result};
use async_channel::Receiver;
use futures::StreamExt;
use kriger_common::messaging::model::ExecutionRequest;
use kriger_common::messaging::{Message, Messaging};
use kriger_common::runtime::AppRuntime;
use std::process::Stdio;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tracing::{debug, info, warn};

const ENV_EXPLOIT_NAME: &'static str = "EXPLOIT_NAME";
const ENV_IP_ADDRESS: &'static str = "IP";
const ENV_FLAG_ID: &'static str = "FLAG_ID";

struct Job {
    request: Box<dyn Message<Payload = ExecutionRequest> + Send>,
    _permit: OwnedSemaphorePermit,
}

async fn worker(
    idx: usize,
    rx: Receiver<Job>,
    exploit_name: &str,
    exploit_command: String,
    exploit_args: Option<String>,
) -> Result<()> {
    loop {
        // The channel has most likely been closed
        let job = rx.recv().await.context("unable to receive job")?;

        // TODO: Check how NATS handle retries for progress and nak
        // We may not want to terminate the worker if ack or nak fails
        job.request.progress().await.context("unable to ack")?;
        match execute(
            job.request.payload(),
            exploit_name,
            &exploit_command,
            exploit_args.as_deref(),
        )
        .await
        {
            Err(err) => {
                job.request.nak().await.context("unable to nak")?;
                warn!("execution failed: {err:?} (worker {idx})")
            }
            Ok(..) => {
                job.request.ack().await.context("unable to ack")?;
                info!("execution succeeded (worker {idx})")
            }
        }
    }
}

async fn execute(
    request: &ExecutionRequest,
    exploit_name: &str,
    exploit_command: &str,
    exploit_args: Option<&str>,
) -> Result<()> {
    info!("processing request: {request:?}");

    let mut command = tokio::process::Command::new(exploit_command);
    command.env(ENV_EXPLOIT_NAME, exploit_name);
    command.env(ENV_IP_ADDRESS, &request.ip_address);
    if let Some(flag_id) = &request.flag_id {
        command.env(ENV_FLAG_ID, flag_id);
    }
    command.stdin(Stdio::null());

    if let Some(args) = exploit_args {
        command.args(args.split(' '));
    }

    let mut child = command.spawn().context("unable to spawn child")?;
    child.wait().await.context("unable to wait for child")?;

    Ok(())
}

pub async fn main(runtime: AppRuntime, config: Config) -> Result<()> {
    info!("starting runner");

    let exploit_name = Box::leak(Box::new(
        config
            .runner_exploit
            .context("runner: the runner-exploit option is undefined")?,
    ));
    let exploit_command = config
        .runner_exploit_command
        .context("runner: the runner-exploit-command option is undefined")?;

    info!("subscribing to execution requests for exploit: {exploit_name}");
    let mut stream = runtime
        .messaging
        .subscribe_execution_requests(exploit_name)
        .await
        .context("unable to subscribe to execution requests")?;

    let worker_count = config.runner_workers.unwrap_or_else(|| 2 * num_cpus::get());
    let semaphore = Arc::new(Semaphore::new(worker_count));
    info!("using a maximum of {worker_count} workers");

    let (tx, rx) = async_channel::bounded::<Job>(worker_count);

    for i in 0..worker_count {
        // TODO: Support cancellation tokens
        let r = rx.clone();

        // Cloning here has a negligible impact to the memory usage
        spawn(worker(
            i,
            r,
            exploit_name,
            exploit_command.clone(),
            config.runner_exploit_args.clone(),
        ));
    }

    loop {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .context("permit acquisition failed")?;
        debug!("permit acquired: {permit:?}");
        match stream.next().await.context("end of stream")? {
            Ok(message) => {
                let job = Job {
                    request: Box::new(message),
                    _permit: permit,
                };
                if let Err(err) = tx.send(job).await {
                    bail!("channel closed: {err:?}");
                }
            }
            Err(err) => warn!("unable to parse message: {err:?}"),
        };
    }
}
