// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use crate::fetcher::{CompetitionData, FetchOptions, Fetcher, FetcherError};
use async_trait::async_trait;
use dashmap::DashMap;
use kriger_common::models;

pub(crate) struct DummyFetcher;

#[async_trait]
impl Fetcher for DummyFetcher {
    async fn fetch(
        &self,
        _options: &FetchOptions,
        _services: &DashMap<String, models::Service>,
    ) -> Result<CompetitionData, FetcherError> {
        Ok(CompetitionData {
            flag_hints: Some(vec![]),
        })
    }
}
