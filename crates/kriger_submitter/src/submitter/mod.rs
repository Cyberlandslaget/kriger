use async_trait::async_trait;
use color_eyre::eyre;
use futures::Stream;
use kriger_common::messaging::model::{FlagSubmission, FlagSubmissionResult};
use kriger_common::messaging::{Message, MessagingError};
use serde::Deserialize;
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

// TODO: Port
//mod dctf;
mod cini;
mod dummy;
mod faust;

// TODO: Devise a more ergonomic way to deal with this.
pub(crate) trait SubmitterCallback {
    fn submit(
        &self,
        flag: &str,
        result: FlagSubmissionResult,
    ) -> impl Future<Output = Result<(), MessagingError>> + Send;
}

// Workaround for non-object traits
// FIXME: Are there workarounds..?
pub(crate) enum Submitters {
    Dummy(dummy::DummySubmitter),
    Cini(cini::CiniSubmitter),
    Faust(faust::FaustSubmitter),
}

/// The submitter will be responsible for handling the flag submission lifecycle with the given
/// [flags] stream and the [callback].
#[async_trait]
pub(crate) trait Submitter {
    async fn run(
        &self,
        flags: Pin<
            Box<
                dyn Stream<Item = (impl Message<Payload = FlagSubmission> + Sync + 'static)> + Send,
            >,
        >,
        callback: impl SubmitterCallback + Send + Sync + 'static,
        cancellation_token: CancellationToken,
    ) -> eyre::Result<()>;
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SubmitterConfig {
    Dummy,
    Cini {
        /// The URL of the flag submission endpoint.
        url: String,
        /// The interval that the submitter should submit flags at.
        interval: u64,
        /// The batch size of each submission request.
        batch: usize,
        /// The team token used to authenticate with the flag submission API.
        token: String,
    },
    Faust {
        host: String,
    },
}

impl SubmitterConfig {
    pub(crate) fn into_submitter(self) -> Submitters {
        match self {
            SubmitterConfig::Dummy => Submitters::Dummy(dummy::DummySubmitter {}),
            SubmitterConfig::Cini {
                url,
                interval,
                batch,
                token,
            } => Submitters::Cini(cini::CiniSubmitter::new(url, interval, batch, token)),
            SubmitterConfig::Faust { host } => Submitters::Faust(faust::FaustSubmitter { host }),
        }
    }
}

/// Did not manage to submit
#[derive(Error, Debug)]
pub enum SubmitError {
    #[error("network error")]
    NetworkError(#[from] std::io::Error),
    #[error("format error")]
    /// The format of the response was not as expected
    FormatError,
    #[error("serde")]
    SerdeJson(#[from] serde_json::Error),
    #[error("reqwest")]
    Reqwest(#[from] reqwest::Error),
    #[error("unknown error: {0}")]
    Unknown(&'static str),
}
