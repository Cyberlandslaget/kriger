// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use super::{CompetitionData, FetchOptions, Fetcher, FetcherError, FlagHint};
use async_trait::async_trait;
use dashmap::DashMap;
use kriger_common::models;
use serde::{self, Deserialize};
use std::collections::HashMap;
use tracing::{debug, instrument};

#[derive(Deserialize, Debug)]
pub struct AttackInfo {
    #[serde(alias = "availableTeams")]
    pub available_teams: Vec<String>,
    pub services: HashMap<String, ServiceInfo>, // service name -> ServiceInfo
}

// service/team ip -> service flag info
#[derive(Deserialize, Debug)]
pub struct ServiceInfo(HashMap<String, HashMap<String, serde_json::Value>>);

// flag type -> vec[vec[str/user]]
// #[derive(Deserialize, Debug)]
// pub struct ServiceFlagInfo(HashMap<String, Vec<serde_json::Value>>);

#[derive(Debug)]
pub struct EnowarsFetcher {
    client: reqwest::Client,
    url: String,
}

impl EnowarsFetcher {
    pub fn new(url: String) -> Self {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(0) // should disable pooling which fixes errors against some hosts
            .build()
            .unwrap();

        Self { url, client }
    }

    /// This fetches the ENOWARS GameServer's team.json data
    async fn get_attack_into(&self) -> Result<AttackInfo, FetcherError> {
        let info: AttackInfo = self.client.get(&self.url).send().await?.json().await?;
        Ok(info)
    }
}

#[async_trait]
impl Fetcher for EnowarsFetcher {
    #[instrument(skip_all)]
    async fn fetch(
        &self,
        _options: &FetchOptions,
        _services: &DashMap<String, models::Service>,
    ) -> Result<CompetitionData, FetcherError> {
        let info = self.get_attack_into().await?;

        debug! {
            team_count = info.available_teams.len(),
            service_count = info.services.len(),
            "fetched attack info"
        }

        let mut flag_hints = Vec::new();
        for (service, info) in info.services {
            for (team_ip, round_hints) in info.0 {
                for (round, hint) in round_hints {
                     flag_hints.push(FlagHint {
                    round: Some(round.parse::<i64>().unwrap()),
                    team_id: team_ip.clone(),
                    service: service.clone(),
                    hint,
                });
                }
            }
        }

        Ok(CompetitionData {
            flag_hints: Some(flag_hints),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEAMS_JSON: &str = r#"
    {
        "availableTeams": [
            "10.1.52.1"
        ],
        "services": {
            "service_1": {
                "10.1.52.1": {
                    "7": [
                        [ "user73" ],
                        [ "user5" ]
                    ],
                    "8": [
                        [ "user96" ],
                        [ "user314" ]
                    ]
                }
            }
        }
    }"#;

    #[test]
    fn should_deserialize_meta_response() {
        let maybe_meta = serde_json::from_str(TEAMS_JSON);
        assert!(maybe_meta.is_ok());

        let meta: AttackInfo = maybe_meta.unwrap();
        assert_eq!(meta.available_teams.len(), 1);
        assert_eq!(meta.services.len(), 1);

        let team = &meta.available_teams[0];
        let service_keys = meta.services.keys().collect::<Vec<&String>>();
        let service = &meta.services[service_keys[0]];

        assert_eq!(
             service_keys[0],
             "service_1"
        );
        assert_eq!(team, "10.1.52.1");
        
        let round_hints = &service.0[team];
        
        let round_7_results = round_hints.get("7").unwrap().as_array().unwrap();
        
        assert_eq!(round_7_results[0].as_array().unwrap()[0], "user73");
        assert_eq!(round_7_results[1].as_array().unwrap()[0], "user5");

        let round_8_results = round_hints.get("8").unwrap().as_array().unwrap();
        
        assert_eq!(round_8_results[0].as_array().unwrap()[0], "user96");
        assert_eq!(round_8_results[1].as_array().unwrap()[0], "user314");

    }
}
