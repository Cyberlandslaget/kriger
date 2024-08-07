use crate::fetcher::{Fetcher, FetcherError};
use async_trait::async_trait;
use dashmap::DashMap;
use kriger_common::messaging::model::Service;
use kriger_common::runtime::AppRuntime;

pub(crate) struct DummyFetcher;

#[async_trait]
impl Fetcher for DummyFetcher {
    async fn run(
        &self,
        _runtime: &AppRuntime,
        _services: &DashMap<String, Service>,
    ) -> Result<(), FetcherError> {
        Ok(())
    }
}
