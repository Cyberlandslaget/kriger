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
    pub flag_regex: Option<String>,
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
pub struct SaarFetcher {
    client: reqwest::Client,
    url: String,
}

impl SaarFetcher {
    pub fn new(url: String) -> Self {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(0) // should disable pooling which fixes errors against some hosts
            .build()
            .unwrap();

        Self { url, client }
    }

    /// This fetches the ENOWARS GameServer's team.json data
    async fn get_flag_ids(&self) -> Result<AttackInfo, FetcherError> {
        let info: AttackInfo = self.client.get(&self.url).send().await?.json().await?;
        Ok(info)
    }
}

#[async_trait]
impl Fetcher for SaarFetcher {
    #[instrument(skip_all)]
    async fn fetch(
        &self,
        _options: &FetchOptions,
        _services: &DashMap<String, models::Service>,
    ) -> Result<CompetitionData, FetcherError> {
        let info = self.get_flag_ids().await?;

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
      "name": "NOP Team",
      "ip": "5.75.216.150"
    }
  ],
  "flag_ids": {
    "fireworx": {
      "5.75.216.150": {
        "5116": [
          "qa7LStxMbMb8QulV0"
        ],
        "5117": [
          "jmwT2JlmZa1jh4Veh"
        ],
        "5118": [
          "3KWTj1hcPGq356g"
        ],
        "5119": [
          "R9f02o6ofmnQzLZdoZ5"
        ],
        "5120": [
          "qiGAgaaOCOFhAOZ3LCBa"
        ]
      }
    },
    "stldoctor": {
      "5.75.216.150": {
        "5116": [
          "Model d02d879b02.. is kinda sus",
          "User 8a0826131d.. is kinda sus"
        ],
        "5117": [
          "Model 18ea0f617d.. is kinda sus",
          "User 6979e838b9.. is kinda sus"
        ],
        "5118": [
          "Model 0ca4a37bab.. is kinda sus",
          "User 58ba598ca5.. is kinda sus"
        ],
        "5119": [
          "Model 2d5201290b.. is kinda sus",
          "User 0a82c7ae01.. is kinda sus"
        ],
        "5120": [
          "Model 8d04fd81a7.. is kinda sus",
          "User c8e9adca6f.. is kinda sus"
        ]
      }
    }
  }
}
"#;

    #[test]
    fn should_deserialize_meta_response() {
        let maybe_meta = serde_json::from_str(TEAMS_JSON);
        assert!(maybe_meta.is_ok(), "{:?}", maybe_meta);

        let meta: AttackInfo = maybe_meta.unwrap();
        assert_eq!(meta.teams.len(), 1);
        assert_eq!(meta.flag_ids.len(), 2);

        let team = &meta.teams[0];
        assert_eq!(team.id, 1);
        assert_eq!(team.name, "NOP Team");
        assert_eq!(team.ip, "5.75.216.150");

        let service_key = meta.flag_ids.keys().next().expect("no service keys");
        assert_eq!(service_key, "fireworx");
        let service = &meta.flag_ids[service_key];
        assert_eq!(service.0.len(), 1);
        let round_hints = &service.0[&team.ip];
        assert_eq!(round_hints.len(), 5);
        assert_eq!(round_hints["5116"][0], "qa7LStxMbMb8QulV0");
    }
}
