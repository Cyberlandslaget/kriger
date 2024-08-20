mod cini;
pub mod dummy;
pub mod enowars;
pub mod faust;
pub mod statisk;

use async_trait::async_trait;
use dashmap::DashMap;
use kriger_common::runtime::AppRuntime;
use kriger_common::{messaging, models};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ServiceOld(pub HashMap<String, TicksOld>);

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct TicksOld(pub HashMap<i32, serde_json::Value>);

/// All services
// /// {service_name: {"10.0.0.1": ["a", "b"], "10.0.0.2"}}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ServiceMap(pub HashMap<String, Service>);

impl ServiceMap {
    /// renames services
    pub fn apply_name_mapping(self, mapping: &HashMap<String, String>) -> ServiceMap {
        ServiceMap(
            self.0
                .into_iter()
                .map(|(old_name, service)| {
                    (
                        mapping.get(&old_name).unwrap_or(&old_name).to_owned(),
                        service,
                    )
                })
                .collect(),
        )
    }
}

/// A service' teams
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Service {
    pub teams: HashMap<String, TeamService>,
}

/// A teams' instance of a service
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct TeamService {
    // in most cases there is just one flagid per tick (we always just read the
    // raw json value), but in the case of faust-like ctfs we may have multiple
    // flagids and we dont know which they belong to, so we have to put
    // multiple for the current tick
    pub ticks: HashMap<i32, Vec<serde_json::Value>>,
}

#[derive(thiserror::Error, Debug)]
pub enum FetcherError {
    #[error("network error")]
    NetworkError(#[from] std::io::Error),
    #[error("format error")]
    /// The format of the response was not as expected
    FormatError,
    #[error("serde")]
    SerdeJson(#[from] serde_json::Error),
    #[error("reqwest")]
    Reqwest(#[from] reqwest::Error),
    #[error("messaging error")]
    Messaging(#[from] messaging::MessagingError),
}

/// Implements fetching flagids and hosts
pub trait OldFetcher {
    /// services (with flagids)
    async fn services(&self) -> Result<ServiceMap, FetcherError>;
    /// "backup" raw get all ips
    async fn ips(&self) -> Result<Vec<String>, FetcherError>;
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum FetcherConfig {
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
}

impl FetcherConfig {
    pub(crate) fn into_fetcher(self) -> Box<dyn Fetcher> {
        match self {
            FetcherConfig::Dummy => Box::new(dummy::DummyFetcher),
            FetcherConfig::Cini { url } => Box::new(cini::CiniFetcher::new(url)),
            FetcherConfig::Faust { url, ip_format } => {
                Box::new(faust::FaustFetcher::new(url, ip_format))
            }
        }
    }
}

#[async_trait]
pub(crate) trait Fetcher: Send {
    async fn run(
        &self,
        runtime: &AppRuntime,
        services: &DashMap<String, models::Service>,
    ) -> Result<(), FetcherError>;
}
