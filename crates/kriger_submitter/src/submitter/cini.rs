use async_trait::async_trait;
use std::collections::HashMap;
use std::pin::Pin;
use std::time::Duration;

use futures::Stream;
use serde::Deserialize;
use tokio::time::{interval_at, Instant, MissedTickBehavior};
use tracing::{debug, error, warn};

use kriger_common::messaging::model::{FlagSubmission, FlagSubmissionResult, FlagSubmissionStatus};
use kriger_common::messaging::Message;

use crate::submitter::{SubmitError, Submitter, SubmitterCallback};
use crate::utils::futures::PollPending;

pub(crate) struct CiniSubmitter {
    pub url: String,
    pub interval: u64,
    pub token: String,
    client: reqwest::Client,
}

#[async_trait]
impl Submitter for CiniSubmitter {
    async fn run(
        &self,
        mut flags: Pin<
            Box<
                dyn Stream<Item = (impl Message<Payload = FlagSubmission> + Sync + 'static)> + Send,
            >,
        >,
        callback: impl SubmitterCallback + Send + Sync + 'static,
    ) -> color_eyre::Result<()> {
        let mut interval = interval_at(Instant::now(), Duration::from_secs(self.interval));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        // The first tick completes immediately
        interval.tick().await;

        loop {
            debug!("submitter tick");
            interval.tick().await;

            let requests = PollPending::new(&mut flags, 100).await;
            let flags: Vec<&str> = requests
                .iter()
                .map(|msg| msg.payload().flag.as_ref())
                .collect();
            debug!("submitting flags: {flags:?}");
            for request in &requests {
                // TODO: parallelize
                let _ = request.progress().await;
            }
            match self.submit(&flags).await {
                Ok(mut results) => {
                    for message in requests {
                        let payload = message.payload();
                        match results.remove(&payload.flag) {
                            Some(status) => {
                                debug!(
                                    "flag submission response for flag `{}`: {:?}",
                                    &payload.flag, status
                                );
                                // TODO: Can't we move??
                                let res = callback
                                    .submit(
                                        &payload.flag,
                                        FlagSubmissionResult {
                                            flag: payload.flag.clone(),
                                            team_id: payload.team_id.clone(),
                                            service: payload.service.clone(),
                                            exploit: payload.exploit.clone(),
                                            status,
                                            points: None,
                                        },
                                    )
                                    .await;
                                match res {
                                    Ok(_) => {
                                        let _ = message.ack().await;
                                    }
                                    Err(err) => {
                                        warn!("unable to submit result: {err:?}");
                                        let _ = message.nak().await;
                                    }
                                }
                            }
                            None => {
                                // ??? was??
                                warn!(
                                    "submitted flag did not receive a response, flag: `{}`",
                                    &payload.flag
                                );
                                let _ = message.nak().await;
                            }
                        }
                    }
                }
                Err(err) => {
                    error!("unable to submit: {err:?}");
                    // TODO: Parallelize
                    for message in requests {
                        // TODO: Send message that an attempt was done
                        let _ = message.nak().await;
                    }
                }
            }
        }
    }
}

impl CiniSubmitter {
    pub(crate) fn new(url: String, interval: u64, token: String) -> Self {
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
    async fn submit(
        &self,
        flags: &[&str],
    ) -> Result<HashMap<String, FlagSubmissionStatus>, SubmitError> {
        let request = self
            .client
            .put(&self.url)
            .header("X-Team-Token", &self.token)
            .json(flags)
            .build()?;

        let response = self.client.execute(request).await?;
        if !response.status().is_success() {
            warn!(
                "received a non-successful response ({}): {:?}",
                response.status(),
                response.text().await
            );
            return Err(SubmitError::FormatError);
        }

        let flag_responses: Vec<FlagResponse> = response.json().await?;

        // TODO: Log responses that don't make sense
        let flag_responses = flag_responses
            .into_iter()
            .map(|flag| {
                (
                    flag.flag,
                    match flag.status {
                        FlagResponseStatus::Accepted => FlagSubmissionStatus::Ok,
                        FlagResponseStatus::Denied => match flag.msg.split_once(' ') {
                            None => FlagSubmissionStatus::Unknown,
                            Some((_, msg)) => self.map_response_message(&msg.to_lowercase()),
                        },
                        FlagResponseStatus::Resubmit => FlagSubmissionStatus::Resubmit,
                        FlagResponseStatus::Error => FlagSubmissionStatus::Error,
                        FlagResponseStatus::Unknown => FlagSubmissionStatus::Unknown,
                    },
                )
            })
            .collect();

        Ok(flag_responses)
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
