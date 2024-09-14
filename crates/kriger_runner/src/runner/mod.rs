pub mod simple;

use std::future::Future;
use std::time::Duration;

#[derive(thiserror::Error, Debug)]
pub enum RunnerError {
    #[error("serde json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("send error: {0}")]
    FlumeSendError(#[from] flume::SendError<RunnerEvent>),
    #[error("stdout unavailable")]
    StdoutUnavailable,
    #[error("stderr unavailable")]
    StderrUnavailable,
    #[error("execution timed out")]
    ExecutionTimeout,
}

pub trait Runner {
    fn run<S: AsRef<str> + Send + Sync>(
        &self,
        ip_address: S,
        flag_hint: &Option<serde_json::Value>,
    ) -> impl Future<Output = Result<impl RunnerExecution, RunnerError>> + Send;
}

pub trait RunnerExecution {
    fn complete(self) -> impl Future<Output = RunnerExecutionResult> + Send;

    fn events(&self) -> flume::Receiver<RunnerEvent>;
}

pub enum RunnerEvent {
    Stdout(String),
    Stderr(String),
    FlagMatch(String),
}

#[derive(Debug)]
pub struct RunnerExecutionResult {
    pub time: Duration,
    pub exit_code: Option<i32>,
    pub error: Option<RunnerError>,
}
