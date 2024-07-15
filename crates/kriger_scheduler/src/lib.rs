mod utils;

use color_eyre::eyre::{Context, ContextCompat, Result};
use kriger_common::messaging::model::{CompetitionConfig, Exploit};
use kriger_common::messaging::{Bucket, Messaging};
use kriger_common::runtime::AppRuntime;
use std::time::Duration;
use tokio::time::{interval_at, MissedTickBehavior};
use tracing::{debug, info};

pub async fn main(runtime: AppRuntime) -> Result<()> {
    info!("starting scheduler");

    let config_bucket = runtime
        .messaging
        .config()
        .await
        .context("unable to retrieve the config bucket")?;
    let exploits_bucket = runtime
        .messaging
        .exploits()
        .await
        .context("unable to retrieve the exploits bucket")?;

    // TODO: Provide a more elegant way to retrieve this and add support for live reload
    let config = config_bucket
        .get::<CompetitionConfig>("competition")
        .await
        .context("unable to retrieve the competition config")?
        .context("the competition config does not exist")?;

    let exploits = exploits_bucket
        .subscribe_all::<Exploit>()
        .await
        .context("unable to subscribe to exploits")?;

    debug!("exploits: {exploits:?}");

    info!(
        "start: {:?} (d = {}), tick duration: {} s",
        &config.start, config.tick_start, config.tick
    );

    let time_since_start = chrono::Utc::now().signed_duration_since(&config.start);
    if time_since_start > chrono::Duration::seconds(0) {
        info!(
            "the competition started {:} s ago",
            time_since_start.num_seconds()
        );
    } else {
        info!(
            "the competition starts in {:} s",
            time_since_start.num_seconds()
        );
    }

    // TODO: Perhaps add tick delay to ensure that we're not going "too fast" and to account for clock skews
    let tick = Duration::from_secs(config.tick);
    let instant = utils::get_instant_from_datetime(config.start).unwrap();

    let mut interval = interval_at(instant, tick);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    // The first tick completes immediately.
    interval.tick().await;

    loop {
        interval.tick().await;
        debug!("ticking");
        debug!("exploits: {exploits:?}");
    }
}
