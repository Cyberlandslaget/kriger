// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use crate::submitter::{FormatErrorKind, SubmitError, Submitter};
use async_trait::async_trait;
use kriger_common::models;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, instrument, warn};

pub(crate) struct CiniSubmitter {
    url: String,
    token: String,
    client: reqwest::Client,
}

#[async_trait]
impl Submitter for CiniSubmitter {
    #[instrument(skip_all)]
    async fn submit(
        &self,
        flags: &[&str],
    ) -> Result<HashMap<String, models::FlagSubmissionStatus>, SubmitError> {
        debug! {
            flags = ?flags,
            "submitting flags"
        }

        // You can submit stolen flags by performing an HTTP PUT request to the game system at
        // http://10.10.0.1:8080/flags. The flags must be submitted as an array of strings and the
        // requests must contain the header X-Team-Token set to the team token given to the participants.
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
            return Err(SubmitError::FormatError(FormatErrorKind::ErrorResponse));
        }

        let flag_responses: Vec<FlagResponse> = response.json().await?;

        let flag_responses = flag_responses
            .into_iter()
            .map(|flag| {
                (
                    flag.flag,
                    match flag.status {
                        FlagResponseStatus::Accepted => models::FlagSubmissionStatus::Ok,
                        FlagResponseStatus::Denied => match flag.msg.split_once(' ') {
                            None => models::FlagSubmissionStatus::Unknown,
                            Some((_, msg)) => map_response_message(&msg.to_lowercase()),
                        },
                        FlagResponseStatus::Resubmit => models::FlagSubmissionStatus::Resubmit,
                        FlagResponseStatus::Error => models::FlagSubmissionStatus::Error,
                        FlagResponseStatus::Unknown => models::FlagSubmissionStatus::Unknown,
                    },
                )
            })
            .collect();

        Ok(flag_responses)
    }
}

impl CiniSubmitter {
    pub(crate) fn new(url: String, token: String) -> Self {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(0) // should disable pooling which fixes errors against some hosts
            .timeout(Duration::from_secs(60)) // This was the recommended timeout. TODO: Make this configurable
            .build()
            .expect("unable to construct reqwest client"); // TODO: Should probably return a result

        Self { url, token, client }
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

fn map_response_message(msg: &str) -> models::FlagSubmissionStatus {
    if msg.contains("invalid") {
        return models::FlagSubmissionStatus::Invalid;
    }
    if msg.contains("nop") {
        return models::FlagSubmissionStatus::Nop;
    }
    if msg.contains("own") {
        return models::FlagSubmissionStatus::Own;
    }
    if msg.contains("old") {
        return models::FlagSubmissionStatus::Old;
    }
    if msg.contains("already claimed") {
        return models::FlagSubmissionStatus::Duplicate;
    }
    if msg.contains("didn't terminate") {
        return models::FlagSubmissionStatus::Stale;
    }

    warn!("unknown response message: {msg}");
    models::FlagSubmissionStatus::Unknown
}
