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

pub(crate) struct EnowarsSubmitter {
    host: String,
    stream: RwLock<Option<BufStream<TcpStream>>>,
}

#[async_trait]
impl Submitter for EnowarsSubmitter {
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

impl EnowarsSubmitter {
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
        let mut socket = BufStream::new(socket);
        let mut header: Vec<u8> = Vec::new();

        // The server MUST indicate that the welcome sequence has finished by sending two subsequent newlines (\n\n).
        while !header.ends_with(b"\n\n") {
            socket.read_until(b'\n', &mut header).await?;
        }

        Ok(socket)
    }
}

fn parse_flag_response(response: String) -> Result<(String, String), SubmitError> {
        // The server's response MUST consist:
        // - A repetition of the submitted flag (1)
        // - Whitespace
        // - One of the response codes defined below (2)
        // - Newline
        let mut split = response.trim().splitn(2, ' ');

        // TODO: Do we want to localize the failure to a single flag submission only?
        let flag = split
            .next()
            .ok_or_else(|| SubmitError::FormatError(FormatErrorKind::MissingField("flag")))?;
        let code = split
            .next()
            .ok_or_else(|| SubmitError::FormatError(FormatErrorKind::MissingField("code")))?;

        return Ok((flag.to_string(), code.to_string()))
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
    for _ in 0..flags.len() {
        let mut response = String::new();
        match stream.read_line(&mut response).await {
            Ok(n) => {
                if n == 0 {
                    return Err(SubmitError::FormatError(FormatErrorKind::EOF));
                }
            }
            Err(err) => return Err(err.into()),
        }

        let (flag, code) = parse_flag_response(response)?;


        let status = map_status_code(code.as_str());
        if let models::FlagSubmissionStatus::Unknown = status {
            warn! {
                code,
                flag,
                "received unknown status from the submission api",
            }
        }

        map.insert(flag.to_owned(), status);
    }

    Ok(map)
}


fn map_status_code(code: &str) -> models::FlagSubmissionStatus {
    match code {
        "OK" => models::FlagSubmissionStatus::Ok,
        "DUP" => models::FlagSubmissionStatus::Duplicate,
        "OWN" => models::FlagSubmissionStatus::Own,
        "OLD" => models::FlagSubmissionStatus::Old,
        "INV" => models::FlagSubmissionStatus::Invalid,
        _ => models::FlagSubmissionStatus::Unknown,
    }
}

#[test]
fn should_respond_with_ok_when_valid_flag() {
    let (flag, code) = parse_flag_response("ENO736a6b6473616a6b647361736a6b64736b646a736b6b6b6b OK\n".to_string()).unwrap();

    let status_code = map_status_code(&code);
    assert_eq!(flag, "ENO736a6b6473616a6b647361736a6b64736b646a736b6b6b6b".to_string());
    assert_eq!(code, "OK".to_string());
    assert_eq!(status_code, models::FlagSubmissionStatus::Ok);
}

#[test]
fn should_respond_with_dup_when_duplicate_flag() {
    let (flag, code) = parse_flag_response("ENO727577716b726a6c776b6a6c6b66736a61666b6c73616b6b DUP\n".to_string()).unwrap();

    let status_code = map_status_code(&code);
    assert_eq!(flag, "ENO727577716b726a6c776b6a6c6b66736a61666b6c73616b6b".to_string());
    assert_eq!(code, "DUP".to_string());
    assert_eq!(status_code, models::FlagSubmissionStatus::Duplicate);
}

#[test]
fn should_respond_with_own_when_own_flag() {
    let (flag, code) = parse_flag_response("ENO6e6576657220676f6e6e61206769766520796f752075702d OWN\n".to_string()).unwrap();

    let status_code = map_status_code(&code);
    assert_eq!(flag, "ENO6e6576657220676f6e6e61206769766520796f752075702d".to_string());
    assert_eq!(code, "OWN".to_string());
    assert_eq!(status_code, models::FlagSubmissionStatus::Own);
}

#[test]
fn should_respond_with_inv_when_invalid_flag() {
    let (flag, code) = parse_flag_response("ENO746869736973686578636f64655f666f7274657374696e67 INV\n".to_string()).unwrap();

    let status_code = map_status_code(&code);
    assert_eq!(flag, "ENO746869736973686578636f64655f666f7274657374696e67".to_string());
    assert_eq!(code, "INV".to_string());
    assert_eq!(status_code, models::FlagSubmissionStatus::Invalid);
}

#[test]
fn should_return_error_when_no_spaces_in_response() {
    let response = parse_flag_response("INVALID_RESPONSE\n".to_string());

    assert!(response.is_err());
}

#[test]
fn should_return_error_when_only_newline_in_response() {
    let response = parse_flag_response("\n".to_string());

    assert!(response.is_err());
}

