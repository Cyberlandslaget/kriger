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
    pub attack_info: HashMap<String, ServiceInfo>, // service name -> ServiceInfo
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
            service_count = info.attack_info.len(),
            "fetched attack info"
        }

        // build team ip to id hash map
        let team_ip_to_info: HashMap<String, &TeamInfo> = info
            .teams
            .iter()
            .map(|team| (team.ip.to_string(), team))
            .collect();

        let mut flag_hints = Vec::new();
        for (service, info) in info.attack_info {
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
  "flag_regex": "ECSC{[A-Za-z0-9-_]{32}}",
  "teams": [
    {
      "id": 2,
      "name": "Alverad Technology Focus Ltd.",
      "ip": "10.41.2.2",
      "online": true
    },
    {
      "id": 3,
      "name": "CybrOps",
      "ip": "10.41.3.2",
      "online": true
    },
    {
      "id": 43,
      "name": "Team Europe",
      "ip": "10.41.43.2",
      "online": true
    }
  ],
  "attack_info": {
    "bbspriest": {
      "10.41.2.2": {
        "17": {
          "0": null,
          "1": null
        },
        "18": {
          "0": null,
          "1": null
        },
        "19": {
          "0": null,
          "1": null
        },
        "20": {
          "0": null,
          "1": "15f73c752d6af307edef763b6958a3091368bc2abd4bf5f56dc4375f2399d8d9"
        },
        "21": {
          "0": "668a0be3e74b0b16d2cd95f59d22a6e08568705ae7112b4473acd0bec9d0d45b",
          "1": "fb4f9cb8abf1f42e0e0e90f9ac93347a366caed01d1e7e2e44e162819dbbe04a"
        },
        "22": {
          "0": null,
          "1": null
        }
      },
      "10.41.3.2": {
        "17": {
          "0": null,
          "1": null
        },
        "18": {
          "0": null,
          "1": null
        },
        "19": {
          "0": null,
          "1": null
        },
        "20": {
          "0": null,
          "1": null
        },
        "21": {
          "0": null,
          "1": null
        },
        "22": {
          "0": null,
          "1": null
        }
      },
      "10.41.4.2": {
        "17": {
          "0": "d536e08d2bb51ce8f6ad79dd3d4d64a732368ef3b769ea615ed771824d90c024",
          "1": "740fd99813e1cfcbeb716527cc841bff9f22a4300bb0e6bdb4558e2a3b0c5cbe"
        },
        "18": {
          "0": "eda56d81e2535db7ce29c050bbdc87be73ebe0f052056553afca7b3f015a5c23",
          "1": "e5bdd6938906c09bab1affcdf486568014a2e56d73950b48303fe77acbcf6330"
        },
        "19": {
          "0": "e7b012a660d6403aa3757dcd1c2796f8df863f8c31ef35f0975053499eec3065",
          "1": "a34c72c3447ad341cf5f3ffb8c91cbe8bce774519377334477d3c0674cc4b408"
        },
        "20": {
          "0": "f7e811f7dee7405fe14b12937542f6b7091aab0069f471e509c2379cfdf79287",
          "1": "b62d5738b9f108d05ec9fda0e44822a26fc729541ab06669558444b784481c7b"
        },
        "21": {
          "0": "2b9fa16a4b5e4b969f058ce39eea60f88f0d3252f78f8bee1a4e51821f9e4a37",
          "1": "6ad377c37cf11f03fb9f46572f5d0255632af124532f21d9cdd254536b49c894"
        },
        "22": {
          "0": null,
          "1": null
        }
      }
    },
    "ChatNG": {
      "10.41.2.2": {
        "17": {
          "0": "username:hu88Ko5H",
          "1": "botname:cocaptenB9mZEu0aBAr"
        },
        "18": {
          "0": "username:WbZErBeh",
          "1": "botname:ripbardHlfNMpwe8luL"
        },
        "19": {
          "0": "username:DdonUpXfB4",
          "1": "botname:ripbardKTR2d3sGZ"
        },
        "20": {
          "0": "username:dWQ4VFqbdQM",
          "1": "botname:cocaptenpEf7tTLXbi9"
        },
        "21": {
          "0": "username:O3TChyBj",
          "1": "botname:cocaptenRIIGwVr4"
        },
        "22": {
          "0": null,
          "1": null
        }
      },
      "10.41.3.2": {
        "17": {
          "0": null,
          "1": null
        },
        "18": {
          "0": null,
          "1": null
        },
        "19": {
          "0": null,
          "1": null
        },
        "20": {
          "0": null,
          "1": null
        },
        "21": {
          "0": null,
          "1": null
        },
        "22": {
          "0": null,
          "1": null
        }
      }
    },
    "FlagTransport": {
      "10.41.2.2": {
        "17": {
          "0": null,
          "1": null
        },
        "18": {
          "0": null,
          "1": null
        },
        "19": {
          "0": null,
          "1": null
        },
        "20": {
          "0": null,
          "1": null
        },
        "21": {
          "0": null,
          "1": null
        },
        "22": {
          "0": null,
          "1": null
        }
      },
      "10.41.3.2": {
        "17": {
          "0": null,
          "1": null
        },
        "18": {
          "0": null,
          "1": null
        },
        "19": {
          "0": null,
          "1": null
        },
        "20": {
          "0": null,
          "1": null
        },
        "21": {
          "0": null,
          "1": null
        },
        "22": {
          "0": null,
          "1": null
        }
      }
    },
    "Onbordo": {
      "10.41.2.2": {
        "17": {
          "0": null,
          "1": null
        },
        "18": {
          "0": null,
          "1": null
        },
        "19": {
          "0": null,
          "1": null
        },
        "20": {
          "0": null,
          "1": null
        },
        "21": {
          "0": null,
          "1": null
        },
        "22": {
          "0": null,
          "1": null
        }
      },
      "10.41.3.2": {
        "17": {
          "0": null,
          "1": null
        },
        "18": {
          "0": null,
          "1": null
        },
        "19": {
          "0": null,
          "1": null
        },
        "20": {
          "0": null,
          "1": null
        },
        "21": {
          "0": null,
          "1": null
        },
        "22": {
          "0": null,
          "1": null
        }
      }
    },
    "NSA": {
      "10.41.2.2": {
        "17": {
          "0": "13",
          "1": "12"
        },
        "18": {
          "0": "16",
          "1": "17"
        },
        "19": {
          "0": "19",
          "1": "20"
        },
        "20": {
          "0": "25",
          "1": "24"
        },
        "21": {
          "0": "28",
          "1": "29"
        },
        "22": {
          "0": null,
          "1": null
        }
      },
      "10.41.3.2": {
        "17": {
          "0": null,
          "1": null
        },
        "18": {
          "0": null,
          "1": null
        },
        "19": {
          "0": null,
          "1": null
        },
        "20": {
          "0": null,
          "1": null
        },
        "21": {
          "0": null,
          "1": null
        },
        "22": {
          "0": null,
          "1": null
        }
      }
    }
  }
}"#;

    #[test]
    fn should_deserialize_meta_response() {
        let maybe_meta = serde_json::from_str(TEAMS_JSON);
        assert!(maybe_meta.is_ok(), "{:?}", maybe_meta);

        let meta: AttackInfo = maybe_meta.unwrap();
        assert_eq!(meta.teams.len(), 3);
        assert_eq!(meta.attack_info.len(), 5);

        let team = &meta.teams[0];
        assert_eq!(team.id, 2);
        assert_eq!(team.name, "Alverad Technology Focus Ltd.");
        assert_eq!(team.ip, "10.41.2.2");

        let service_keys = meta.attack_info.keys().collect::<Vec<&String>>();
        let service = &meta.attack_info[service_keys[0]];

        assert_eq!(service_keys[0], "bbspriest");

        let round_hints = &service.0[&team.ip];
        //
        // let round_7_results = round_hints.get("15").unwrap().as_array().unwrap();
        //
        // assert_eq!(round_7_results[0], "username1");
        // assert_eq!(round_7_results[1], "username1.2");
        //
        // let round_8_results = round_hints.get("16").unwrap().as_array().unwrap();
        //
        // assert_eq!(round_8_results[0], "username2");
        // assert_eq!(round_8_results[1], "username2.2");
    }
}
