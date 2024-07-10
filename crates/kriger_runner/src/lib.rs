pub mod config;

use std::sync::Arc;
use std::time::Duration;
use anyhow::{bail, Context, Result};
use async_channel::Receiver;
use futures::StreamExt;
use tokio::spawn;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::sleep;
use tracing::{debug, info, warn};
use kriger_common::messaging::{Message, Messaging};
use kriger_common::messaging::model::ExecutionRequest;
use kriger_common::runtime::AppRuntime;
use crate::config::Config;

const ENV_EXPLOIT_NAME: &'static str = "EXPLOIT_NAME";
const ENV_IP_ADDRESS: &'static str = "IP";
const ENV_FLAG_ID: &'static str = "FLAG_ID";

struct Job<'a> {
    request: Box<dyn Message<Payload=ExecutionRequest> + Send + 'a>,
    permit: OwnedSemaphorePermit,
}


async fn worker(idx: usize, rx: Receiver<Job<'_>>) -> Result<()> {
    loop {
        let job = rx.recv().await.context("unable to receive job")?;
        if let Err(err) = execute(job).await {
            warn!("execution failed: {err:?} (worker {idx})")
        }
    }
}

async fn execute(job: Job<'_>) -> Result<()> {
    job.request.progress().await.context("unable to ack")?;

    let request = job.request.payload();
    info!("processing request: {request:?}");
    sleep(Duration::from_secs(5)).await;

    job.request.ack().await.context("unable to ack")?;

    Ok(())
}


pub async fn main(runtime: AppRuntime, config: Config) -> Result<()> {
    info!("starting runner");

    let exploit = Box::new(config.runner_exploit.context("runner: the runner-exploit option was not set")?);

    let worker_count = config.runner_workers.unwrap_or_else(|| 2 * num_cpus::get());
    info!("using a maximum of {worker_count} workers");

    info!("subscribing to execution requests for exploit: {exploit}");
    let messaging = Box::new(runtime.messaging);
    // FIXME: Is there a way to avoid leak?
    let mut stream = Box::leak(messaging).subscribe_execution_requests(Box::leak(exploit)).await
        .context("unable to subscribe to execution requests")?;

    let semaphore = Arc::new(Semaphore::new(worker_count));
    let (tx, rx) = async_channel::bounded::<Job>(worker_count);

    for i in 0..worker_count {
        // TODO: Support cancellation tokens
        let r = rx.clone();
        spawn(worker(i, r));
    }

    loop {
        let permit = semaphore.clone().acquire_owned().await.context("permit acquisition failed")?;
        debug!("permit acquired: {permit:?}");
        match stream.next().await.context("end of stream")? {
            Ok(message) => {
                let job = Job {
                    request: Box::new(message),
                    permit,
                };
                if let Err(err) = tx.send(job).await {
                    bail!("channel closed: {err:?}");
                }
            }
            Err(err) => warn!("unable to parse message: {err:?}"),
        };
    }
}
