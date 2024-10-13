// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::histogram::{exponential_buckets, Histogram};
use prometheus_client::registry::Registry;

pub(crate) struct FetcherMetrics {
    pub start: Counter,
    pub complete: Counter,
    pub error: Counter,
    pub duration: Histogram,
}

impl FetcherMetrics {
    pub(crate) fn register(&self, registry: &mut Registry) {
        registry.register(
            "kriger_fetcher_start",
            "The number of fetch start",
            self.start.clone(),
        );
        registry.register(
            "kriger_fetcher_complete",
            "The number of fetch complete",
            self.complete.clone(),
        );
        registry.register(
            "kriger_fetcher_error",
            "The number of fetch errors",
            self.error.clone(),
        );
        registry.register(
            "kriger_fetcher_duration_seconds",
            "A histogram for the amount of time taken to fetch",
            self.duration.clone(),
        );
    }
}

impl Default for FetcherMetrics {
    fn default() -> Self {
        Self {
            start: Default::default(),
            complete: Default::default(),
            error: Default::default(),
            duration: Histogram::new(exponential_buckets(0.001, 2.0, 18)),
        }
    }
}
