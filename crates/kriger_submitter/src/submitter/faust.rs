use super::{FormatErrorKind, SubmitError, Submitter};
use async_trait::async_trait;
use kriger_common::models;
use std::collections::HashMap;
use std::ops::DerefMut;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufStream};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tracing::{debug, instrument, warn};

pub(crate) struct FaustSubmitter {
    host: String,
    stream: RwLock<Option<BufStream<TcpStream>>>,
}

#[async_trait]
impl Submitter for FaustSubmitter {
    #[instrument(skip_all, fields(flag_count = flags.len()))]
    async fn submit(
        &self,
        flags: &[&str],
    ) -> Result<HashMap<String, models::FlagSubmissionStatus>, SubmitError> {
        let mut stream_ref = self.stream.write().await;

        // TODO: Improve this mess?
        let mut stream = match stream_ref.deref_mut() {
            Some(stream) => stream,
            opt => {
                let stream = self.create_connection().await?;
                opt.insert(stream)
            }
        };

        match submit_internal(&mut stream, flags).await {
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

impl FaustSubmitter {
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

        // https://ctf-gameserver.org/submission/
        // The server MUST indicate that the welcome sequence has finished by sending two subsequent newlines (\n\n).
        while !header.ends_with(b"\n\n") {
            socket.read_until(b'\n', &mut header).await?;
        }

        Ok(socket)
    }
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

        // The server's response MUST consist of:
        // - A repetition of the submitted flag (1)
        // - Whitespace
        // - One of the response codes defined below (2)
        // - Optionally: Whitespace, followed by a custom message consisting of any characters except newlines (3)
        // - Newline
        let mut split = response.trim().splitn(3, ' ');

        // TODO: Do we want to localize the failure to a single flag submission only?
        let flag = split
            .next()
            .ok_or_else(|| SubmitError::FormatError(FormatErrorKind::MissingField("flag")))?;
        let code = split
            .next()
            .ok_or_else(|| SubmitError::FormatError(FormatErrorKind::MissingField("code")))?;

        let status = map_status_code(code);
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
        "ERR" => models::FlagSubmissionStatus::Error,
        _ => models::FlagSubmissionStatus::Unknown,
    }
}
