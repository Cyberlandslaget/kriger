// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

mod metrics;
mod submitter;
mod utils;

use crate::metrics::{FlagSubmissionStatusLabels, SubmitterMetrics};
use crate::submitter::{Submitter, SubmitterConfig};
use async_nats::jetstream::AckKind;
use color_eyre::eyre;
use color_eyre::eyre::Context;
use futures::future::join_all;
use futures::StreamExt;
use kriger_common::messaging::nats::{Fetcher, MessageWrapper, MessagingServiceError};
use kriger_common::messaging::services::flags::FlagsService;
use kriger_common::server::runtime::AppRuntime;
use kriger_common::{messaging, models};
use std::ops::DerefMut;
use std::time::Duration;
use tokio::select;
use tokio::time::{Instant, MissedTickBehavior};
use tracing::{debug, error, info, instrument, warn};

pub async fn main(runtime: AppRuntime) -> eyre::Result<()> {
    info!("starting submitter");

    let metrics = SubmitterMetrics::default();
    metrics.register(runtime.metrics_registry.write().await.deref_mut());

    let flags_bucket = runtime.messaging.flags();
    let flags_svc = Box::leak(Box::new(flags_bucket));
    // TODO FIXME Using Box::leak is ugly, avoid doing that

    let config: SubmitterConfig = runtime
        .config
        .submitter
        .clone()
        .try_into()
        .context("unable to parse the submitter config")?;

    let batch = config.batch.unwrap_or(10_000);
    let mut fetcher = flags_svc
        .submissions_fetcher(Some("submitter".to_string()), batch)
        .await
        .context("unable to create a fetcher for flag submissions")?;

    let submitter = config.inner.into_submitter();

    let mut interval = tokio::time::interval(Duration::from_secs(config.interval));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        select! {
            _ = interval.tick() => {}
            _ = runtime.cancellation_token.cancelled() => return Ok(())
        }

        debug!("submitter tick");
        let submissions = match fetcher.next().await {
            Ok(submissions) => submissions.filter_map(|maybe_submission| async {
                match maybe_submission {
                    Ok(submission) => Some(submission),
                    Err(MessagingServiceError::ProcessingError { message, error }) => {
                        error! {
                            ?error,
                            "messaging processing error"
                        }
                        _ = message.ack_with(AckKind::Term).await;
                        None
                    }
                    Err(error) => {
                        error! {
                            ?error,
                            "unexpected messaging error"
                        }
                        None
                    }
                }
            }),
            Err(error) => {
                error! {
                    ?error,
                    "unable to fetch flag submissions"
                }
                continue;
            }
        };
        let submissions = submissions.collect().await;
        handle_submit(submitter.as_ref(), &metrics, submissions, flags_svc).await;
    }
}
#[instrument(skip_all, fields(flag_count = %requests.len()))]
async fn handle_submit(
    submitter: &(dyn Submitter + Send + Sync),
    metrics: &SubmitterMetrics,
    requests: Vec<MessageWrapper<messaging::model::FlagSubmission>>,
    flags_svc: &FlagsService,
) {
    if requests.len() == 0 {
        debug!("there are no flags to submit, skipping");
        return;
    }

    info! {
        flags.count = requests.len(),
        "preparing to submit flags"
    }
    metrics.start.inc();
    metrics.flag_submissions.inc_by(requests.len() as u64);

    let flags: Vec<&str> = requests
        .iter()
        .map(|msg| msg.payload.flag.as_ref())
        .collect();

    // Indicate the work is in progress. We do this concurrently.
    // FIXME: Indicating in-progress will only increase the ongoing period by another AckWait, maybe increase this periodically?
    let progress_futures = requests.iter().map(|req| req.progress());
    join_all(progress_futures).await;

    let start = Instant::now();
    let res = submitter.submit(&flags).await;
    let elapsed = start.elapsed();
    metrics
        .duration
        .observe(elapsed.as_micros() as f64 / 1_000_000.0);

    match res {
        Ok(mut results) => {
            metrics.complete.inc();

            for (_, status) in &results {
                metrics
                    .flag_results
                    .get_or_create(&FlagSubmissionStatusLabels {
                        status: status.into(),
                    })
                    .inc();
            }

            let futures = requests.into_iter().map(|message| {
                handle_result(
                    flags_svc,
                    results.remove(message.payload.flag.as_str()),
                    message,
                )
            });
            join_all(futures).await;
        }
        Err(error) => {
            error! {
                ?error,
                "unable to submit flags"
            }
            metrics.error.inc();

            let backoff = Some(Duration::from_secs(2));
            let futures = requests.iter().map(|message| message.nak(backoff));
            join_all(futures).await;
        }
    }
}

#[instrument(
    level = "DEBUG",
    skip_all,
    fields(
        flag = %message.payload.flag,
        flag.status = ?maybe_status
    )
)]
async fn handle_result(
    flags_svc: &FlagsService,
    maybe_status: Option<models::FlagSubmissionStatus>,
    message: MessageWrapper<messaging::model::FlagSubmission>,
) {
    let nak_backoff = Some(Duration::from_secs(2));

    let payload = &message.payload;
    let status = match maybe_status {
        Some(status) => status,
        None => {
            // How?
            warn! {
                flag = %payload.flag,
                "submitted flag did not receive a response",
            }
            let _ = message.nak(nak_backoff).await;
            return;
        }
    };

    debug!("received flag submission response");
    let should_retry = status.should_retry();
    let result = messaging::model::FlagSubmissionResult {
        flag: payload.flag.clone(),
        team_id: payload.team_id.clone(),
        service: payload.service.clone(),
        exploit: payload.exploit.clone(),
        status,
        points: None,
    };

    debug!("flag submission result: {result:?}");

    if let Err(error) = flags_svc.submit_flag_result(&result).await {
        warn! {
            ?error,
            "unable to publish flag submission result"
        }
        let _ = message.nak(nak_backoff).await;
        return;
    }

    if should_retry {
        let _ = message.nak(nak_backoff).await;
    } else {
        let _ = message.ack().await;
    }
}
