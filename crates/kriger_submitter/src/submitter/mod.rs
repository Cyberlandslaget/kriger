use color_eyre::eyre;
use futures::Stream;
use kriger_common::messaging::model::{FlagSubmission, FlagSubmissionResult};
use kriger_common::messaging::{Message, MessagingError};
use serde::Deserialize;
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;

// TODO: Port
//mod dctf;
//mod faust;
mod dummy;

// TODO: Devise a more ergonomic way to deal with this.
pub(crate) trait SubmitterCallback {
    fn submit(
        &self,
        flag: &str,
        result: FlagSubmissionResult,
    ) -> impl Future<Output = Result<(), MessagingError>> + Send;
}

/// The submitter will be responsible for handling the flag submission lifecycle with the given
/// [flags] stream and the [callback].
pub(crate) trait Submitter {
    fn run(
        &self,
        flags: Pin<
            Box<
                dyn Stream<Item = (impl Message<Payload = FlagSubmission> + Sync + 'static)> + Send,
            >,
        >,
        callback: impl SubmitterCallback + Send + Sync + 'static,
    ) -> impl Future<Output = eyre::Result<()>> + Send;
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SubmitterConfig {
    Dummy,
}

impl SubmitterConfig {
    pub(crate) fn into_submitter(self) -> impl Submitter + Send {
        match self {
            SubmitterConfig::Dummy => dummy::DummySubmitter {},
        }
    }
}

/// Did not manage to submit
#[derive(Error, Debug)]
pub enum SubmitError {
    #[error("Network error")]
    NetworkError(#[from] std::io::Error),
    #[error("Format error")]
    /// The format of the response was not as expected
    FormatError,
    #[error("serde")]
    SerdeJson(#[from] serde_json::Error),
    #[error("reqwest")]
    Reqwest(#[from] reqwest::Error),
}
