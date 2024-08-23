// TODO: Remove once things are implemented
#![allow(dead_code)]

mod fetcher;

use crate::fetcher::FetcherConfig;
use color_eyre::eyre::{Context, Result};
use kriger_common::messaging::{Bucket, Messaging};
use kriger_common::models;
use kriger_common::server::runtime::AppRuntime;
use tokio::time::MissedTickBehavior;
use tokio::{select, time};
use tracing::{info, warn};

pub async fn main(runtime: AppRuntime) -> Result<()> {
    info!("starting data fetcher");

    let config: FetcherConfig = runtime
        .config
        .fetcher
        .clone()
        .try_into()
        .context("unable to parse the fetcher config")?;

    let services_bucket = runtime
        .messaging
        .services()
        .await
        .context("unable to retrieve the services bucket")?;

    let services = services_bucket
        .subscribe_all::<models::Service>()
        .await
        .context("unable to subscribe to services")?;

    let fetcher = config.into_fetcher();

    // TODO: Un-hardcode this
    let tick_duration = time::Duration::from_secs(20);
    let mut interval = time::interval_at(time::Instant::now(), tick_duration);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    interval.tick().await; // The first tick will immediately complete

    loop {
        select! {
            _ = interval.tick() => {}
            _ = runtime.cancellation_token.cancelled() => {
                return Ok(())
            }
        }

        if let Err(error) = fetcher.run(&runtime, &services).await {
            warn! {
                ?error,
                "fetcher error"
            }
        }
    }
}
