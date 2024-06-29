use serde::{self, Deserialize};
use std::collections::HashMap;
use tracing::warn;

use super::{Fetcher, FetcherError, Service, ServiceMap, TeamService};

#[derive(Deserialize, Debug)]
pub struct AttackInfo {
    pub teams: Vec<i32>,
    // TODO! should also accept <i32, _> and convert the i32 to String...
    pub flag_ids: HashMap<String, ServiceContent>,
}

#[derive(Deserialize, Debug)]
pub struct Scoreboard {
    pub tick: i32,
}

/// teamid -> `Vec<flagid>`
#[derive(Deserialize, Debug)]
pub struct ServiceContent(HashMap<String, serde_json::Value>); // treat all the flagids as one

#[derive(Debug)]
pub struct FaustFetcher {
    client: reqwest::Client,
    teams: String,
    format: String,
    scoreboard: String,
}

impl FaustFetcher {
    pub fn new(teams: String, scoreboard: String, format: String) -> Self {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(0) // should disable pooling which fixes errors against some hosts
            .build()
            .unwrap();

        Self {
            client,
            teams,
            scoreboard,
            format,
        }
    }
}

impl Fetcher for FaustFetcher {
    async fn services(&self) -> Result<ServiceMap, FetcherError> {
        let scoreboard: Scoreboard = self
            .client
            .get(&self.scoreboard)
            .send()
            .await?
            .json()
            .await?;

        let resp: AttackInfo = self.client.get(&self.teams).send().await?.json().await?;

        let mut services = HashMap::new();
        for (service, content) in resp.flag_ids {
            let mut service_content = HashMap::new();

            // shitty solution: we dont know which flagid is for which tick, so just give all the
            // current ones for the current tick\
            // the fetcher routine should discard the duplicates

            // on cold start: ex. 5 flagids sent for current tick
            // every tick afterwards: just 1 flagid, because 4 others are known

            let current_tick = scoreboard.tick;

            for (team, flagids) in content.0 {
                // faust gives an array of the last few flagids here, extract them manually :grimace:
                let flagids = match flagids.as_array() {
                    Some(a) => a,
                    None => {
                        warn!("Should be array but isn't");
                        continue;
                    }
                }
                    .to_owned();

                let team = team.parse::<i32>().unwrap();
                let team = self.format.replace("{x}", &format!("{}", team));

                let mut ticks = HashMap::new();
                ticks.insert(current_tick, flagids); // just this one

                service_content.insert(team, TeamService { ticks });
            }
            services.insert(
                service,
                Service {
                    teams: service_content,
                },
            );
        }

        Ok(ServiceMap(services))
    }

    async fn ips(&self) -> Result<Vec<String>, FetcherError> {
        let resp: AttackInfo = self.client.get(&self.teams).send().await?.json().await?;

        let ips = resp
            .teams
            .into_iter()
            .map(|team_nr| self.format.replace("{x}", &format!("{}", team_nr)))
            .collect();

        Ok(ips)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::Filter;

    const TEAMS_JSON: &str = r#"
            {
                "teams": [
                    2
                ],
                "flag_ids": {
                    "service_1": {
                        "2": [
                                [
                                    [ "user73" ],
                                    [ "user5" ]
                                ],
                                [
                                    [ "user96" ],
                                    [ "user314" ]
                                ]
                        ]
                    }
                }
            }"#;

    const SCOREBOARD_JSON: &str = r#"
            {
                "tick": 271
            }
    "#;

    #[tokio::test]
    async fn faust_local_test() {
        let gameserver = tokio::spawn(async move {
            let teams = warp::path!("teams").map(|| TEAMS_JSON);
            let scoreboard = warp::path!("scoreboard").map(|| SCOREBOARD_JSON);
            warp::serve(teams.or(scoreboard))
                .run(([127, 0, 0, 1], 8888))
                .await
        });

        let fetcher = FaustFetcher::new(
            "http://localhost:8888/teams".to_string(),
            "http://localhost:8888/scoreboard".to_string(),
            "1.20.{x}.1".to_string(),
        );

        let services = fetcher.services().await.unwrap();

        dbg!(&services);

        gameserver.abort();
    }
}
