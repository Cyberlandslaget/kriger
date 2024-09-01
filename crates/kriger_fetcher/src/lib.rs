// TODO: Remove once things are implemented
#![allow(dead_code)]

mod fetcher;

use crate::fetcher::{FetchOptions, FetcherConfig, FlagHint};
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use color_eyre::eyre::{Context, Result};
use kriger_common::messaging::{Bucket, Messaging, MessagingError};
use kriger_common::server::runtime::AppRuntime;
use kriger_common::{messaging, models};
use tokio::task::JoinSet;
use tokio::time::MissedTickBehavior;
use tokio::{select, time};
use tracing::{debug, error, info, instrument, warn};

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

    let hints_bucket = runtime
        .messaging
        .data_hints()
        .await
        .context("unable to retrieve the hints bucket")?;
    let hints_bucket = Box::leak(Box::new(hints_bucket)); // TODO: fix?

    let fetcher = config.into_fetcher();

    // TODO: Un-hardcode this and align the start to the start of a tick
    let tick_duration = time::Duration::from_secs(20);
    let mut interval = time::interval(tick_duration);
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

        debug!("fetcher tick");
        let data = match fetcher.fetch(&options, &services).await {
            Ok(data) => data,
            Err(error) => {
                warn! {
                    ?error,
                    "fetcher error"
                }
                continue;
            }
        };

        if let Some(flag_hints) = data.flag_hints {
            let mut set = JoinSet::new();
            for hint in flag_hints {
                set.spawn(handle_hint_insertion(hints_bucket, hint));
            }

            while let Some(res) = set.join_next().await {
                if let Err(error) = res {
                    error! {
                        ?error,
                        "unable to join task"
                    }
                }
            }
        }
    }
}

#[instrument(level = "DEBUG", skip_all, fields(team_id, service, hint))]
async fn handle_hint_insertion(bucket: &impl Bucket, hint: FlagHint) {
    let serialized = match serde_json::to_vec(&hint.hint) {
        Ok(serialized) => serialized,

        Err(error) => {
            error! {
                ?error,
                "unable to serialize the hint"
            }
            return;
        }
    };

    let key = format!(
        "{}.{}.{}",
        STANDARD_NO_PAD.encode(&hint.service),
        hint.team_id,
        STANDARD_NO_PAD.encode(&serialized)
    );
    let data = messaging::model::FlagHint {
        team_id: hint.team_id,
        service: hint.service,
        hint: hint.hint,
    };
    match bucket.create(&key, &data).await {
        Err(MessagingError::KeyValueConflictError) => {
            // Ignore
        }
        Err(error) => {
            error! {
                ?error,
                "unable to insert the flag hint to the k/v store"
            }
        }
        _ => {}
    }
}
