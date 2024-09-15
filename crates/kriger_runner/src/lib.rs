pub mod args;
pub mod runner;

use crate::args::Args;
use crate::runner::simple::SimpleRunner;
use crate::runner::{Runner, RunnerError, RunnerEvent, RunnerExecution, RunnerExecutionResult};
use async_nats::jetstream::AckKind;
use color_eyre::eyre;
use color_eyre::eyre::{bail, Context, ContextCompat};
use futures::StreamExt;
use kriger_common::messaging;
use kriger_common::messaging::model::FlagSubmission;
use kriger_common::messaging::nats::{MessageWrapper, MessagingServiceError, NatsMessaging};
use kriger_common::messaging::services::executions::ExecutionsService;
use kriger_common::messaging::services::flags::FlagsService;
use kriger_common::server::runtime::create_shutdown_cancellation_token;
use regex::Regex;
use std::str;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::task::JoinSet;
use tokio::{join, pin, select};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, warn};

pub struct Job {
    pub request: MessageWrapper<messaging::model::ExecutionRequest>,
    pub _permit: OwnedSemaphorePermit,
}

pub async fn main(args: Args) -> eyre::Result<()> {
    info!("initializing messaging");
    let messaging = NatsMessaging::new(&args.nats_url, None).await?;
    let cancellation_token = create_shutdown_cancellation_token();

    let flag_format =
        Regex::new(&args.flag_format).context("unable to parse the flag format regex")?;

    let executions_svc = Arc::new(messaging.executions());
    let flags_svc = Arc::new(messaging.flags());

    info!("using the flag format: `{flag_format}`");
    info!(
        "subscribing to execution requests for exploit: {}",
        &args.exploit
    );
    let stream = executions_svc
        .subscribe_execution_requests(
            Some(format!("exploit:{}", &args.exploit)),
            Some(args.exploit.as_str()),
        )
        .await
        .context("unable to subscribe to execution requests")?;
    pin!(stream);

    let worker_count = args.workers.unwrap_or_else(|| 2 * num_cpus::get());
    let semaphore = Arc::new(Semaphore::new(worker_count));
    info!("using a maximum of {worker_count} workers");

    let (tx, rx) = async_channel::bounded::<Job>(worker_count);

    let runner = Arc::new(SimpleRunner {
        exploit_name: args.exploit.clone(),
        exploit_command: args.command,
        exploit_args: args.args,
        flag_format,
        timeout: Duration::from_secs(args.timeout),
    });

    let mut set = JoinSet::new();
    for i in 0..worker_count {
        set.spawn(worker(
            i,
            rx.clone(),
            runner.clone(),
            flags_svc.clone(),
            executions_svc.clone(),
            args.exploit.clone(),
            args.service.clone(),
            cancellation_token.clone(),
        ));
    }

    loop {
        let maybe_permit = select! {
            _ = cancellation_token.cancelled() => break,
            maybe_permit = semaphore.clone().acquire_owned() => maybe_permit
        };

        let permit = maybe_permit.context("permit acquisition failed")?;
        debug!("permit acquired: {permit:?}");

        let maybe_message = select! {
            _ = cancellation_token.cancelled() => break,
            maybe_message = stream.next() => maybe_message
        };
        match maybe_message.context("end of stream")? {
            Ok(message) => {
                let job = Job {
                    request: message,
                    _permit: permit,
                };
                if let Err(err) = tx.send(job).await {
                    bail!("channel closed: {err:?}");
                }
            }
            Err(MessagingServiceError::ProcessingError { message, error }) => {
                warn! {
                    ?error,
                    "messaging processing error"
                }
                _ = message.ack_with(AckKind::Term).await;
            }
            Err(error) => {
                error! {
                    ?error,
                    "unexpected messaging error"
                }
            }
        };
    }
    while let Some(res) = set.join_next().await {
        debug!("worker shutdown");
        res.context("join error")?;
    }
    Ok(())
}

async fn worker(
    idx: usize,
    job_rx: async_channel::Receiver<Job>,
    runner: Arc<impl Runner>,
    flags_svc: Arc<FlagsService>,
    executions_svc: Arc<ExecutionsService>,
    exploit: String,
    service: Option<String>,
    cancellation_token: CancellationToken,
) {
    loop {
        let maybe_job = select! {
            // Wait for the jobs to gracefully shut down
            _ = cancellation_token.cancelled() => return,
            maybe_job = job_rx.recv() => maybe_job
        };
        let job = match maybe_job {
            Ok(job) => job,
            Err(error) => {
                error! {
                    ?error,
                    "unable to receive job"
                }

                // Something is wrong. The channel has most likely been closed
                cancellation_token.cancel();
                return;
            }
        };
        if let Err(error) = handle_job(
            idx,
            runner.as_ref(),
            flags_svc.as_ref(),
            executions_svc.as_ref(),
            &exploit,
            &service,
            job,
        )
        .await
        {
            error! {
                ?error,
                "unexpected job handling error"
            }
            // TODO: Consider delaying with jitter? Perhaps NATS is down?
        }
    }
}

#[instrument(level = "debug", skip_all, fields(
    worker = worker_idx,
    job.team_id = job.request.payload.team_id,
    job.ip_address = job.request.payload.ip_address,
    job.flag_hint = ?job.request.payload.flag_hint,
))]
async fn handle_job(
    #[allow(unused_variables)] // This is used by tracing's `instrument` macro
    worker_idx: usize,
    runner: &impl Runner,
    flags_svc: &FlagsService,
    executions_svc: &ExecutionsService,
    exploit_name: &str,
    service: &Option<String>,
    job: Job,
) -> eyre::Result<()> {
    debug!("processing job");
    job.request.progress().await.context("unable to ack")?;

    let payload = &job.request.payload;
    let execution = match runner.run(&payload.ip_address, &payload.flag_hint).await {
        Ok(execution) => execution,
        Err(error) => {
            error! {
                ?error,
                "unexpected runner error"
            }
            // TODO: Improve retry
            job.request
                .retry_linear(Duration::from_secs(2), 3)
                .await
                .context("unable to nak")?;
            return Ok(());
        }
    };
    let (has_events_failed, result) = join!(
        handle_events(
            flags_svc,
            exploit_name,
            service,
            &payload.team_id,
            execution.events()
        ),
        execution.complete()
    );

    let should_retry = match &result {
        RunnerExecutionResult {
            time,
            error: Some(error),
            exit_code,
        } => {
            warn! {
                ?time,
                ?error,
                exit_code,
                "exploit execution completed with an error"
            }
            true
        }
        // TODO: Retry on status
        result => {
            debug! {
                ?result,
                "exploit execution execution completed"
            }
            false
        }
    };
    let result_message = messaging::model::ExecutionResult {
        team_id: payload.team_id.clone(),
        time: result.time.as_millis(),
        exit_code: result.exit_code,
        status: match &result {
            RunnerExecutionResult {
                error: Some(RunnerError::ExecutionTimeout),
                ..
            } => messaging::model::ExecutionResultStatus::Timeout,
            RunnerExecutionResult { error: Some(_), .. } => {
                messaging::model::ExecutionResultStatus::Error
            }
            _ => messaging::model::ExecutionResultStatus::Success,
        },
        request_sequence: job.request.info.stream_sequence,
        attempt: Some(job.request.info.delivered),
    };

    let res = executions_svc
        .publish_execution_result(&exploit_name, &result_message)
        .await;
    let result_published = match res {
        Ok(_) => true,
        Err(error) => {
            error! {
                ?error,
                "unable to publish execution result"
            }
            false
        }
    };

    if has_events_failed || should_retry || !result_published {
        job.request
            .retry_linear(Duration::from_secs(2), 3)
            .await
            .context("unable to nak")?;
    } else {
        job.request.ack().await.context("unable to ack")?;
    }
    Ok(())
}

async fn handle_events(
    flags_svc: &FlagsService,
    exploit: &str,
    service: &Option<String>,
    team_id: &Option<String>,
    events: flume::Receiver<RunnerEvent>,
) -> bool {
    let mut should_retry = false;
    while let Ok(event) = events.recv_async().await {
        match event {
            RunnerEvent::FlagMatch(flag) => {
                debug! {
                    flag,
                    "flag matched"
                }
                let submission = FlagSubmission {
                    flag: flag.to_string(),
                    exploit: Some(exploit.to_string()),
                    service: service.clone(),
                    team_id: team_id.clone(),
                };
                if let Err(error) = flags_svc.submit_flag(&submission).await {
                    should_retry = true;
                    error! {
                        ?error,
                        "unable to submit flag"
                    }
                }
            }
            RunnerEvent::Stdout(line) => {
                debug!("stdout: {line}");
            }
            RunnerEvent::Stderr(line) => {
                debug!("stderr: {line}");
            }
        }
    }
    should_retry
}
