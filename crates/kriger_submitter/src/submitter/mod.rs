use async_trait::async_trait;
use kriger_common::models;
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

// TODO: Port
//mod dctf;
mod cini;
mod dummy;
mod faust;

/// The submitter will be responsible for submitting the flags in bulk. See ADR-002.
#[async_trait]
pub(crate) trait Submitter {
    /// Submits a slice of flags and returns a map of flags associated with their respective
    /// results.
    ///
    /// This operation is not cancel safe.
    async fn submit(
        &self,
        flags: &[&str],
    ) -> Result<HashMap<String, models::FlagSubmissionStatus>, SubmitError>;
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct SubmitterConfig {
    /// The interval that the submitter should submit flags at.
    pub(crate) interval: u64,

    /// The maximum batch size for flag submissions.
    pub(crate) batch: Option<usize>,

    #[serde(flatten)]
    pub(crate) inner: InnerSubmitterConfig,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InnerSubmitterConfig {
    Dummy,
    Cini {
        /// The URL of the flag submission endpoint.
        url: String,
        /// The team token used to authenticate with the flag submission API.
        token: String,
    },
    Faust {
        host: String,
    },
}

impl InnerSubmitterConfig {
    pub(crate) fn into_submitter(self) -> Box<dyn Submitter + Send + Sync> {
        match self {
            InnerSubmitterConfig::Dummy => Box::new(dummy::DummySubmitter),
            InnerSubmitterConfig::Cini { url, token } => {
                Box::new(cini::CiniSubmitter::new(url, token))
            }
            InnerSubmitterConfig::Faust { host } => Box::new(faust::FaustSubmitter::new(host)),
        }
    }
}

/// Did not manage to submit
#[derive(Error, Debug)]
pub enum SubmitError {
    #[error("network error")]
    NetworkError(#[from] std::io::Error),
    #[error("format error: {0}")]
    FormatError(FormatErrorKind),
    #[error("serde")]
    SerdeJson(#[from] serde_json::Error),
    #[error("reqwest")]
    Reqwest(#[from] reqwest::Error),
    #[error("unknown error: {0}")]
    Unknown(&'static str),
}

#[derive(Error, Debug)]
pub enum FormatErrorKind {
    #[error("eof reached")]
    EOF,
    #[error("missing data")]
    MissingData,
    #[error("missing field `{0}`")]
    MissingField(&'static str),
    #[error("error response")]
    ErrorResponse,
    #[error("unknown")]
    Unknown,
}
