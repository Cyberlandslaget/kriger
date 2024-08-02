mod utils;

use crate::utils::get_current_tick;
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use chrono::Utc;
use color_eyre::eyre::{Context, ContextCompat, Result};
use kriger_common::messaging::model::{
    CompetitionConfig, ExecutionRequest, Exploit, Service, Team,
};
use kriger_common::messaging::{Bucket, Messaging};
use kriger_common::runtime::AppRuntime;
use std::time::Duration;
use tokio::select;
use tokio::time::{interval_at, MissedTickBehavior};
use tracing::{debug, info, instrument, warn};

#[instrument(skip_all)]
pub async fn main(runtime: AppRuntime) -> Result<()> {
    info!("starting scheduler");

    debug!("retrieving buckets");
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
    let services_bucket = runtime
        .messaging
        .services()
        .await
        .context("unable to retrieve the services bucket")?;
    let teams_bucket = runtime
        .messaging
        .teams()
        .await
        .context("unable to retrieve the teams bucket")?;

    debug!("subscribing to streams");
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

    let services = services_bucket
        .subscribe_all::<Service>()
        .await
        .context("unable to subscribe to exploits")?;

    // TODp: Investigate what the performance impact of subscribing to teams like this is.
    // There may be dozens or hundreds of teams, `subscribe_all` will continuously stream updates
    // and propagate the updates to a thread-safe map.
    let teams = teams_bucket
        .subscribe_all::<Team>()
        .await
        .context("unable to subscribe to teams")?;

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

    // TODO: Add scheduling for services with hints
    loop {
        select! {
            _ = interval.tick() => {}
            _ = runtime.cancellation_token.cancelled() => {
                return Ok(());
            }
        }
        let tick = get_current_tick(config.start, Utc::now(), config.tick);
        info! {
            current_tick = tick,
            "ticking"
        }
        // TODO: Send tick message

        // O(|Exploits| * |Services| * |Teams|) - I hope?
        for exploit in exploits.iter() {
            if !exploit.manifest.enabled {
                continue;
            }

            // Used as the key in our K/V store since the service name can be unpredictable
            let service_name_b64 = STANDARD_NO_PAD.encode(&exploit.manifest.service);
            match services.get(&service_name_b64) {
                Some(service) => {
                    // If the service has hints / flag ids, then we don't schedule the executions now.
                    // We will schedule the execution once the hint is made available.
                    if service.has_hint {
                        debug!("service {} requires hint, skipping", &service.name);
                        continue;
                    }

                    debug!("teams: {}", teams.len());
                    for team in teams.iter() {
                        let ip_address = team
                            .services
                            .get(&service.name)
                            .or(team.ip_address.as_ref());

                        if let Some(ip_address) = ip_address {
                            let request = ExecutionRequest {
                                ip_address: ip_address.clone(),
                                flag_hint: None,
                                team_id: Some(team.key().clone()),
                            };

                            debug!("scheduling: {request:?}");

                            // TODO: Parallelize this
                            let res = runtime
                                .messaging
                                .publish(
                                    format!("executions.{}.request", &exploit.manifest.name),
                                    &request,
                                    false, // TODO: Do we need double ack?
                                )
                                .await;

                            if let Err(err) = res {
                                warn!("unable to request execution: {err:?}");
                            }
                        } else {
                            warn!(
                                "the team `{}` does not have an ip address for the service `{}`",
                                &team.key(),
                                &service.name
                            );
                        }
                    }
                }
                None => {
                    warn!(
                        "unable to find the service `{}` referenced by the exploit `{}`",
                        &exploit.manifest.service, &exploit.manifest.name
                    );
                }
            }
        }
        debug!("exploits: {exploits:?}");
    }
}
