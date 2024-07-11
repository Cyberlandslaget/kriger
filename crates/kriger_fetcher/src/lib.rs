mod fetcher;

use color_eyre::eyre::Result;
use kriger_common::runtime::AppRuntime;
use tokio::time;
use tokio::time::MissedTickBehavior;
use tracing::info;

pub async fn main(runtime: AppRuntime) -> Result<()> {
    info!("starting data fetcher");

    let tick_duration = time::Duration::from_secs(60);
    let mut interval = time::interval_at(time::Instant::now(), tick_duration);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    Ok(())
}
