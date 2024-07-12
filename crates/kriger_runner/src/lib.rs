pub mod config;

use crate::config::Config;
use async_channel::Receiver;
use color_eyre::eyre::{bail, Context, ContextCompat, Result};
use futures::StreamExt;
use kriger_common::messaging::model::ExecutionRequest;
use kriger_common::messaging::nats::NatsMessaging;
use kriger_common::messaging::{Message, Messaging};
use std::process::Stdio;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tracing::{debug, info, warn};

const ENV_EXPLOIT_NAME: &'static str = "EXPLOIT";
const ENV_IP_ADDRESS: &'static str = "IP";
const ENV_FLAG_HINT: &'static str = "HINT";

struct Job {
    request: Box<dyn Message<Payload = ExecutionRequest> + Send>,
    _permit: OwnedSemaphorePermit,
}

async fn worker(
    idx: usize,
    rx: Receiver<Job>,
    exploit_name: String,
    exploit_command: String,
    exploit_args: Vec<String>,
) -> Result<()> {
    loop {
        // The channel has most likely been closed
        let job = rx.recv().await.context("unable to receive job")?;

        // TODO: Check how NATS handle retries for progress and nak
        // We may not want to terminate the worker if ack or nak fails
        job.request.progress().await.context("unable to ack")?;
        match execute(
            job.request.payload(),
            &exploit_name,
            &exploit_command,
            &exploit_args,
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
    exploit_args: &Vec<String>,
) -> Result<()> {
    info!("processing request: {request:?}");

    let mut command = tokio::process::Command::new(exploit_command);
    command.env(ENV_EXPLOIT_NAME, exploit_name);
    command.env(ENV_IP_ADDRESS, &request.ip_address);
    if let Some(flag_hint) = &request.flag_hint {
        let value = serde_json::to_string(flag_hint).context("unable to serialize flag hint")?;
        command.env(ENV_FLAG_HINT, value);
    }
    command.stdin(Stdio::null());
    command.args(exploit_args);

    let mut child = command.spawn().context("unable to spawn child")?;
    child.wait().await.context("unable to wait for child")?;

    Ok(())
}

pub async fn main(config: Config) -> Result<()> {
    info!("initializing messaging");
    let messaging = NatsMessaging::new(&config.nats_url).await?;

    info!(
        "subscribing to execution requests for exploit: {}",
        &config.exploit
    );
    let mut stream = messaging
        .subscribe_execution_requests(&config.exploit)
        .await
        .context("unable to subscribe to execution requests")?;

    let worker_count = config.workers.unwrap_or_else(|| 2 * num_cpus::get());
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
            config.exploit.clone(),
            config.command.clone(),
            config.args.clone(),
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
