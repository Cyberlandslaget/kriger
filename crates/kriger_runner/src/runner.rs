use async_channel::Receiver;
use color_eyre::eyre::{Context, ContextCompat, Result};
use futures::stream::select_all;
use futures::StreamExt;
use kriger_common::messaging::model::ExecutionRequest;
use kriger_common::messaging::Message;
use regex::Regex;
use std::future::Future;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::select;
use tokio::sync::OwnedSemaphorePermit;
use tokio_stream::wrappers::LinesStream;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, instrument, warn};

#[derive(Clone)]
pub struct Runner {
    pub job_rx: Receiver<Job>,
    pub exploit_name: String,
    pub exploit_command: String,
    pub exploit_args: Vec<String>,
    pub flag_format: Regex,
    pub timeout: Duration,
}

pub struct Job {
    pub request: Box<dyn Message<Payload = ExecutionRequest> + Send>,
    pub _permit: OwnedSemaphorePermit,
}

const ENV_EXPLOIT_NAME: &'static str = "EXPLOIT";
const ENV_IP_ADDRESS: &'static str = "IP";
const ENV_FLAG_HINT: &'static str = "HINT";

enum OutputLine {
    Stdout(String),
    Stderr(String),
}

pub trait RunnerCallback {
    /// Called upon once the execution receives a flag. If this results in an error, the execution
    /// will be queued for retry.
    fn on_flag(&self, request: &ExecutionRequest, flag: &str) -> impl Future<Output = Result<()>>;
}

impl Runner {
    #[instrument(skip_all, fields(idx))]
    pub(crate) async fn worker(
        self,
        idx: usize,
        callback: impl RunnerCallback,
        token: CancellationToken,
    ) -> Result<()> {
        loop {
            select! {
                // Wait for the jobs to gracefully shut down
                _ = token.cancelled() => {
                    return Ok(());
                }
                res = self.job_rx.recv() => {
                    // The channel has most likely been closed
                    let job = res.context("unable to receive job")?;
                    if let Err(error) = self.handle_job(job, &callback).await {
                        error! {
                            ?error,
                            "unexpected job handling error"
                        }
                        // TODO: Consider delaying with jitter? Perhaps NATS is down?
                    }
                }
            }
        }
    }

    #[instrument(level = "debug", skip_all, fields(
        job.team_id = job.request.payload().team_id,
        job.ip_address = job.request.payload().ip_address,
        job.flag_hint = ?job.request.payload().flag_hint,
    ))]
    async fn handle_job(&self, job: Job, callback: &impl RunnerCallback) -> Result<()> {
        job.request.progress().await.context("unable to ack")?;
        match self.execute(job.request.payload(), callback).await {
            Err(error) => {
                warn! {
                    ?error,
                    "execution failed"
                }
                job.request.nak().await.context("unable to nak")?;
            }
            Ok(..) => {
                debug!("execution succeeded");
                job.request.ack().await.context("unable to ack")?;
            }
        }
        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    async fn execute(
        &self,
        request: &ExecutionRequest,
        callback: &impl RunnerCallback,
    ) -> Result<()> {
        debug!("performing execution");

        let mut command = tokio::process::Command::new(&self.exploit_command);
        command.env(ENV_EXPLOIT_NAME, &self.exploit_name);
        command.env(ENV_IP_ADDRESS, &request.ip_address);
        if let Some(flag_hint) = &request.flag_hint {
            let value =
                serde_json::to_string(flag_hint).context("unable to serialize flag hint")?;
            command.env(ENV_FLAG_HINT, value);
        }
        command.stdin(Stdio::null());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command.args(&self.exploit_args);

        let mut child = command.spawn().context("unable to spawn child")?;

        let stdout = child
            .stdout
            .take()
            .context("unable to retrieve a handle to the stdout pipe")?;
        let stderr = child
            .stderr
            .take()
            .context("unable to retrieve a handle to the stderr pipe")?;

        let handle_output = async move {
            let stdout_stream = LinesStream::new(BufReader::new(stdout).lines())
                .map(|line| line.map(OutputLine::Stdout));
            let stderr_stream = LinesStream::new(BufReader::new(stderr).lines())
                .map(|line| line.map(OutputLine::Stderr));

            let mut fused_stream = select_all(vec![stdout_stream.boxed(), stderr_stream.boxed()]);
            while let Some(Ok(line)) = fused_stream.next().await {
                match line {
                    OutputLine::Stdout(line) => {
                        debug!("stdout: {line}");
                        for m in self.flag_format.find_iter(&line) {
                            debug!("flag matched: {}", m.as_str());
                            callback
                                .on_flag(request, m.as_str())
                                .await
                                .context("flag callback failed")?;
                        }
                    }
                    OutputLine::Stderr(line) => {
                        debug!("stderr: {line}");
                        // TODO: Do something with this
                    }
                }
            }

            Ok::<(), color_eyre::eyre::ErrReport>(())
        };

        match tokio::time::timeout(self.timeout, handle_output).await {
            Ok(res) => res?,
            Err(_) => {
                warn! {
                    timeout = self.timeout.as_secs(),
                    "timeout exceeded"
                }
                child
                    .start_kill()
                    .context("unable to kill the exploit process")?;
            }
        }

        let exit_status = child.wait().await.context("unable to wait for child")?;

        if exit_status.success() {
            debug! {
                %exit_status,
                "exploit process exited with a successful exit code"
            }
        } else {
            warn! {
                %exit_status,
                "exploit process exited with an unsuccessful exit code"
            }
        }

        Ok(())
    }
}
