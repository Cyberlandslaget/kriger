use crate::runner::{Runner, RunnerError, RunnerEvent, RunnerExecution, RunnerExecutionResult};
use futures::stream::select_all;
use futures::StreamExt;
use regex::Regex;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_stream::wrappers::LinesStream;
const ENV_EXPLOIT_NAME: &'static str = "EXPLOIT";
const ENV_IP_ADDRESS: &'static str = "IP";
const ENV_FLAG_HINT: &'static str = "HINT";

pub struct SimpleRunner {
    pub exploit_name: String,
    pub exploit_command: String,
    pub exploit_args: Vec<String>,
    pub flag_format: Regex,
    pub timeout: Duration,
}

struct SimpleRunnerExecution<'a> {
    child: tokio::process::Child,
    events_rx: flume::Receiver<RunnerEvent>,
    events_tx: flume::Sender<RunnerEvent>,
    timeout: Duration,
    flag_format: &'a Regex,
}

impl Runner for SimpleRunner {
    async fn run<S: AsRef<str>>(
        &self,
        ip_address: S,
        flag_hint: &Option<serde_json::Value>,
    ) -> Result<impl RunnerExecution, RunnerError> {
        let mut command = tokio::process::Command::new(&self.exploit_command);
        command.env(ENV_EXPLOIT_NAME, &self.exploit_name);
        command.env(ENV_IP_ADDRESS, ip_address.as_ref());
        if let Some(flag_hint) = &flag_hint {
            let value = serde_json::to_string(flag_hint)?;
            command.env(ENV_FLAG_HINT, value);
        }
        command.stdin(Stdio::null());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command.args(&self.exploit_args);

        let child = command.spawn()?;
        let (events_tx, events_rx) = flume::unbounded();
        Ok(SimpleRunnerExecution {
            child,
            events_tx,
            events_rx,
            timeout: self.timeout,
            flag_format: &self.flag_format,
        })
    }
}

impl RunnerExecution for SimpleRunnerExecution<'_> {
    async fn complete(mut self) -> RunnerExecutionResult {
        let start = tokio::time::Instant::now();
        match tokio::time::timeout(self.timeout, self.wait_for_child()).await {
            Ok(Ok(status)) => RunnerExecutionResult {
                time: start.elapsed(),
                exit_code: status.code(),
                error: None,
            },
            Ok(Err(error)) => {
                // TODO: We may end up with a zombie process. Handle it somehow?
                // Ensure that the child has exited / is killed.
                _ = self.child.start_kill();

                RunnerExecutionResult {
                    time: start.elapsed(),
                    exit_code: None,
                    error: Some(error),
                }
            }
            Err(_) => {
                // TODO: We may end up with a zombie process. Handle it somehow?
                _ = self.child.start_kill();

                RunnerExecutionResult {
                    time: start.elapsed(),
                    exit_code: None,
                    error: Some(RunnerError::ExecutionTimeout),
                }
            }
        }
    }

    fn events(&self) -> flume::Receiver<RunnerEvent> {
        self.events_rx.clone()
    }
}

impl SimpleRunnerExecution<'_> {
    async fn wait_for_child(&mut self) -> Result<std::process::ExitStatus, RunnerError> {
        let stdout = self
            .child
            .stdout
            .take()
            .ok_or(RunnerError::StdoutUnavailable)?;
        let stderr = self
            .child
            .stderr
            .take()
            .ok_or(RunnerError::StdoutUnavailable)?;

        let stdout_stream = LinesStream::new(BufReader::new(stdout).lines())
            .map(|line| line.map(RunnerEvent::Stdout));
        let stderr_stream = LinesStream::new(BufReader::new(stderr).lines())
            .map(|line| line.map(RunnerEvent::Stderr));

        let mut fused_stream = select_all(vec![stdout_stream.boxed(), stderr_stream.boxed()]);
        while let Some(maybe_event) = fused_stream.next().await {
            let event = maybe_event?;
            if let RunnerEvent::Stdout(line) = &event {
                for flag in self.flag_format.find_iter(&line) {
                    let event = RunnerEvent::FlagMatch(flag.as_str().to_string());
                    self.events_tx.send_async(event).await?;
                }
            }
            self.events_tx.send_async(event).await?;
        }
        let exit_status = self.child.wait().await?;
        Ok(exit_status)
    }
}
