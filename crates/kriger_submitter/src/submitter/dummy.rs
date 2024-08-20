use super::{SubmitError, Submitter};
use async_trait::async_trait;
use kriger_common::models;
use rand::Rng;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct DummySubmitter;

#[async_trait]
impl Submitter for DummySubmitter {
    async fn submit(
        &self,
        flags: &[&str],
    ) -> Result<HashMap<String, models::FlagSubmissionStatus>, SubmitError> {
        Ok(flags
            .into_iter()
            .map(|&flag| (flag.to_owned(), gen_submission_status()))
            .collect())
    }
}

fn gen_submission_status() -> models::FlagSubmissionStatus {
    let mut rng = rand::thread_rng();
    let r = rng.gen_range(0..=99);
    match r {
        0..=69 => models::FlagSubmissionStatus::Ok,
        70..=74 => models::FlagSubmissionStatus::Duplicate,
        75..=79 => models::FlagSubmissionStatus::Own,
        80..=84 => models::FlagSubmissionStatus::Old,
        85..=94 => models::FlagSubmissionStatus::Invalid,
        95..=99 => models::FlagSubmissionStatus::Error,
        _ => unreachable!(),
    }
}
