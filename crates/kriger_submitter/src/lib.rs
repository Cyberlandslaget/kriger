mod submitter;

use crate::submitter::{Submitter, SubmitterCallback, SubmitterConfig};
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use color_eyre::eyre;
use color_eyre::eyre::{Context, ContextCompat};
use futures::StreamExt;
use kriger_common::messaging::model::{CompetitionConfig, FlagSubmission, FlagSubmissionResult};
use kriger_common::messaging::{AckPolicy, Bucket, DeliverPolicy, Messaging, MessagingError};
use kriger_common::runtime::AppRuntime;
use std::sync::Arc;
use tracing::{info, warn};

struct SubmitterCallbackImpl<B: Bucket + Send + Sync> {
    bucket: Arc<B>,
}

impl<B: Bucket + Send + Sync> SubmitterCallback for SubmitterCallbackImpl<B> {
    async fn submit(&self, flag: &str, result: FlagSubmissionResult) -> Result<(), MessagingError> {
        let flag_b64 = STANDARD_NO_PAD.encode(flag);
        let key = format!("{}.result", &flag_b64);
        self.bucket.put(&key, &result).await?;
        Ok(())
    }
}

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
    let flags_bucket = Arc::new(flags_bucket);

    let flag_submissions = flags_bucket
        .watch_key::<FlagSubmission>(
            "*.submit",
            Some("submitter".to_string()),
            AckPolicy::Explicit,
            DeliverPolicy::New,
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
        bucket: flags_bucket.clone(),
    };

    let config: SubmitterConfig = serde_json::from_value(competition_config.submitter)
        .context("unable to parse the flag submitter config")?;

    let submitter = config.into_submitter();
    submitter.run(flag_submissions.boxed(), callback).await?;
    Ok(())
}
