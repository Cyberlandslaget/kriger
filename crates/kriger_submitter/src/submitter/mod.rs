use color_eyre::{eyre::eyre, Report};
use thiserror::Error;

// implementations
mod dctf;
pub use dctf::DctfSubmitter;
mod faust;
pub use faust::FaustSubmitter;
mod dummy;
pub use dummy::DummySubmitter;
use kriger_common::messaging::model::FlagSubmissionStatus;

#[derive(Debug)]
pub enum Submitters {
    Dummy(DummySubmitter),
    Faust(FaustSubmitter),
    Dctf(DctfSubmitter),
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

/// Implements the low-level operation of submitting a bunch of flags
pub trait Submitter {
    async fn submit(
        &self,
        flags: Vec<String>,
    ) -> Result<Vec<(String, FlagSubmissionStatus)>, SubmitError>;
}

// Deserialize
impl Submitters {
    /*
    pub fn from_conf(manager: &config::Manager) -> Result<Self, Report> {
        match manager.submitter_name.as_str() {
            "dummy" => Ok(Self::Dummy(DummySubmitter {})),
            "faust" => {
                let host = manager
                    .submitter
                    .get("host")
                    .ok_or(eyre!("Faust submitter requires host"))?
                    .as_str()
                    .ok_or(eyre!("Faust submitter host must be a string"))?
                    .to_owned();

                let faust = FaustSubmitter::new(host);

                Ok(Self::Faust(faust))
            }
            "dctf" => {
                let url = manager
                    .submitter
                    .get("url")
                    .ok_or(eyre!("DCTF submitter requires url"))?
                    .as_str()
                    .ok_or(eyre!("DCTF submitter url must be a string"))?
                    .to_owned();

                let cookie = manager
                    .submitter
                    .get("cookie")
                    .ok_or(eyre!("DCTF submitter requires cookie"))?
                    .as_str()
                    .ok_or(eyre!("DCTF submitter cookie must be a string"))?
                    .to_owned();

                let dctf = DctfSubmitter::new(url, cookie);

                Ok(Self::Dctf(dctf))
            }
            _ => Err(eyre!("Unknown submitter {}", manager.submitter_name)),
        }
    }*/
}
