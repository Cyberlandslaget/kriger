mod cini;
pub mod enowars;
pub mod faust;
pub mod statisk;

use crate::fetcher::cini::CiniFetcher;
use async_trait::async_trait;
use dashmap::DashMap;
use kriger_common::messaging::model;
use kriger_common::runtime::AppRuntime;
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
    #[error("reqwest failed")]
    Reqwest(#[from] reqwest::Error),
    #[error("unknown error")]
    Unknown,
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
}

impl FetcherConfig {
    pub(crate) fn into_fetcher(self) -> Box<dyn Fetcher> {
        match self {
            FetcherConfig::Dummy => unimplemented!(),
            FetcherConfig::Cini { url } => Box::new(CiniFetcher::new(url)),
        }
    }
}

#[async_trait]
pub(crate) trait Fetcher {
    async fn run(
        &self,
        runtime: &AppRuntime,
        services: &DashMap<String, model::Service>,
    ) -> Result<(), FetcherError>;
}
