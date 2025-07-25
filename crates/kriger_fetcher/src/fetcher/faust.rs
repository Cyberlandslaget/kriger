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
    pub teams: Vec<i32>,
    pub flag_ids: HashMap<String, ServiceMap>,
}

// teamid -> `Vec<flagid>`
#[derive(Deserialize, Debug)]
pub struct ServiceMap(HashMap<String, Vec<serde_json::Value>>);

#[derive(Debug)]
pub struct FaustFetcher {
    client: reqwest::Client,
    url: String,
    ip_format: String,
}

impl FaustFetcher {
    pub fn new(url: String, ip_format: String) -> Self {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(0) // should disable pooling which fixes errors against some hosts
            .build()
            .unwrap();

        Self {
            url,
            ip_format,
            client,
        }
    }

    /// This fetches the FAUST GameServer's team.json data
    async fn get_attack_info(&self) -> Result<AttackInfo, FetcherError> {
        let info: AttackInfo = self.client.get(&self.url).send().await?.json().await?;
        Ok(info)
    }
}

#[async_trait]
impl Fetcher for FaustFetcher {
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

        let mut flag_hints = Vec::new();
        for (service, map) in info.flag_ids {
            for (team_id, hints) in map.0 {
                for hint in hints {
                    flag_hints.push(FlagHint {
                        round: None,
                        team_id: team_id.clone(),
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use warp::Filter;
//
//     const TEAMS_JSON: &str = r#"
//             {
//                 "teams": [
//                     2
//                 ],
//                 "flag_ids": {
//                     "service_1": {
//                         "2": [
//                                 [
//                                     [ "user73" ],
//                                     [ "user5" ]
//                                 ],
//                                 [
//                                     [ "user96" ],
//                                     [ "user314" ]
//                                 ]
//                         ]
//                     }
//                 }
//             }"#;
//
//     const SCOREBOARD_JSON: &str = r#"
//             {
//                 "tick": 271
//             }
//     "#;
//
//     #[tokio::test]
//     async fn faust_local_test() {
//         let gameserver = tokio::spawn(async move {
//             let teams = warp::path!("teams").map(|| TEAMS_JSON);
//             let scoreboard = warp::path!("scoreboard").map(|| SCOREBOARD_JSON);
//             warp::serve(teams.or(scoreboard))
//                 .run(([127, 0, 0, 1], 8888))
//                 .await
//         });
//
//         let fetcher = FaustFetcher::new(
//             "http://localhost:8888/teams".to_string(),
//             "http://localhost:8888/scoreboard".to_string(),
//             "1.20.{x}.1".to_string(),
//         );
//
//         let services = fetcher.services().await.unwrap();
//
//         dbg!(&services);
//
//         gameserver.abort();
//     }
// }
