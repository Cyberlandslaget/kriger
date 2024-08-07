use super::{SubmitError, Submitter, SubmitterCallback};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use kriger_common::messaging::model::{FlagSubmission, FlagSubmissionResult, FlagSubmissionStatus};
use kriger_common::messaging::Message;
use std::pin::Pin;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufStream};
use tokio::net::TcpStream;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{instrument, warn};

#[derive(Clone, Debug)]
pub(crate) struct FaustSubmitter {
    pub(crate) host: String,
}

impl FaustSubmitter {
    pub fn new(host: String) -> Self {
        Self { host }
    }

    #[instrument(skip_all)]
    async fn create_connection(&self) -> Result<BufStream<TcpStream>, SubmitError> {
        let socket = TcpStream::connect(&self.host).await?;
        // bufread over it
        let mut socket = BufStream::new(socket);
        let mut header: Vec<u8> = Vec::new();
        // https://ctf-gameserver.org/submission/
        while !header.ends_with(b"\n\n") {
            socket.read_until(b'\n', &mut header).await?;
        }

        Ok(socket)
    }

    #[instrument(skip_all, fields(flag))]
    async fn submit(
        &self,
        stream: &mut BufStream<TcpStream>,
        flag: String,
    ) -> Result<FlagSubmissionStatus, SubmitError> {
        // Send flags
        stream.write_all((flag + "\n").as_bytes()).await?;
        stream.flush().await?;

        // Read the data
        let mut response = String::new();
        match stream.read_line(&mut response).await {
            Ok(n) => {
                if n == 0 {
                    return Err(SubmitError::Unknown("reached eof"));
                }
            }
            Err(err) => return Err(err.into()),
        }

        // Parse the data
        // split twice on space to get 3 variables
        let (flag, rest) = response.trim().split_once(' ').ok_or(SubmitError::FormatError)?;

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
            _ => FlagSubmissionStatus::Unknown,
        };

        if let FlagSubmissionStatus::Unknown = status {
            warn! {
                code,
                flag,
                "received unknown status from the submission api",
            }
        }

        Ok(status)
    }
}

#[async_trait]
impl Submitter for FaustSubmitter {
    async fn run(
        &self,
        mut flags: Pin<
            Box<
                dyn Stream<Item = (impl Message<Payload = FlagSubmission> + Sync + 'static)> + Send,
            >,
        >,
        callback: impl SubmitterCallback + Send + Sync + 'static,
        cancellation_token: CancellationToken,
    ) -> color_eyre::Result<()> {
        // We return an error if the initial connection fails
        let mut stream = self.create_connection().await?;

        loop {
            select! {
                _ = cancellation_token.cancelled() => {
                    return Ok(())
                }
                maybe_message = flags.next() => {
                    if let Some(message) = maybe_message {
                        let payload = message.payload();
                        if let Err(error) = message.progress().await{
                            warn! {
                                ?error,
                                "unable to ack the message"
                            }
                        }

                        match self.submit(&mut stream, payload.flag.to_string()).await {
                            Ok(status) => {
                                let should_retry = status.should_retry();
                                let result = FlagSubmissionResult {
                                    flag: payload.flag.to_string(),
                                    team_id: payload.team_id.clone(),
                                    service: payload.service.clone(),
                                    exploit: payload.exploit.clone(),
                                    status,
                                    points: None,
                                };
                                match callback.submit(&payload.flag, result).await {
                                    Ok(_) => {
                                        if should_retry {
                                            if let Err(error) = message.nak().await {
                                                warn! {
                                                    ?error,
                                                    "unable to ack the message"
                                                }
                                            }
                                        } else {
                                            if let Err(error) = message.ack().await {
                                                warn! {
                                                    ?error,
                                                    "unable to ack the message"
                                                }
                                            }
                                        }
                                    }
                                    Err(error) => {
                                        warn! {
                                            ?error,
                                            "unable to send the submission message"
                                        }
                                        if let Err(error) = message.nak().await {
                                            warn! {
                                                ?error,
                                                "unable to ack the message"
                                            }
                                        }
                                    }
                                }
                            }
                            Err(error) => {
                                warn! {
                                    ?error,
                                    "unable to submit flag"
                                }
                                if let Err(error) = message.nak().await {
                                    warn! {
                                        ?error,
                                        "unable to ack the message"
                                    }
                                }

                                match self.create_connection().await {
                                    Ok(new_stream) => {
                                        stream = new_stream;
                                    }
                                    Err(error) => {
                                        warn! {
                                            ?error,
                                            "unable to create a new connection"
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        return Ok(())
                    }
                }
            }
        }
    }
}
