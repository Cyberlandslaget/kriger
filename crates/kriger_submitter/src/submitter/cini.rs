use std::pin::Pin;
use std::time::Duration;

use futures::Stream;
use futures::StreamExt;
use serde::Deserialize;
use tokio::time::{interval_at, Instant};
use tracing::warn;

use kriger_common::messaging::model::{FlagSubmission, FlagSubmissionResult, FlagSubmissionStatus};
use kriger_common::messaging::Message;

use crate::submitter::{SubmitError, Submitter, SubmitterCallback};

pub(crate) struct CiniSubmitter {
    pub url: String,
    pub interval: u64,
    pub token: String,
    client: reqwest::Client,
}

impl Submitter for CiniSubmitter {
    async fn run(
        &self,
        flags: Pin<
            Box<
                dyn Stream<Item = (impl Message<Payload = FlagSubmission> + Sync + 'static)> + Send,
            >,
        >,
        callback: impl SubmitterCallback + Send + Sync + 'static,
    ) -> color_eyre::Result<()> {
        let mut interval = interval_at(Instant::now(), Duration::from_secs(self.interval));

        // The first tick completes immediately
        interval.tick().await;

        loop {
            interval.tick().await;
        }
    }
}

impl CiniSubmitter {
    fn new(url: String, interval: u64, token: String) -> Self {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(0) // should disable pooling which fixes errors against some hosts
            .build()
            .expect("unable to construct reqwest client"); // TODO: Should probably return a result

        Self {
            url,
            interval,
            token,
            client,
        }
    }

    /// You can submit stolen flags by performing an HTTP PUT request to the game system at
    /// http://10.10.0.1:8080/flags. The flags must be submitted as an array of strings and the
    /// requests must contain the header X-Team-Token set to the team token given to the participants.
    async fn submit(&self, flags: &[&str]) -> Result<Vec<FlagSubmissionResult>, SubmitError> {
        let request = self
            .client
            .put(&self.url)
            .header("X-Team-Token", &self.token)
            .json(flags)
            .build()?;

        let response = self.client.execute(request).await?;
        if !response.status().is_success() {
            return Err(SubmitError::FormatError);
        }

        let flags: Vec<FlagResponse> = response.json().await?;

        // TODO: Log responses that don't make sense
        let flags = flags
            .iter()
            .map(|flag| FlagSubmissionResult {
                flag: flag.flag.to_string(),
                status: match flag.status {
                    FlagResponseStatus::Accepted => FlagSubmissionStatus::Ok,
                    FlagResponseStatus::Denied => match flag.msg.split_once(' ') {
                        None => FlagSubmissionStatus::Unknown,
                        Some((_, msg)) => self.map_response_message(&msg.to_lowercase()),
                    },
                    FlagResponseStatus::Resubmit => FlagSubmissionStatus::Resubmit,
                    FlagResponseStatus::Error => FlagSubmissionStatus::Error,
                    FlagResponseStatus::Unknown => FlagSubmissionStatus::Unknown,
                },
                points: None,
            })
            .collect();

        Ok(flags)
    }

    fn map_response_message(&self, msg: &str) -> FlagSubmissionStatus {
        if msg.contains("invalid") {
            return FlagSubmissionStatus::Invalid;
        }
        if msg.contains("nop") {
            return FlagSubmissionStatus::Nop;
        }
        if msg.contains("own") {
            return FlagSubmissionStatus::Own;
        }
        if msg.contains("old") {
            return FlagSubmissionStatus::Old;
        }
        if msg.contains("already claimed") {
            return FlagSubmissionStatus::Duplicate;
        }

        warn!("unknown response message: {msg}");
        return FlagSubmissionStatus::Unknown;
    }
}

#[derive(Deserialize, Debug)]
struct FlagResponse {
    msg: String,
    flag: String,
    status: FlagResponseStatus,
}

#[derive(Deserialize, Debug)]
enum FlagResponseStatus {
    #[serde(rename = "ACCEPTED")]
    Accepted,
    #[serde(rename = "DENIED")]
    Denied,
    #[serde(rename = "RESUBMIT")]
    Resubmit,
    #[serde(rename = "ERROR")]
    Error,
    #[serde(other)]
    Unknown,
}
