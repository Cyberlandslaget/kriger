pub mod args;
mod runner;

use crate::args::Args;
use crate::runner::{Job, Runner, RunnerCallback};
use color_eyre::eyre::{bail, Context, ContextCompat, Result};
use futures::StreamExt;
use kriger_common::messaging::model::{ExecutionRequest, FlagSubmission};
use kriger_common::messaging::nats::NatsMessaging;
use kriger_common::messaging::services::flags::FlagsService;
use kriger_common::server::runtime::create_shutdown_cancellation_token;
use regex::Regex;
use std::str;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::{pin, select, spawn};
use tracing::{debug, info, warn};

#[derive(Clone)]
struct RunnerCallbackImpl {
    flags_svc: FlagsService,
    exploit: String,
    service: Option<String>,
}

impl RunnerCallback for RunnerCallbackImpl {
    async fn on_flag(&self, request: &ExecutionRequest, flag: &str) -> Result<()> {
        let submission = FlagSubmission {
            flag: flag.to_string(),
            team_id: request.team_id.clone(),
            service: self.service.clone(),
            exploit: Some(self.exploit.clone()),
        };
        self.flags_svc.submit_flag(&submission).await?;

        Ok(())
    }
}

pub async fn main(args: Args) -> Result<()> {
    info!("initializing messaging");
    let messaging = NatsMessaging::new(&args.nats_url, None).await?;
    let cancellation_token = create_shutdown_cancellation_token();

    let flag_format =
        Regex::new(&args.flag_format).context("unable to parse the flag format regex")?;

    info!("using the flag format: `{flag_format}`");
    info!(
        "subscribing to execution requests for exploit: {}",
        &args.exploit
    );
    let stream = messaging
        .executions()
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

    let callback = RunnerCallbackImpl {
        flags_svc: messaging.flags(),
        exploit: args.exploit.clone(),
        service: args.service,
    };

    let runner = Runner {
        job_rx: rx,
        exploit_name: args.exploit,
        exploit_command: args.command,
        exploit_args: args.args,
        flag_format,
        timeout: Duration::from_secs(args.timeout),
    };

    for i in 0..worker_count {
        spawn(
            runner
                .clone()
                .worker(i, callback.clone(), cancellation_token.clone()),
        );
    }

    loop {
        select! {
            _ = cancellation_token.cancelled() => {
                return Ok(());
            }
            maybe_permit = semaphore.clone().acquire_owned() => {
                let permit = maybe_permit.context("permit acquisition failed")?;
                debug!("permit acquired: {permit:?}");
                select! {
                    _ = cancellation_token.cancelled() => {
                        return Ok(());
                    }
                    maybe_message = stream.next() => {
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
                            Err(err) => warn!("unable to parse message: {err:?}"),
                        };
                    }
                }
            }
        }
    }
}
