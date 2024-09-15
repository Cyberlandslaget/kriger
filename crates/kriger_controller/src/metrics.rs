use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::registry::Registry;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub(crate) struct ExploitLabels {
    pub exploit: String,
}

#[derive(Default)]
pub(crate) struct ControllerMetrics {
    pub requests: Family<ExploitLabels, Counter>,
    pub complete: Family<ExploitLabels, Counter>,
    pub error: Family<ExploitLabels, Counter>,
}

impl ControllerMetrics {
    pub(crate) fn register(&self, registry: &mut Registry) {
        registry.register(
            "kriger_controller_reconciliation_requests",
            "The number of reconciliation requests",
            self.requests.clone(),
        );
        registry.register(
            "kriger_controller_reconciliation_complete",
            "The number of completed reconciliations",
            self.complete.clone(),
        );
        registry.register(
            "kriger_controller_reconciliation_error",
            "The number of errored reconciliations",
            self.error.clone(),
        );
    }
}
