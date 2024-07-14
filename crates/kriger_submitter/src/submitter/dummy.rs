use kriger_common::messaging::model::FlagSubmissionStatus;
use rand::Rng;

use super::{SubmitError, Submitter};

#[derive(Clone, Debug)]
pub struct DummySubmitter {}

impl Submitter for DummySubmitter {
    async fn submit(
        &self,
        flags: Vec<String>,
    ) -> Result<Vec<(String, FlagSubmissionStatus)>, SubmitError> {
        let statuses = flags
            .into_iter()
            .map(|flag| {
                let mut rng = rand::thread_rng();
                let r = rng.gen_range(0..=99);
                match r {
                    0..=49 => (flag, FlagSubmissionStatus::Ok),
                    50..=59 => (flag, FlagSubmissionStatus::Duplicate),
                    60..=69 => (flag, FlagSubmissionStatus::Own),
                    70..=79 => (flag, FlagSubmissionStatus::Old),
                    80..=89 => (flag, FlagSubmissionStatus::Invalid),
                    90..=99 => (flag, FlagSubmissionStatus::Error),
                    _ => unreachable!(),
                }
            })
            .collect();
        Ok(statuses)
    }
}
