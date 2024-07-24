mod submitter;
mod utils;

use crate::submitter::{Submitter, SubmitterCallback, SubmitterConfig, Submitters};
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use color_eyre::eyre;
use color_eyre::eyre::{Context, ContextCompat};
use futures::StreamExt;
use kriger_common::messaging::model::{CompetitionConfig, FlagSubmission, FlagSubmissionResult};
use kriger_common::messaging::{AckPolicy, Bucket, DeliverPolicy, Messaging, MessagingError};
use kriger_common::runtime::AppRuntime;
use std::time::Duration;
use tracing::{debug, info, instrument, warn};

struct SubmitterCallbackImpl<B: Bucket + Send + Sync> {
    // NOTE this ought to be an Arc or something once the Box::leak is removed
    bucket: &'static B,
}

impl<B: Bucket + Send + Sync> SubmitterCallback for SubmitterCallbackImpl<B> {
    async fn submit(&self, flag: &str, result: FlagSubmissionResult) -> Result<(), MessagingError> {
        debug!("flag submission result: {result:?}");

        let flag_b64 = STANDARD_NO_PAD.encode(flag);
        let key = format!("{}.result", &flag_b64);
        self.bucket.put(&key, &result).await?;
        Ok(())
    }
}

#[instrument(skip_all)]
pub async fn main(runtime: AppRuntime) -> eyre::Result<()> {
    info!("starting submitter");

    let config_bucket = runtime
        .messaging
        .config()
        .await
        .context("unable to retrieve the config bucket")?;

    // TODO: Provide a more elegant way to retrieve this and add support for live reload
    let competition_config = config_bucket
        .get::<CompetitionConfig>("competition")
        .await
        .context("unable to retrieve the competition config")?
        .context("the competition config does not exist")?;

    let flags_bucket = runtime
        .messaging
        .flags()
        .await
        .context("unable to retrieve the flags bucket")?;
    let flags_bucket = Box::leak(Box::new(flags_bucket));
    // TODO FIXME Using Box::leak is ugly, avoid doing that

    let flag_submissions = flags_bucket
        .watch_key::<FlagSubmission>(
            "*.submit",
            Some("submitter".to_string()),
            AckPolicy::Explicit,
            DeliverPolicy::New,
            // TODO: Un-hardcode
            vec![
                Duration::from_secs(1),
                Duration::from_secs(3),
                Duration::from_secs(5),
                Duration::from_secs(10),
                Duration::from_secs(30),
                Duration::from_secs(60),
                Duration::from_secs(90),
                Duration::from_secs(120),
            ],
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
        });

    let callback = SubmitterCallbackImpl {
        bucket: flags_bucket,
    };

    let config: SubmitterConfig = serde_json::from_value(competition_config.submitter)
        .context("unable to parse the flag submitter config")?;

    match config.into_submitter() {
        Submitters::Dummy(submitter) => {
            submitter
                .run(
                    flag_submissions.boxed(),
                    callback,
                    runtime.cancellation_token,
                )
                .await?;
        }
        Submitters::Cini(submitter) => {
            submitter
                .run(
                    flag_submissions.boxed(),
                    callback,
                    runtime.cancellation_token,
                )
                .await?;
        }
    }
    Ok(())
}
