use super::{OldFetcher, FetcherError, Service, ServiceMap, ServiceOld, TeamService};
use serde::{self, Deserialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttackInfo {
    #[allow(dead_code)]
    pub available_teams: Vec<String>,
    pub services: HashMap<String, ServiceOld>,
}

// the only real thing it does is take the single flagid and make it into a vector with one element
impl From<HashMap<String, ServiceOld>> for ServiceMap {
    fn from(value: HashMap<String, ServiceOld>) -> Self {
        ServiceMap(
            value
                .into_iter()
                .map(|(service_name, teams)| {
                    (
                        service_name,
                        Service {
                            teams: teams
                                .0
                                .into_iter()
                                .map(|(team_ip, ticks)| {
                                    (
                                        team_ip,
                                        TeamService {
                                            ticks: ticks
                                                .0
                                                .into_iter()
                                                .map(|(tick_nr, value)| (tick_nr, vec![value])) // val -> vec![val]
                                                .collect(),
                                        },
                                    )
                                })
                                .collect(),
                        },
                    )
                })
                .collect(),
        )
    }
}

#[derive(Debug)]
pub struct EnowarsFetcher {
    client: reqwest::Client,
    endpoint: String,
    ips_endpoint: String,
}

impl EnowarsFetcher {
    pub fn new(endpoint: String, ips_endpoint: String) -> Self {
        let client = reqwest::Client::new();

        Self {
            client,
            endpoint,
            ips_endpoint,
        }
    }
}

impl OldFetcher for EnowarsFetcher {
    async fn services(&self) -> Result<ServiceMap, FetcherError> {
        // TODO handle failures more gracefully (retry?)
        let resp: AttackInfo = self.client.get(&self.endpoint).send().await?.json().await?;

        Ok(resp.services.into())
    }

    async fn ips(&self) -> Result<Vec<String>, FetcherError> {
        let resp: String = self
            .client
            .get(&self.ips_endpoint)
            .send()
            .await?
            .text()
            .await?;

        let ips = resp.trim().lines().map(|s| s.trim().to_string()).collect();

        Ok(ips)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::Filter;

    // from https://7.enowars.com/setup
    const JSON: &str = r#"
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

    fn eno_deser() -> ServiceMap {
        let attack_info: AttackInfo = serde_json::from_str(JSON).unwrap();

        attack_info.services.into()
    }

    #[tokio::test]
    /// Fetch the response from a local test server
    async fn eno_local_test() {
        let gameserver = tokio::spawn(async move {
            // note, content-type not set probably
            let endpoint = warp::path!("endpoint").map(|| JSON);

            warp::serve(endpoint).run(([127, 0, 0, 1], 9999)).await
        });

        let fetcher =
            EnowarsFetcher::new("http://localhost:9999/endpoint".to_string(), "".to_string());

        let services = fetcher.services().await.unwrap();

        dbg!(&services);

        for (service, service_info) in services.0.iter() {
            for (ip, ticks) in service_info.teams.iter() {
                for (tick, flagids) in ticks.ticks.iter() {
                    for flagid in flagids {
                        println!("{} {} {} {}", service, ip, tick, flagid);
                    }
                }
            }
        }

        // make sure we got the same content as directly deserializing locally
        assert_eq!(&services, &eno_deser());

        gameserver.abort();
    }
}
