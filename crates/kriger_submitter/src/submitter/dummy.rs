use async_trait::async_trait;
use color_eyre::eyre;
use color_eyre::eyre::Context;
use futures::{Stream, StreamExt};
use kriger_common::messaging::model::{FlagSubmission, FlagSubmissionResult, FlagSubmissionStatus};
use kriger_common::messaging::Message;
use rand::Rng;
use std::pin::Pin;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::warn;

use super::{Submitter, SubmitterCallback};

#[derive(Clone, Debug)]
pub struct DummySubmitter {}

#[async_trait]
impl Submitter for DummySubmitter {
    async fn run(
        &self,
        mut flags: Pin<
            Box<
                dyn Stream<Item = (impl Message<Payload = FlagSubmission> + Sync + 'static)> + Send,
            >,
        >,
        callback: impl SubmitterCallback + Send + Sync + 'static,
        cancellation_token: CancellationToken,
    ) -> eyre::Result<()> {
        loop {
            select! {
                _ = cancellation_token.cancelled() => {
                    return Ok(());
                }
                res = flags.next() => {
                    match res {
                        Some(message) => {
                            if let Err(error) = self.handle_flag(&message, &callback).await {
                                let _ = message.nak().await;
                                warn! {
                                    ?error,
                                    flag = message.payload().flag,
                                    "unable to handle flag"
                                }
                            }
                        }
                        None => {
                            // End of stream
                        }
                    }
                }
            }
        }
    }
}

impl DummySubmitter {
    async fn handle_flag(
        &self,
        msg: &impl Message<Payload = FlagSubmission>,
        callback: &impl SubmitterCallback,
    ) -> eyre::Result<()> {
        msg.progress().await.context("unable to ack")?;
        let status = self.gen_submission_status();

        let payload = msg.payload();
        let result = FlagSubmissionResult {
            flag: payload.flag.to_string(),
            team_id: payload.team_id.clone(),
            service: payload.service.clone(),
            exploit: payload.exploit.clone(),
            status,
            points: None,
        };
        // TODO: Extract flag from the key
        callback
            .submit(&payload.flag, result)
            .await
            .context("unable to save submission result")?;

        msg.ack().await.context("unable to ack")?;
        Ok(())
    }

    fn gen_submission_status(&self) -> FlagSubmissionStatus {
        let mut rng = rand::thread_rng();
        let r = rng.gen_range(0..=99);
        match r {
            0..=69 => FlagSubmissionStatus::Ok,
            70..=74 => FlagSubmissionStatus::Duplicate,
            75..=79 => FlagSubmissionStatus::Own,
            80..=84 => FlagSubmissionStatus::Old,
            85..=94 => FlagSubmissionStatus::Invalid,
            95..=99 => FlagSubmissionStatus::Error,
            _ => unreachable!(),
        }
    }
}
