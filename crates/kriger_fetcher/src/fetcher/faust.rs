use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use dashmap::DashMap;
use futures::future::join_all;
use kriger_common::messaging::model::FlagHint;
use kriger_common::messaging::{Bucket, Messaging, MessagingError};
use kriger_common::runtime::AppRuntime;
use serde::{self, Deserialize};
use std::collections::HashMap;
use tracing::{debug, error, instrument};

use super::{Fetcher, FetcherError};

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
    async fn get_attack_into(&self) -> Result<AttackInfo, FetcherError> {
        let info: AttackInfo = self.client.get(&self.url).send().await?.json().await?;
        Ok(info)
    }

    #[instrument(level = "DEBUG", skip_all, fields(team_id, service, hint))]
    async fn handle_flag_insertion(
        &self,
        bucket: &impl Bucket,
        team_id: String,
        service: String,
        hint: serde_json::Value,
    ) {
        match serde_json::to_vec(&hint) {
            Ok(serialized) => {
                let data = FlagHint {
                    team_id,
                    service,
                    hint,
                };
                let key = STANDARD_NO_PAD.encode(&serialized);
                match bucket.create(&key, &data).await {
                    Err(MessagingError::KeyValueConflictError) => {
                        // Ignore
                    }
                    Err(error) => {
                        error! {
                            ?error,
                            "unable to insert the flag hint to the k/v store"
                        }
                    }
                    _ => {}
                }
            }
            Err(error) => {
                error! {
                    ?error,
                    "unable to serialize the hint"
                }
            }
        }
    }
}

#[async_trait]
impl Fetcher for FaustFetcher {
    #[instrument(skip_all)]
    async fn run(
        &self,
        runtime: &AppRuntime,
        _services: &DashMap<String, kriger_common::messaging::model::Service>,
    ) -> Result<(), FetcherError> {
        let hints_bucket = runtime.messaging.data_hints().await?;
        let info = self.get_attack_into().await?;
        
        debug! {
            team_count = info.teams.len(),
            service_count = info.flag_ids.len(),
            "fetched attack info"
        }

        let mut tasks = Vec::new();

        for (service, map) in info.flag_ids {
            for (team_id, hints) in map.0 {
                for hint in hints {
                    // FIXME: Functional programming?
                    tasks.push(self.handle_flag_insertion(
                        &hints_bucket,
                        team_id.clone(),
                        service.clone(),
                        hint,
                    ));
                }
            }
        }

        join_all(tasks).await;

        Ok(())
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
