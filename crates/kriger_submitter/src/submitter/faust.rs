use std::{str::FromStr, time::Instant};

use super::{SubmitError, Submitter};
use kriger_common::messaging::model::FlagSubmissionStatus;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tracing::{debug, warn};

#[derive(Clone, Debug)]
pub struct FaustSubmitter {
    host: String,
}

impl FaustSubmitter {
    pub fn new(host: String) -> Self {
        Self { host }
    }
}

impl Submitter for FaustSubmitter {
    async fn submit(
        &self,
        flags: Vec<String>,
    ) -> Result<Vec<(String, FlagSubmissionStatus)>, SubmitError> {
        if flags.is_empty() {
            return Ok(Vec::new());
        }

        let inst = Instant::now();

        let socket = tokio::net::TcpStream::connect(&self.host).await?;

        // bufread over it
        let mut socket = tokio::io::BufStream::new(socket);

        debug!("Opened socket.");

        // read header
        let mut header: Vec<u8> = Vec::new();

        // https://ctf-gameserver.org/submission/
        while !header.ends_with(b"\n\n") {
            socket.read_until(b'\n', &mut header).await?;
        }
        debug!("Header read.");

        // send all flags
        let all_flags = flags.join("\n") + "\n";
        socket.write_all(all_flags.as_bytes()).await?;
        socket.flush().await?;

        // read all data
        let response = {
            let mut total_text = String::new();

            loop {
                if total_text.trim().lines().count() == flags.len() {
                    debug!("Got all {} flags, so stopping.", flags.len());
                    break;
                }

                match socket.read_line(&mut total_text).await {
                    Ok(n) => {
                        if n == 0 {
                            // TODO return here saying try to resubmit
                            warn!("EOF when reading from socket");
                            break;
                        }
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }

            total_text
        };

        // extract responses
        let lines = {
            let body = response.trim();
            let lines = body.split('\n').collect::<Vec<_>>();

            if lines.len() != flags.len() {
                warn!(
                    "Got {} lines, but expected {}. Content {}",
                    lines.len(),
                    flags.len(),
                    response
                );
                return Err(SubmitError::FormatError);
            }

            lines
        };

        let mut statuses = Vec::new();
        for line in lines {
            // split twice on space to get 3 variables
            let (flag, rest) = line.split_once(' ').ok_or(SubmitError::FormatError)?;

            // msg is optional
            let (code, _msg) = {
                match rest.split_once(' ') {
                    Some((code, msg)) => (code, Some(msg)),
                    None => (rest, None),
                }
            };

            let status = match code {
                "OK" => FlagSubmissionStatus::Ok,
                "DUP" => FlagSubmissionStatus::Duplicate,
                "OWN" => FlagSubmissionStatus::Own,
                "OLD" => FlagSubmissionStatus::Old,
                "INV" => FlagSubmissionStatus::Invalid,
                "ERR" => FlagSubmissionStatus::Error,
                code => FlagSubmissionStatus::Unknown(code.to_string()),
            };

            if let FlagSubmissionStatus::Unknown(..) = status {
                warn!(
                    "Unknown flag status: {} for flag {}, putting ERR",
                    code, flag
                );
            }

            statuses.push((flag.to_string(), status));
        }

        let elapsed = inst.elapsed();

        debug!(
            "Submitted {} flags in {}ms",
            flags.len(),
            elapsed.as_millis()
        );

        Ok(statuses)
    }
}
