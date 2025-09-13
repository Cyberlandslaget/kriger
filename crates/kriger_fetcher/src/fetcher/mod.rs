// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

mod cini;
pub mod dummy;
pub mod enowars;
pub mod faust;
pub mod seer;

use async_trait::async_trait;
use dashmap::DashMap;
use kriger_common::{messaging, models};
use serde::Deserialize;

pub(crate) struct CompetitionData {
    pub flag_hints: Option<Vec<FlagHint>>,
}

pub(crate) struct FlagHint {
    pub round: Option<i64>,
    pub team_id: String,
    pub service: String,
    pub hint: serde_json::Value,
}

#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
pub enum FetcherError {
    #[error("network error")]
    NetworkError(#[from] std::io::Error),
    /// The format of the response was not as expected
    #[error("format error")]
    FormatError,
    #[error("serde")]
    SerdeJson(#[from] serde_json::Error),
    #[error("reqwest")]
    Reqwest(#[from] reqwest::Error),
    #[error("messaging error")]
    Messaging(#[from] messaging::MessagingError),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) struct FetcherConfig {
    /// The interval that the fetcher should fetch at, in seconds.
    pub(crate) interval: u64,

    #[serde(flatten)]
    pub(crate) inner: InnerFetcherConfig,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum InnerFetcherConfig {
    Dummy,
    Cini {
        /// The URL of the "flag ids" service endpoint.
        url: String,
    },
    Faust {
        /// The URL of the "flag ids" service endpoint. This is usually located at
        /// /competition/teams.json
        url: String,
        /// The IP address format to use. This will be used to associate a team network ID with the
        /// correct IP address.
        ip_format: String,
    },
    Enowars {
        /// The URL of the "flag ids" service endpoint. This is usually located at
        /// /scoreboard/attack.json
        url: String,
    },
    Seer {
        /// The URL of the "flag ids" service endpoint. This is usually located at
        /// /scoreboard/attack.json
        url: String,
    },
}

impl InnerFetcherConfig {
    pub(crate) fn into_fetcher(self) -> Box<dyn Fetcher> {
        match self {
            InnerFetcherConfig::Dummy => Box::new(dummy::DummyFetcher),
            InnerFetcherConfig::Cini { url } => Box::new(cini::CiniFetcher::new(url)),
            InnerFetcherConfig::Faust { url, ip_format } => {
                Box::new(faust::FaustFetcher::new(url, ip_format))
            }
            InnerFetcherConfig::Enowars { url } => Box::new(enowars::EnowarsFetcher::new(url)),
            InnerFetcherConfig::Seer { url } => Box::new(seer::SeerFetcher::new(url)),
        }
    }
}

/// Instructs what the fetcher needs to fetch.
///
/// Providing options instead of strictly requesting specific data allows fetchers
/// to efficiently return data in a manner that is the most efficient. For example, some
/// competitions may return flag hints (flag ids) and the list of teams in a single response,
/// while others may return them separately.
pub(crate) struct FetchOptions {
    pub require_hints: bool,
}

#[async_trait]
pub(crate) trait Fetcher: Send + Sync {
    async fn fetch(
        &self,
        options: &FetchOptions,
        services: &DashMap<String, models::Service>,
    ) -> Result<CompetitionData, FetcherError>;
}
