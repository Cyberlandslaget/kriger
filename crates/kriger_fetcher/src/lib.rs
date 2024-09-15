mod fetcher;
mod metrics;

use crate::fetcher::{FetchOptions, Fetcher, FetcherConfig};
use crate::metrics::FetcherMetrics;
use color_eyre::eyre::{Context, Result};
use dashmap::DashMap;
use kriger_common::messaging;
use kriger_common::messaging::services::data::DataService;
use kriger_common::messaging::Bucket;
use kriger_common::models::Service;
use kriger_common::server::runtime::AppRuntime;
use std::ops::DerefMut;
use tokio::time::{Instant, MissedTickBehavior};
use tokio::{select, time};
use tracing::{debug, error, info, instrument, warn};

pub async fn main(runtime: AppRuntime) -> Result<()> {
    info!("starting data fetcher");

    let metrics = FetcherMetrics::default();
    metrics.register(runtime.metrics_registry.write().await.deref_mut());

    let config: FetcherConfig = runtime
        .config
        .fetcher
        .clone()
        .try_into()
        .context("unable to parse the fetcher config")?;

    let services_bucket = runtime.messaging.services();

    let services = services_bucket
        .subscribe_all()
        .await
        .context("unable to subscribe to services")?;

    let data_svc = runtime.messaging.data();

    let fetcher = config.inner.into_fetcher();

    let tick_duration = time::Duration::from_secs(config.interval);
    let instant =
        kriger_common::utils::time::get_instant_from_datetime(runtime.config.competition.start)?;
    let mut interval = time::interval_at(instant, tick_duration);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let options = FetchOptions {
        require_hints: true,
    };
    loop {
        select! {
            _ = interval.tick() => {}
            _ = runtime.cancellation_token.cancelled() => {
                return Ok(())
            }
        }

        handle_fetcher_tick(&services, &data_svc, &metrics, fetcher.as_ref(), &options).await;
    }
}

#[instrument(level = "DEBUG", skip_all)]
async fn handle_fetcher_tick(
    services: &DashMap<String, Service>,
    data_svc: &DataService,
    metrics: &FetcherMetrics,
    fetcher: &dyn Fetcher,
    options: &FetchOptions,
) {
    debug!("fetcher tick");
    metrics.start.inc();
    let start = Instant::now();
    let fetcher_res = fetcher.fetch(&options, &services).await;
    let elapsed = start.elapsed();

    metrics
        .duration
        .observe((elapsed.as_micros() as f64) / 1_000_000.0);

    let data = match fetcher_res {
        Ok(data) => data,
        Err(error) => {
            warn! {
                ?error,
                "fetcher error"
            }
            metrics.error.inc();
            return;
        }
    };
    metrics.complete.inc();
    debug!("received fetcher data");

    if let Some(flag_hints) = data.flag_hints {
        for hint in flag_hints {
            let data = messaging::model::FlagHint {
                team_id: hint.team_id,
                service: hint.service,
                hint: hint.hint,
            };
            if let Err(error) = data_svc.publish_flag_hint(&data).await {
                error! {
                    ?error,
                    hint.team_id = data.team_id,
                    hint.service = data.service,
                    "unable to publish flag hint"
                }
            }
        }
    }
}
