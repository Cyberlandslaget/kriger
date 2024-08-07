use crate::fetcher::{Fetcher, FetcherError};
use async_trait::async_trait;
use color_eyre::eyre;
use color_eyre::owo_colors::OwoColorize;
use dashmap::DashMap;
use futures::future::join_all;
use kriger_common::messaging::model;
use kriger_common::runtime::AppRuntime;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{instrument, warn};

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

    async fn get_flag_ids_by_service<S: AsRef<str>>(
        &self,
        service: S,
    ) -> eyre::Result<FlagIdsResponse> {
        let res = self
            .client
            .get(format!("{}/flagIds", &self.endpoint))
            .query(&[("service", service.as_ref())])
            .send()
            .await?;
        let flag_ids: FlagIdsResponse = res.json().await?;

        Ok(flag_ids)
    }

    #[instrument(skip_all, fields(service))]
    async fn handle_service<S: AsRef<str>>(&self, runtime: AppRuntime, service: S) {
        match self.get_flag_ids_by_service(service.as_ref()).await {
            Ok(res) => match res.get(service.as_ref()) {
                Some(map) => {
                }
                None => {
                    warn! {
                        "unable to find the service in the flag ids response"
                    }
                }
            },
            Err(error) => {
                warn! {
                    ?error,
                    "unable to fetch flag ids"
                }
            }
        }
    }
}

#[async_trait]
impl Fetcher for CiniFetcher {
    async fn run(
        &self,
        runtime: &AppRuntime,
        services: &DashMap<String, model::Service>,
    ) -> Result<(), FetcherError> {
        let mut tasks = Vec::new();

        for service in services.iter() {
            if !service.has_hint {
                continue;
            }

            tasks.push(self.handle_service(runtime.clone(), service.name.clone()));
        }

        join_all(tasks).await;

        Ok(())
    }
}

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

/// A map consisting of flag ID maps associated with service names
type FlagIdsResponse = HashMap<String, TeamFlagIdMap>;

/// A map consisting of flag IDs associated with team IDs.
type TeamFlagIdMap = HashMap<String, Vec<Value>>;

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
        const FLAGIDS_JSON: &str = r#"{"MineCClicker":{"0":[{"boardname":"dTaU0T7TGkoufV"}],"1":[{"boardname":"QRDk8Q4ajUHdLb"}],"2":[{"boardname":"KkmyAUTIRhViuP8tTi5b"}],"3":[{"boardname":"AWxS5tCVAg6Ts0Oy2HFpmxo9kHhoAR"}],"4":[{"boardname":"NivajZsi4iaZVZQRMqj7aTRSQDUK3W"}],"5":[{"boardname":"IFXj86iCGh5ia"}],"6":[{"boardname":"zgOYyAXtmAnmT3aiBG8VEic"}],"7":[{"boardname":"SL6VtqTegCB9btlseZ6"}],"8":[{"boardname":"ZVui0Zgs2QfBsGHb"}],"9":[{"boardname":"OIPWYSCSg6"}],"10":[{"boardname":"aysvU8LU8Wd1J80rXZGwgFbmz"}],"11":[{"boardname":"aBB9jBga2Vglq40xKHTqpBiW8io7ix"}],"12":[{"boardname":"avPsojxjk171O91R8A4d"}],"13":[{"boardname":"vZ671lhc"}],"14":[{"boardname":"5pMX84DxCE"}],"15":[{"boardname":"XLdoR67bQpl0Sq52jJJOmv0"}],"16":[{"boardname":"2CnJFyVS"}],"17":[{"boardname":"ovppp82mLmHou1lVWteTxtbubiHuHzk"}],"18":[{"boardname":"pMdBnn2mg5F6JrCFFXOxuU7le"}],"19":[{"boardname":"poPWsFHTq0ULfV1IX0vEzsC3jO8hYT"}],"20":[{"boardname":"m0eu8hMC8K"}],"21":[{"boardname":"ap8kgW4juu3jtA"}],"22":[{"boardname":"ALj8kpBO26jaHLIIBVXAcpNNqTUeu66"}],"23":[{"boardname":"O7ld92p8Z21ZtTdfKJ"}],"24":[{"boardname":"rakVgQaQlqa8oJ2VsUvQLSLj9JeWDKu"}],"25":[{"boardname":"QzLPiGkObkypfZAfJuBTA"}],"26":[{"boardname":"OvcHppEjKj31iBLcfkzrhVbCH8eW"}],"27":[{"boardname":"ubGN4jRE8"}],"28":[{"boardname":"sa0Q1I3wEk"}],"29":[{"boardname":"pDrxv2cb9Dbu9TCRlODuCXSg"}],"30":[{"boardname":"a2iYJesTlzn"}],"31":[{"boardname":"VbmHpEDis2re7CA1KIiJXQKu0"}],"32":[{"boardname":"Vrxi2E1gIfIt"}],"33":[{"boardname":"6Q5tUiB1cR6ba1MH0IydtIgw7H"}],"34":[{"boardname":"ORaRlEyljQ6t0uxMMWA"}],"35":[{"boardname":"vp5xf28coeDD"}],"36":[{"boardname":"ey0kQZ7eXnZsQp6ga0wJ8hg"}],"37":[{"boardname":"zVM4Y0JICTX"}],"38":[{"boardname":"qZdcRZ8AHwKWiuM"}],"39":[{"boardname":"knnoFW8jKgRXKccuS5shN0"}]}}"#;

        let maybe_res = serde_json::from_str(FLAGIDS_JSON);
        assert!(maybe_res.is_ok());

        let res: FlagIdsResponse = maybe_res.unwrap();
        assert_eq!(res.len(), 1);
        assert!(res.contains_key("MineCClicker"));

        let team_map = &res["MineCClicker"];
        assert_eq!(team_map.len(), 40);
        assert!(team_map.contains_key("29"));

        let nor_flag_ids = &team_map["29"];
        assert_eq!(nor_flag_ids.len(), 1);
    }
}
