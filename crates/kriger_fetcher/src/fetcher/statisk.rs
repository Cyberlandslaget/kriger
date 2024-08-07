use std::collections::HashMap;

use super::{OldFetcher, FetcherError, ServiceMap};

#[derive(Debug)]
pub struct StaticFetcher {
    pub ids: Vec<u8>,
}

impl OldFetcher for StaticFetcher {
    async fn services(&self) -> Result<ServiceMap, FetcherError> {
        Ok(ServiceMap(HashMap::new()))
    }

    async fn ips(&self) -> Result<Vec<String>, FetcherError> {
        Ok(self.ids.iter().map(|i| format!("10.10.{i}.2")).collect())
    }
}
