mod utils;

use crate::utils::get_current_tick;
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use chrono::Utc;
use color_eyre::eyre;
use color_eyre::eyre::{Context, ContextCompat};
use dashmap::DashMap;
use futures::StreamExt;
use kriger_common::messaging::model::{ExecutionRequest, FlagHint, SchedulingTick};
use kriger_common::messaging::{AckPolicy, Bucket, DeliverPolicy, Message, Messaging};
use kriger_common::server::runtime::AppRuntime;
use kriger_common::{messaging, models};
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time::{interval_at, MissedTickBehavior};
use tokio::{pin, select};
use tracing::{debug, error, info, instrument, warn};

pub async fn main(runtime: AppRuntime) -> eyre::Result<()> {
    info!("starting scheduler");

    debug!("retrieving buckets");
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
    let data_hints_bucket = runtime
        .messaging
        .data_hints()
        .await
        .context("unable to retrieve the data_hints bucket")?;

    debug!("subscribing to streams");
    let exploits = exploits_bucket
        .subscribe_all::<models::Exploit>()
        .await
        .context("unable to subscribe to exploits")?;

    let services = services_bucket
        .subscribe_all::<models::Service>()
        .await
        .context("unable to subscribe to exploits")?;

    // TODO: Investigate what the performance impact of subscribing to teams like this is.
    // There may be dozens or hundreds of teams, `subscribe_all` will continuously stream updates
    // and propagate the updates to a thread-safe map.
    let teams = teams_bucket
        .subscribe_all::<models::Team>()
        .await
        .context("unable to subscribe to teams")?;

    let mut set = JoinSet::new();
    set.spawn(handle_scheduling(
        runtime.clone(),
        exploits.clone(),
        services,
        teams.clone(),
    ));
    set.spawn(handle_hint_scheduling(
        runtime,
        data_hints_bucket,
        exploits,
        teams,
    ));

    while let Some(res) = set.join_next().await {
        res??;
    }

    Ok(())
}

async fn handle_scheduling(
    runtime: AppRuntime,
    exploits: Arc<DashMap<String, models::Exploit>>,
    services: Arc<DashMap<String, models::Service>>,
    teams: Arc<DashMap<String, models::Team>>,
) -> eyre::Result<()> {
    let config = &runtime.config;
    info!(
        "start: {:?} (d = {}), tick duration: {} s",
        &config.competition.start, config.competition.tick_start, config.competition.tick
    );

    let time_since_start = Utc::now().signed_duration_since(&config.competition.start);
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
    let tick = Duration::from_secs(config.competition.tick);
    let instant = utils::get_instant_from_datetime(config.competition.start)?;

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
        let tick = get_current_tick(
            config.competition.start,
            Utc::now(),
            config.competition.tick,
        );

        handle_tick(tick, &runtime, &exploits, &services, &teams).await;
    }
}

#[instrument(
    skip_all,
    fields(
        tick,
        exploit_count = exploits.len(),
        service_count = services.len(),
        team_count = teams.len(),
    )
)]
async fn handle_tick(
    tick: i64,
    runtime: &AppRuntime,
    exploits: &DashMap<String, models::Exploit>,
    services: &DashMap<String, models::Service>,
    teams: &DashMap<String, models::Team>,
) {
    info!("ticking");
    let res = runtime
        .messaging
        .publish(
            "scheduling.start".to_string(),
            &SchedulingTick { tick },
            false,
        )
        .await;
    if let Err(error) = res {
        warn! {
            ?error,
            "unable to publish scheduling start message"
        }
    }

    // O(|Exploits| * |Services| * |Teams|) - I hope?
    for exploit in exploits.iter() {
        if !exploit.manifest.enabled {
            debug! {
                exploit.name = exploit.manifest.name,
                "the exploit is disabled, skipping"
            }
            continue;
        }
        debug! {
            exploit.name = exploit.manifest.name,
            "scheduling executions"
        }

        // Used as the key in our K/V store since the service name can be unpredictable
        let service_name_b64 = STANDARD_NO_PAD.encode(&exploit.manifest.service);
        match services.get(&service_name_b64) {
            Some(service) => {
                // If the service has hints / flag ids, then we don't schedule the executions now.
                // We will schedule the execution once the hint is made available.
                if service.has_hint {
                    debug! {
                        service.name,
                        "the service requires hint, skipping"
                    }
                    continue;
                }

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

                        debug! {
                            ?request,
                            "sending execution request"
                        }

                        // TODO: Parallelize this
                        let res = runtime
                            .messaging
                            .publish(
                                format!("executions.{}.request", &exploit.manifest.name),
                                &request,
                                false, // TODO: Do we need double ack?
                            )
                            .await;

                        if let Err(error) = res {
                            warn! {
                                ?error,
                                "unable to send execution request"
                            }
                        }
                    } else {
                        warn! {
                            team.name = team.key(),
                            service.name,
                            "the team does not have an ip address for the service",
                        }
                    }
                }
            }
            None => {
                warn! {
                    exploit.name = exploit.manifest.name,
                    service.name = exploit.manifest.service,
                    "unable to find the service referenced by the exploit"
                }
            }
        }
    }
}

async fn handle_hint_scheduling(
    runtime: AppRuntime,
    data_hints_bucket: impl Bucket,
    exploits: Arc<DashMap<String, models::Exploit>>,
    teams: Arc<DashMap<String, models::Team>>,
) -> eyre::Result<()> {
    let data_hints = data_hints_bucket
        .watch_all::<FlagHint>(
            Some("scheduler".to_string()),
            AckPolicy::Explicit,
            Duration::from_secs(60), // TODO: Adjust
            DeliverPolicy::New,
        )
        .await
        .context("unable to watch flag hints")?;
    pin!(data_hints);

    loop {
        select! {
            _ = runtime.cancellation_token.cancelled() => {
                return Ok(())
            }
            maybe_message = data_hints.next() => {
                match maybe_message {
                    Some(Ok(message)) => {
                        // TODO: Make this more efficient.
                        if let Err(error) = handle_hint_schedule(&runtime, &teams, &exploits, &message).await {
                            error! {
                                ?error,
                                "scheduling error"
                            }
                            if let Err(error) = message.nak().await {
                                error! {
                                    ?error,
                                    "unable to ack message"
                                }
                            }
                        } else {
                            if let Err(error) = message.ack().await {
                                error! {
                                    ?error,
                                    "unable to ack message"
                                }
                            }
                        }
                    }
                    Some(Err(error)) => {
                        error! {
                            ?error,
                            "unexpected messaging error"
                        }
                    }
                    None => {
                        return Ok(())
                    }
                }
            }
        }
    }
}

#[instrument(
    skip_all,
    fields(
        team.id = message.payload().team_id,
        service.name = message.payload().service)
    )
]
async fn handle_hint_schedule(
    runtime: &AppRuntime,
    teams: &DashMap<String, models::Team>,
    exploits: &DashMap<String, models::Exploit>,
    message: &impl messaging::Message<Payload = FlagHint>,
) -> eyre::Result<()> {
    message.progress().await?;

    let payload = message.payload();

    let team = teams.get(&payload.team_id).context("unknown team id")?;
    let ip_address = team
        .services
        .get(&payload.service)
        .or(team.ip_address.as_ref())
        .context("unknown target ip address")?;

    let request = ExecutionRequest {
        ip_address: ip_address.clone(),
        flag_hint: Some(payload.hint.clone()),
        team_id: Some(team.key().clone()),
    };

    debug! {
        ?request,
        "sending execution request"
    }

    for exploit in exploits.iter() {
        if !exploit.manifest.enabled || exploit.manifest.service != payload.service {
            continue;
        }

        runtime
            .messaging
            .publish(
                format!("executions.{}.request", &exploit.manifest.name),
                &request,
                false,
            )
            .await?;
    }

    Ok(())
}
