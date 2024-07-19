use color_eyre::eyre;
use color_eyre::eyre::Context;
use futures::Stream;
use kriger_common::messaging::model::{FlagSubmission, FlagSubmissionResult, FlagSubmissionStatus};
use kriger_common::messaging::Message;
use rand::Rng;
use std::pin::Pin;

use super::{Submitter, SubmitterCallback};

#[derive(Clone, Debug)]
pub struct DummySubmitter {}

impl Submitter for DummySubmitter {
    async fn run(
        &self,
        flags: Pin<
            Box<dyn Stream<Item = (impl Message<Payload = FlagSubmission> + 'static)> + Send>,
        >,
        callback: impl SubmitterCallback + Send + Sync + 'static,
    ) -> eyre::Result<()> {
        // pin!(flags);
        // while let Some(msg) = flags.next().await {
        //     if let Err(err) = self.handle_flag(&msg, &callback).await {
        //         let _ = msg.nak();
        //         warn!("unable to handle flag: {err:?}");
        //     }
        // }
        Ok(())
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

        let result = FlagSubmissionResult {
            status,
            points: None,
        };
        // TODO: Extract flag from the key
        callback
            .submit("", result)
            .await
            .context("unable to save submission result")?;

        msg.ack().await.context("unable to ack")?;
        Ok(())
    }

    fn gen_submission_status(&self) -> FlagSubmissionStatus {
        let mut rng = rand::thread_rng();
        let r = rng.gen_range(0..=99);
        match r {
            0..=49 => FlagSubmissionStatus::Ok,
            50..=59 => FlagSubmissionStatus::Duplicate,
            60..=69 => FlagSubmissionStatus::Own,
            70..=79 => FlagSubmissionStatus::Old,
            80..=89 => FlagSubmissionStatus::Invalid,
            90..=99 => FlagSubmissionStatus::Error,
            _ => unreachable!(),
        }
    }
}
