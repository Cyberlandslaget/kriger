pub mod config;

use anyhow::{Context, Result};
use futures::StreamExt;
use tracing::{info, warn};
use kriger_common::messaging::{Message, Messaging};
use kriger_common::runtime::AppRuntime;
use crate::config::Config;

pub async fn main(runtime: AppRuntime, config: Config) -> Result<()> {
    info!("starting runner");

    let exploit = config.runner_exploit.context("runner: the runner-exploit option was not set")?;

    let worker_count = config.runner_workers.unwrap_or_else(|| num_cpus::get());
    info!("using a maximum of {worker_count} workers");

    info!("subscribing to execution requests for exploit: {exploit}");
    let mut stream = runtime.messaging.subscribe_execution_requests(&exploit).await
        .context("unable to subscribe to execution requests")?;

    // TODO: Implement
    while let Some(res) = stream.next().await {
        match res {
            Ok(message) => {
                message.progress().await?;
                info!("received {:?}", &message.payload());
                message.ack().await?;
                info!("acked");
            }
            Err(err) => warn!("unable to parse message: {err:?}"),
        }
    }

    Ok(())
}
