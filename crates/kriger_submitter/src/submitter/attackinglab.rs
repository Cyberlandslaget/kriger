// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use super::{FormatErrorKind, SubmitError, Submitter};
use async_trait::async_trait;
use kriger_common::models;
use std::collections::HashMap;
use std::ops::DerefMut;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufStream};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tracing::{debug, instrument, warn};

pub(crate) struct AttackingLabSubmitter {
    host: String,
    stream: RwLock<Option<BufStream<TcpStream>>>,
}

#[async_trait]
impl Submitter for AttackingLabSubmitter {
    #[instrument(skip_all, fields(flag_count = flags.len()))]
    async fn submit(
        &self,
        flags: &[&str],
    ) -> Result<HashMap<String, models::FlagSubmissionStatus>, SubmitError> {
        let mut stream_ref = self.stream.write().await;

        // TODO: Improve this mess?
        let stream = match stream_ref.deref_mut() {
            Some(stream) => stream,
            opt => {
                let stream = self.create_connection().await?;
                opt.insert(stream)
            }
        };

        match submit_internal(stream, flags).await {
            Ok(map) => Ok(map),
            Err(error) => {
                // Drop the connection and propagate the error. This should trigger a retry and a
                // new connection will be created on the next attempt.
                stream_ref.deref_mut().take();
                Err(error)
            }
        }
    }
}

impl AttackingLabSubmitter {
    pub fn new(host: String) -> Self {
        Self {
            host,
            stream: RwLock::new(None),
        }
    }

    #[instrument(skip_all)]
    async fn create_connection(&self) -> Result<BufStream<TcpStream>, SubmitError> {
        debug!("created a new connection");
        let socket = TcpStream::connect(&self.host).await?;
        let socket = BufStream::new(socket);

        // TODO: Find out if there is any headers?
        // let mut header: Vec<u8> = Vec::new();
        // The server MUST indicate that the welcome sequence has finished by sending two subsequent newlines (\n\n).
        // while !header.ends_with(b"\n\n") {
        //     socket.read_until(b'\n', &mut header).await?;
        // }

        Ok(socket)
    }
}

fn parse_flag_response(response: String) -> Result<(String, String), SubmitError> {
    // The server's response MUST consist:
    // - One of the response codes defined below (2)
    // - Newline
    let mut split = response.trim().splitn(2, ' ');

    // TODO: Do we want to localize the failure to a single flag submission only?
    let status = split
        .next()
        .ok_or_else(|| SubmitError::FormatError(FormatErrorKind::MissingField("status")))?;

    if status.is_empty() {
        return Err(SubmitError::FormatError(FormatErrorKind::MissingField(
            "status",
        )));
    }

    let message = split.next().unwrap_or("");

    return Ok((status.to_string(), message.to_string()));
}

async fn submit_internal(
    stream: &mut BufStream<TcpStream>,
    flags: &[&str],
) -> Result<HashMap<String, models::FlagSubmissionStatus>, SubmitError> {
    for &flag in flags {
        // - To submit a flag, the client MUST send the flag followed by a single newline.
        // - During a single connection, the client MAY submit an arbitrary number of flags.
        // - The client MAY send flags without waiting for the welcome sequence or responses to previously submitted flags.
        stream.write_all(flag.as_bytes()).await?;
        stream.write_u8(b'\n').await?;
    }
    stream.flush().await?;

    let mut map = HashMap::with_capacity(flags.len());

    // We expect |flags| responses from the submission server
    for i in 0..flags.len() {
        let mut response = String::new();
        match stream.read_line(&mut response).await {
            Ok(n) => {
                if n == 0 {
                    return Err(SubmitError::FormatError(FormatErrorKind::EOF));
                }
            }
            Err(err) => return Err(err.into()),
        }

        let (status_msg, msg) = parse_flag_response(response)?;

        let status = map_status_code(status_msg.as_str(), msg.as_str());
        if let models::FlagSubmissionStatus::Unknown = status {
            warn! {
                status_msg,
                msg,
                "received unknown status from the submission api",
            }
        }

        // map.insert(flag.to_owned(), status);
        map.insert(flags[i].to_string(), status);
    }

    Ok(map)
}

fn map_status_code(code: &str, msg: &str) -> models::FlagSubmissionStatus {
    match code {
        "[OK]" => models::FlagSubmissionStatus::Ok,
        "[ERR]" => match msg {
            "Invalid format" => models::FlagSubmissionStatus::Invalid,
            "Invalid flag" => models::FlagSubmissionStatus::Invalid,
            "Expired" => models::FlagSubmissionStatus::Old,
            "Already submitted" => models::FlagSubmissionStatus::Duplicate,
            "Can't submit flag from NOP team" => models::FlagSubmissionStatus::Nop,
            "This is your own flag" => models::FlagSubmissionStatus::Own,
            _ => models::FlagSubmissionStatus::Unknown,
        },
        "[OFFLINE]" => models::FlagSubmissionStatus::Error,
        _ => models::FlagSubmissionStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_respond_with_ok_when_valid_flag() {
        let (code, msg) = parse_flag_response("[OK]\n".to_string()).unwrap();

        let status_code = map_status_code(&code, &msg);
        assert_eq!(code, "[OK]".to_string());
        assert_eq!(msg, "".to_string());
        assert_eq!(status_code, models::FlagSubmissionStatus::Ok);
    }

    #[test]
    fn should_respond_with_dup_when_duplicate_flag() {
        let (code, msg) = parse_flag_response("[ERR] Already submitted\n".to_string()).unwrap();

        let status_code = map_status_code(&code, &msg);
        assert_eq!(code, "[ERR]".to_string());
        assert_eq!(msg, "Already submitted".to_string());
        assert_eq!(status_code, models::FlagSubmissionStatus::Duplicate);
    }

    #[test]
    fn should_respond_with_own_when_own_flag() {
        let (code, msg) = parse_flag_response("[ERR] This is your own flag\n".to_string()).unwrap();

        let status_code = map_status_code(&code, &msg);
        assert_eq!(code, "[ERR]".to_string());
        assert_eq!(msg, "This is your own flag".to_string());
        assert_eq!(status_code, models::FlagSubmissionStatus::Own);
    }

    #[test]
    fn should_respond_with_inv_when_invalid_format() {
        let (code, msg) = parse_flag_response("[ERR] Invalid format\n".to_string()).unwrap();

        let status_code = map_status_code(&code, &msg);
        assert_eq!(code, "[ERR]".to_string());
        assert_eq!(msg, "Invalid format".to_string());
        assert_eq!(status_code, models::FlagSubmissionStatus::Invalid);
    }

    #[test]
    fn should_respond_with_inv_when_invalid_flag() {
        let (code, msg) = parse_flag_response("[ERR] Invalid flag\n".to_string()).unwrap();

        let status_code = map_status_code(&code, &msg);
        assert_eq!(code, "[ERR]".to_string());
        assert_eq!(msg, "Invalid flag".to_string());
        assert_eq!(status_code, models::FlagSubmissionStatus::Invalid);
    }

    #[test]
    fn should_return_error_when_only_newline_in_response() {
        let response = parse_flag_response("\n".to_string());

        assert!(response.is_err());
    }
}

