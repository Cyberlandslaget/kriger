// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use crate::fetcher::{CompetitionData, FetchOptions, Fetcher, FetcherError, FlagHint};
use async_trait::async_trait;
use color_eyre::eyre;
use dashmap::DashMap;
use kriger_common::models;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use tracing::instrument;

#[derive(Deserialize, PartialEq)]
struct MetaResponse {
    teams: Vec<Team>,
    services: Vec<Service>,
}

#[derive(Deserialize, PartialEq)]
struct Team {
    id: u32,
    shortname: String,
}

#[derive(Deserialize, PartialEq)]
struct Service {
    id: String,
    shortname: String,
}

/// Flag IDs are indexed by service name, team id, and round number.
type FlagIdsResponse = HashMap<String, HashMap<String, HashMap<String, Value>>>;

pub(crate) struct CiniFetcher {
    client: reqwest::Client,
    endpoint: String,
}

impl CiniFetcher {
    pub(crate) fn new<S: Into<String>>(endpoint: S) -> Self {
        let client = reqwest::Client::new();

        CiniFetcher {
            client,
            endpoint: endpoint.into(),
        }
    }

    async fn get_meta(&self) -> eyre::Result<MetaResponse> {
        let res = self.client.get(&self.endpoint).send().await?;
        let body: MetaResponse = res.json().await?;

        Ok(body)
    }

    async fn get_flag_ids(&self) -> Result<FlagIdsResponse, FetcherError> {
        let res = self
            .client
            .get(format!("{}/flagIds", &self.endpoint))
            .send()
            .await?;
        let flag_ids: FlagIdsResponse = res.json().await?;

        Ok(flag_ids)
    }
}

#[async_trait]
impl Fetcher for CiniFetcher {
    #[instrument(skip_all)]
    async fn fetch(
        &self,
        options: &FetchOptions,
        _services: &DashMap<String, models::Service>,
    ) -> Result<CompetitionData, FetcherError> {
        let mut flag_hints: Option<Vec<FlagHint>> = None;

        let res = self.get_flag_ids().await?;

        if options.require_hints {
            let mut hints = Vec::new();
            for (service, teams) in res {
                for (team_id, rounds) in teams {
                    for (round_id, hint) in rounds {
                        hints.push(FlagHint {
                            round: round_id.parse().ok(),
                            team_id: team_id.clone(),
                            service: service.clone(),
                            hint,
                        })
                    }
                }
            }
            flag_hints = Some(hints);
        }

        Ok(CompetitionData { flag_hints })
    }
}

#[cfg(test)]
mod tests {
    use crate::fetcher::cini::{FlagIdsResponse, MetaResponse};

    #[test]
    fn should_deserialize_meta_response() {
        const META_JSON: &str = r#"{"teams":[{"id":0,"shortname":"cybersecnatlab"},{"id":1,"shortname":"albania"},{"id":2,"shortname":"australia"},{"id":3,"shortname":"austria"},{"id":4,"shortname":"belgium"},{"id":5,"shortname":"bulgaria"},{"id":6,"shortname":"canada"},{"id":7,"shortname":"chile"},{"id":8,"shortname":"costa-rica"},{"id":9,"shortname":"croatia"},{"id":10,"shortname":"cyprus"},{"id":11,"shortname":"czech"},{"id":12,"shortname":"denmark"},{"id":13,"shortname":"estonia"},{"id":14,"shortname":"finland"},{"id":15,"shortname":"france"},{"id":16,"shortname":"georgia"},{"id":17,"shortname":"germany"},{"id":18,"shortname":"greece"},{"id":19,"shortname":"hungary"},{"id":20,"shortname":"iceland"},{"id":21,"shortname":"ireland"},{"id":22,"shortname":"italy"},{"id":23,"shortname":"kosovo"},{"id":24,"shortname":"latvia"},{"id":25,"shortname":"liechtenstein"},{"id":26,"shortname":"luxembourg"},{"id":27,"shortname":"malta"},{"id":28,"shortname":"netherlands"},{"id":29,"shortname":"norway"},{"id":30,"shortname":"poland"},{"id":31,"shortname":"portugal"},{"id":32,"shortname":"romania"},{"id":33,"shortname":"serbia"},{"id":34,"shortname":"slovakia"},{"id":35,"shortname":"slovenia"},{"id":36,"shortname":"spain"},{"id":37,"shortname":"sweden"},{"id":38,"shortname":"switzerland"},{"id":39,"shortname":"usa"}],"services":[{"id":"CheesyCheats-1","shortname":"CheesyCheats-1"},{"id":"CheesyCheats-2","shortname":"CheesyCheats-2"},{"id":"Polls","shortname":"Polls"},{"id":"GadgetHorse-1","shortname":"GadgetHorse-1"},{"id":"GadgetHorse-2","shortname":"GadgetHorse-2"},{"id":"MineCClicker","shortname":"MineCClicker"}]}"#;

        let maybe_meta = serde_json::from_str(META_JSON);
        assert!(maybe_meta.is_ok());

        let meta: MetaResponse = maybe_meta.unwrap();
        assert_eq!(meta.teams.len(), 40);
        assert_eq!(meta.services.len(), 6);

        assert_eq!(meta.teams[0].id, 0);
        assert_eq!(meta.teams[0].shortname, "cybersecnatlab");
        assert_eq!(meta.teams[29].id, 29);
        assert_eq!(meta.teams[29].shortname, "norway");

        assert_eq!(meta.services[0].id, "CheesyCheats-1");
        assert_eq!(meta.services[0].shortname, "CheesyCheats-1");
        assert_eq!(meta.services[3].id, "GadgetHorse-1");
        assert_eq!(meta.services[3].shortname, "GadgetHorse-1");
    }

    #[test]
    fn should_deserialize_flag_ids_response_filtered_by_service() {
        const FLAGIDS_JSON: &str = r#"{"foobar":{"1":{"5":{"flag_id_description":"flag_id_service_foobar_team_1_round_5"}}}}"#;

        let maybe_res = serde_json::from_str(FLAGIDS_JSON);
        assert!(maybe_res.is_ok());

        let res: FlagIdsResponse = maybe_res.unwrap();
        assert_eq!(res.len(), 1);
        assert!(res.contains_key("foobar"));

        let team_map = &res["foobar"];
        assert_eq!(team_map.len(), 1);
        assert!(team_map.contains_key("1"));

        let round_map = &team_map["1"];
        assert_eq!(round_map.len(), 1);
        assert!(round_map.contains_key("5"));
    }
}
