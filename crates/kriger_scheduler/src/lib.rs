mod utils;

use chrono::NaiveDate;
use color_eyre::eyre::Result;
use kriger_common::runtime::AppRuntime;
use std::time::Duration;
use tokio::time::{interval_at, MissedTickBehavior};
use tracing::{debug, info};

pub async fn main(runtime: AppRuntime) -> Result<()> {
    info!("starting scheduler");

    let start = NaiveDate::from_ymd_opt(2020, 1, 1)
        .unwrap()
        .and_hms_opt(8, 0, 0)
        .unwrap()
        .and_utc();

    // TODO: Use data from the competition config
    // TODO: Perhaps add tick delay to ensure that we're not going "too fast" and to account for clock skews
    let tick = Duration::from_secs(1);
    let instant = utils::get_instant_from_datetime(start).unwrap();

    let mut interval = interval_at(instant, tick);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        interval.tick().await;
        debug!("ticking");
    }
}
