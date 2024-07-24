use crate::submitter::{SubmitError, Submitter, SubmitterCallback};
use crate::utils::futures::PollPending;
use async_trait::async_trait;
use futures::future::join_all;
use futures::Stream;
use kriger_common::messaging::model::{FlagSubmission, FlagSubmissionResult, FlagSubmissionStatus};
use kriger_common::messaging::Message;
use serde::Deserialize;
use std::collections::HashMap;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::{interval_at, Instant, MissedTickBehavior};
use tracing::{debug, error, info, instrument, warn};

pub(crate) struct CiniSubmitter {
    pub url: String,
    pub interval: u64,
    pub batch: usize,
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

            let requests = PollPending::new(&mut flags, self.batch).await;
            if requests.len() == 0 {
                debug!("there are no flags to submit, skipping");
                continue;
            }

            self.handle_submit(requests, &callback).await;
        }
    }
}

impl CiniSubmitter {
    pub(crate) fn new(url: String, interval: u64, batch: usize, token: String) -> Self {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(0) // should disable pooling which fixes errors against some hosts
            .timeout(Duration::from_secs(60)) // This was the recommended timeout. TODO: Make this configurable
            .build()
            .expect("unable to construct reqwest client"); // TODO: Should probably return a result

        Self {
            url,
            interval,
            batch,
            token,
            client,
        }
    }

    #[instrument(level="info", skip_all, fields(flag_count = %requests.len()))]
    async fn handle_submit(
        &self,
        requests: Vec<impl Message<Payload = FlagSubmission>>,
        callback: &(impl SubmitterCallback + Send + Sync + Sized),
    ) {
        info! {
            flags.count = requests.len(),
            "preparing to submit flags"
        }

        // TODO: Investigate why duplicate flags were received?
        let flags: Vec<&str> = requests
            .iter()
            .map(|msg| msg.payload().flag.as_ref())
            .collect();

        // Indicate the work is in progress. We do this concurrently.
        // FIXME: Indicating in-progress will only increase the ongoing period by another AckWait, maybe increase this periodically?
        let progress_futures = requests.iter().map(|req| req.progress());
        join_all(progress_futures).await;

        match self.submit(&flags).await {
            Ok(mut results) => {
                let futures = requests.into_iter().map(|message| {
                    Self::handle_result(results.remove(&message.payload().flag), message, callback)
                });
                join_all(futures).await;
            }
            Err(error) => {
                error! {
                    ?error,
                    "unable to submit flags"
                };
                let futures = requests.iter().map(|message| message.nak());
                join_all(futures).await;
            }
        }
    }

    /// You can submit stolen flags by performing an HTTP PUT request to the game system at
    /// http://10.10.0.1:8080/flags. The flags must be submitted as an array of strings and the
    /// requests must contain the header X-Team-Token set to the team token given to the participants.
    #[instrument(level = "info", skip_all)]
    async fn submit(
        &self,
        flags: &[&str],
    ) -> Result<HashMap<String, FlagSubmissionStatus>, SubmitError> {
        debug! {
            flags = ?flags,
            "submitting flags"
        }

        let request = self
            .client
            .put(&self.url)
            .header("X-Team-Token", &self.token)
            .json(flags)
            .build()?;

        let response = self.client.execute(request).await?;
        if !response.status().is_success() {
            let response_status = response.status().as_u16();
            let response_body = response.text().await;
            warn! {
                response.status = %response_status,
                response.body = ?response_body,
                "received a non-successful response from the flag submission api",
            }
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

    async fn handle_result(
        maybe_status: Option<FlagSubmissionStatus>,
        message: impl Message<Payload = FlagSubmission>,
        callback: &(impl SubmitterCallback + Send + Sync + Sized),
    ) {
        let payload = message.payload();
        match maybe_status {
            Some(status) => {
                debug! {
                    flag = %payload.flag,
                    flag.status = ?status,
                    "received flag submission response"
                };
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
                    Err(error) => {
                        warn! {
                            ?error,
                            "unable to publish flag submission result"
                        }
                        let _ = message.nak().await;
                    }
                }
            }
            None => {
                // ??? was??
                warn! {
                    flag = %payload.flag,
                    "submitted flag did not receive a response",
                };
                let _ = message.nak().await;
            }
        }
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
