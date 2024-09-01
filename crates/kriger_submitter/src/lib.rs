mod submitter;
mod utils;

use crate::submitter::{Submitter, SubmitterConfig};
use crate::utils::futures::PollPending;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use color_eyre::eyre;
use color_eyre::eyre::Context;
use futures::future::join_all;
use futures::StreamExt;
use kriger_common::messaging::model::{FlagSubmission, FlagSubmissionResult};
use kriger_common::messaging::{AckPolicy, Bucket, DeliverPolicy, Message, Messaging};
use kriger_common::models;
use kriger_common::server::runtime::AppRuntime;
use std::time::Duration;
use tokio::select;
use tokio::time::{interval_at, Instant, MissedTickBehavior};
use tracing::{debug, error, info, instrument, warn};

pub async fn main(runtime: AppRuntime) -> eyre::Result<()> {
    info!("starting submitter");

    let flags_bucket = runtime
        .messaging
        .flags()
        .await
        .context("unable to retrieve the flags bucket")?;
    let flags_bucket = Box::leak(Box::new(flags_bucket));
    // TODO FIXME Using Box::leak is ugly, avoid doing that

    let mut flag_submissions = flags_bucket
        .watch_key::<FlagSubmission>(
            "*.submit",
            Some("submitter".to_string()),
            AckPolicy::Explicit,
            // TODO: Use approximately the same duration as flag submission timeout
            Duration::from_secs(60),
            DeliverPolicy::New,
            // TODO: Un-hardcode
            vec![], // TODO: Do something with this. AckWait is not supported when using consumer backoffs
                    // vec![
                    //     Duration::from_secs(1),
                    //     Duration::from_secs(3),
                    //     Duration::from_secs(5),
                    //     Duration::from_secs(10),
                    //     Duration::from_secs(30),
                    //     Duration::from_secs(60),
                    //     Duration::from_secs(90),
                    //     Duration::from_secs(120),
                    // ],
        )
        .await
        .context("unable to watch flag submissions")?
        .filter_map(|item| async move {
            match item {
                Ok(msg) => Some(msg),
                Err(err) => {
                    warn!("unable to parse flag submission: {err:?}");
                    None
                }
            }
        })
        .boxed(); // FIXME: Does actually have to be boxed?

    let config: SubmitterConfig = runtime
        .config
        .submitter
        .clone()
        .try_into()
        .context("unable to parse the submitter config")?;

    let submitter = config.inner.into_submitter();

    let mut interval = interval_at(Instant::now(), Duration::from_secs(config.interval));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    // The first tick completes immediately
    interval.tick().await;

    loop {
        debug!("submitter tick");
        select! {
            _ = runtime.cancellation_token.cancelled() => {
                return Ok(());
            }
            _ = interval.tick() => {}
        }

        // TODO: Is `PollPending` the best solution here, or should we just use the consumer's pending count?
        // TODO: Handle backpressure somehow
        let requests = PollPending::new(&mut flag_submissions, config.batch).await;
        handle_submit(submitter.as_ref(), requests, flags_bucket).await;
    }
}
#[instrument(skip_all, fields(flag_count = %requests.len()))]
async fn handle_submit(
    submitter: &(dyn Submitter + Send + Sync),
    requests: Vec<impl Message<Payload = FlagSubmission>>,
    flags_bucket: &impl Bucket,
) {
    if requests.len() == 0 {
        debug!("there are no flags to submit, skipping");
        return;
    }

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

    match submitter.submit(&flags).await {
        Ok(mut results) => {
            let futures = requests.into_iter().map(|message| {
                handle_result(
                    flags_bucket,
                    results.remove(message.payload().flag.as_str()),
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
            let backoff = Some(Duration::from_secs(2));
            let futures = requests.iter().map(|message| message.nak(backoff));
            join_all(futures).await;
        }
    }
}

#[instrument(skip_all, fields(
    flag = %message.payload().flag,
    flag.status = ?maybe_status
))]
async fn handle_result(
    flags_bucket: &impl Bucket,
    maybe_status: Option<models::FlagSubmissionStatus>,
    message: impl Message<Payload = FlagSubmission>,
) {
    let nak_backoff = Some(Duration::from_secs(2));

    let payload = message.payload();
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
    // FIXME: Can't we move??
    let should_retry = status.should_retry();
    let result = FlagSubmissionResult {
        flag: payload.flag.clone(),
        team_id: payload.team_id.clone(),
        service: payload.service.clone(),
        exploit: payload.exploit.clone(),
        status,
        points: None,
    };

    debug!("flag submission result: {result:?}");

    let flag_b64 = STANDARD_NO_PAD.encode(&payload.flag);
    let key = format!("{}.result", &flag_b64);

    if let Err(error) = flags_bucket.put(&key, &result).await {
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
