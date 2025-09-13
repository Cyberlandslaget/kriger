// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use super::{CompetitionData, FetchOptions, Fetcher, FetcherError, FlagHint};
use async_trait::async_trait;
use dashmap::DashMap;
use kriger_common::models;
use serde::{self, Deserialize};
use std::collections::HashMap;
use tracing::{debug, instrument, warn};

#[derive(Deserialize, Debug)]
pub struct AttackInfo {
    pub teams: Vec<TeamInfo>,
    pub flag_ids: HashMap<String, ServiceInfo>, // service name -> ServiceInfo
}

// service/team ip -> service flag info
#[derive(Deserialize, Debug)]
pub struct TeamInfo {
    pub id: i64,
    pub name: String,
    pub ip: String,
}

// service/team ip -> service flag info
#[derive(Deserialize, Debug)]
pub struct ServiceInfo(HashMap<String, HashMap<String, serde_json::Value>>);

// flag type -> vec[vec[str/user]]
// #[derive(Deserialize, Debug)]
// pub struct ServiceFlagInfo(HashMap<String, Vec<serde_json::Value>>);

#[derive(Debug)]
pub struct SeerFetcher {
    client: reqwest::Client,
    url: String,
}

impl SeerFetcher {
    pub fn new(url: String) -> Self {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(0) // should disable pooling which fixes errors against some hosts
            .build()
            .unwrap();

        Self { url, client }
    }

    /// This fetches the ENOWARS GameServer's team.json data
    async fn get_attack_info(&self) -> Result<AttackInfo, FetcherError> {
        let info: AttackInfo = self.client.get(&self.url).send().await?.json().await?;
        Ok(info)
    }
}

#[async_trait]
impl Fetcher for SeerFetcher {
    #[instrument(skip_all)]
    async fn fetch(
        &self,
        _options: &FetchOptions,
        _services: &DashMap<String, models::Service>,
    ) -> Result<CompetitionData, FetcherError> {
        let info = self.get_attack_info().await?;

        debug! {
            team_count = info.teams.len(),
            service_count = info.flag_ids.len(),
            "fetched attack info"
        }

        // build team ip to id hash map
        let team_ip_to_info: HashMap<String, &TeamInfo> = info
            .teams
            .iter()
            .map(|team| (team.ip.to_string(), team))
            .collect();

        let mut flag_hints = Vec::new();
        for (service, info) in info.flag_ids {
            for (service_ip, round_hints) in info.0 {
                let team_info = match team_ip_to_info.get(&service_ip) {
                    Some(info) => info,
                    None => {
                        warn!("Failed to map service {} to a team!", service_ip);
                        continue;
                    }
                };

                for (round, hint) in round_hints {
                    match round.parse::<i64>() {
                        Ok(round_id) => {
                            flag_hints.push(FlagHint {
                                round: Some(round_id),
                                team_id: team_info.id.to_string(),
                                service: service.clone(),
                                hint,
                            });
                        }
                        Err(err) => {
                            warn!("Error trying to parse round_id: {err:?}");
                        }
                    }
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
        "teams": [
            {
                "id": 1,
                "name": "NOP",
                "ip": "10.32.1.2"
            },
            {
                "id": 2,
                "name": "Team 2",
                "ip": "10.32.2.2"
            }
        ],
        "flag_ids": {
            "service_1": {
                "10.32.1.2": {
                    "15": ["username1", "username1.2"],
                    "16": ["username2", "username2.2"]
                },
                "10.32.2.2": {
                    "15": ["username3", "username3.2"],
                    "16": ["username4", "username4.2"]
                }
            }
        }
    }"#;

    #[test]
    fn should_deserialize_meta_response() {
        let maybe_meta = serde_json::from_str(TEAMS_JSON);
        assert!(maybe_meta.is_ok(), "{:?}", maybe_meta);

        let meta: AttackInfo = maybe_meta.unwrap();
        assert_eq!(meta.teams.len(), 2);
        assert_eq!(meta.flag_ids.len(), 1);

        let team = &meta.teams[0];
        assert_eq!(team.id, 1);
        assert_eq!(team.name, "NOP");
        assert_eq!(team.ip, "10.32.1.2");

        let service_keys = meta.flag_ids.keys().collect::<Vec<&String>>();
        let service = &meta.flag_ids[service_keys[0]];

        assert_eq!(service_keys[0], "service_1");

        let round_hints = &service.0[&team.ip];

        let round_7_results = round_hints.get("15").unwrap().as_array().unwrap();

        assert_eq!(round_7_results[0], "username1");
        assert_eq!(round_7_results[1], "username1.2");

        let round_8_results = round_hints.get("16").unwrap().as_array().unwrap();

        assert_eq!(round_8_results[0], "username2");
        assert_eq!(round_8_results[1], "username2.2");
    }
}
