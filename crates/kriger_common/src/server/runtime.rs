// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use crate::messaging::nats::NatsMessaging;
use crate::models;
use futures::future::select_all;
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::signal::unix::SignalKind;
use tokio::sync::RwLock;
use tokio::{signal, spawn};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

/// Common state for components
#[derive(Clone)]
pub struct AppRuntime {
    pub config: Arc<AppConfig>,
    pub messaging: Arc<NatsMessaging>,
    pub metrics_registry: Arc<RwLock<prometheus_client::registry::Registry>>,
    pub cancellation_token: CancellationToken,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct AppConfig {
    pub competition: CompetitionConfig,
    /// The submitter configuration. This will be dynamically checked by the submitter at runtime
    /// to avoid having to model it in this crate.
    pub submitter: toml::Value,
    /// The fetcher configuration.
    pub fetcher: toml::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct CompetitionConfig {
    /// The start time of the competition in UTC
    pub start: chrono::DateTime<chrono::Utc>,
    /// Tick/round length in seconds
    pub tick: u64,
    /// The start tick in ticks. This indicates the first ticking round between T+0 and T+tick.
    pub tick_start: i64,
    /// The validity of flags in rounds
    pub flag_validity: u32,
    /// The regular expression for the flag format
    pub flag_format: String,
    /// The team id of the NOP team
    pub nop_team: Option<String>,
    /// The team id of the self team
    pub self_team: Option<String>,
}

impl Into<models::AppConfig> for AppConfig {
    fn into(self) -> models::AppConfig {
        models::AppConfig {
            competition: self.competition.into(),
        }
    }
}

impl Into<models::CompetitionConfig> for CompetitionConfig {
    fn into(self) -> models::CompetitionConfig {
        models::CompetitionConfig {
            start: self.start,
            tick: self.tick,
            tick_start: self.tick_start,
            flag_validity: self.flag_validity,
            flag_format: self.flag_format,
            nop_team: self.nop_team,
            self_team: self.self_team,
        }
    }
}

pub fn create_shutdown_cancellation_token() -> CancellationToken {
    let cancellation_token = CancellationToken::new();
    let signal_cancellation_token = cancellation_token.clone();

    spawn(async move {
        // TODO: Support Windows?
        let mut signals: Vec<signal::unix::Signal> = [
            signal::unix::signal(SignalKind::terminate()),
            signal::unix::signal(SignalKind::interrupt()),
        ]
        .into_iter()
        .filter_map(|maybe_signal| match maybe_signal {
            Ok(signal) => Some(signal),
            Err(error) => {
                error! {
                    ?error,
                    "unable to listen for shutdown signal"
                }
                None
            }
        })
        .collect();

        let signal_futures = signals.iter_mut().map(|signal| signal.recv().boxed());
        select_all(signal_futures).await;

        info!("shutdown signal received");
        signal_cancellation_token.cancel();
    });
    cancellation_token
}
