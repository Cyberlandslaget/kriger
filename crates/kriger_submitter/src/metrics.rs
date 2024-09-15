use kriger_common::models;
use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::histogram::{exponential_buckets, Histogram};
use prometheus_client::registry::Registry;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue)]
pub(crate) enum FlagSubmissionStatus {
    Ok,
    Duplicate,
    Own,
    Nop,
    Old,
    Invalid,
    Resubmit,
    Error,
    Stale,
    Unknown,
}

impl Into<FlagSubmissionStatus> for &models::FlagSubmissionStatus {
    fn into(self) -> FlagSubmissionStatus {
        match self {
            models::FlagSubmissionStatus::Ok => FlagSubmissionStatus::Ok,
            models::FlagSubmissionStatus::Duplicate => FlagSubmissionStatus::Duplicate,
            models::FlagSubmissionStatus::Own => FlagSubmissionStatus::Own,
            models::FlagSubmissionStatus::Nop => FlagSubmissionStatus::Nop,
            models::FlagSubmissionStatus::Old => FlagSubmissionStatus::Old,
            models::FlagSubmissionStatus::Invalid => FlagSubmissionStatus::Invalid,
            models::FlagSubmissionStatus::Resubmit => FlagSubmissionStatus::Resubmit,
            models::FlagSubmissionStatus::Error => FlagSubmissionStatus::Error,
            models::FlagSubmissionStatus::Stale => FlagSubmissionStatus::Stale,
            models::FlagSubmissionStatus::Unknown => FlagSubmissionStatus::Unknown,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub(crate) struct FlagSubmissionStatusLabels {
    pub(crate) status: FlagSubmissionStatus,
}

pub(crate) struct SubmitterMetrics {
    pub start: Counter,
    pub complete: Counter,
    pub error: Counter,
    pub duration: Histogram,
    pub flag_submissions: Counter,
    pub flag_results: Family<FlagSubmissionStatusLabels, Counter>,
}

impl SubmitterMetrics {
    pub(crate) fn register(&self, registry: &mut Registry) {
        registry.register(
            "kriger_submitter_submission_start",
            "The number of submission batch start",
            self.start.clone(),
        );
        registry.register(
            "kriger_submitter_submission_complete",
            "The number of submission batch complete",
            self.complete.clone(),
        );
        registry.register(
            "kriger_submitter_submission_error",
            "The number of submission batch error",
            self.error.clone(),
        );
        registry.register(
            "kriger_submitter_submission_duration_seconds",
            "A histogram for the amount of time taken to submit flags",
            self.duration.clone(),
        );
        registry.register(
            "kriger_submitter_flag_submissions",
            "The number of flag submissions consumed",
            self.flag_submissions.clone(),
        );
        registry.register(
            "kriger_submitter_flag_results",
            "The number of flag results received",
            self.flag_results.clone(),
        );
    }
}

impl Default for SubmitterMetrics {
    fn default() -> Self {
        Self {
            start: Default::default(),
            complete: Default::default(),
            error: Default::default(),
            duration: Histogram::new(exponential_buckets(0.001, 2.0, 18)),
            flag_submissions: Default::default(),
            flag_results: Default::default(),
        }
    }
}
