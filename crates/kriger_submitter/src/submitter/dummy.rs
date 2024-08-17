use async_trait::async_trait;
use kriger_common::messaging::model::FlagSubmissionStatus;
use rand::Rng;
use std::collections::HashMap;

use super::{SubmitError, Submitter};

#[derive(Clone, Debug)]
pub struct DummySubmitter;

#[async_trait]
impl Submitter for DummySubmitter {
    async fn submit(
        &self,
        flags: &[&str],
    ) -> Result<HashMap<String, FlagSubmissionStatus>, SubmitError> {
        Ok(flags
            .into_iter()
            .map(|&flag| (flag.to_owned(), gen_submission_status()))
            .collect())
    }
}

fn gen_submission_status() -> FlagSubmissionStatus {
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
