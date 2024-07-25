pub mod config;
mod runner;

use crate::config::Config;
use crate::runner::{Job, Runner, RunnerCallback};
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use color_eyre::eyre::{bail, Context, ContextCompat, Result};
use futures::StreamExt;
use kriger_common::messaging::model::{CompetitionConfig, ExecutionRequest, FlagSubmission};
use kriger_common::messaging::nats::NatsMessaging;
use kriger_common::messaging::{
    AckPolicy, Bucket, DeliverPolicy, Messaging, MessagingError, Stream,
};
use kriger_common::runtime::create_shutdown_cancellation_token;
use regex::Regex;
use std::str;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::{pin, select, spawn};
use tracing::{debug, info, instrument, warn};

#[derive(Clone)]
struct RunnerCallbackImpl<T: Bucket> {
    flags: Box<T>,
    exploit: String,
    service: Option<String>,
}

impl<T: Bucket> RunnerCallback for RunnerCallbackImpl<T> {
    async fn on_flag(&self, request: &ExecutionRequest, flag: &str) -> Result<()> {
        let key = format!("{}.submit", STANDARD_NO_PAD.encode(flag.as_bytes()));
        let payload = FlagSubmission {
            flag: flag.to_string(),
            team_id: request.team_id.clone(),
            service: self.service.clone(),
            exploit: Some(self.exploit.clone()),
        };
        let res = self.flags.create(&key, &payload).await;
        if let Err(MessagingError::KeyValueConflictError) = res {
            // This means that the flag has already been submitted. We don't need to treat it as
            // an error.
            debug!("the flag `{flag}` already exists, ignoring");
            return Ok(());
        }

        // Propagate other error
        res?;

        Ok(())
    }
}

#[instrument(skip_all)]
pub async fn main(args: Config) -> Result<()> {
    info!("initializing messaging");
    let messaging = NatsMessaging::new(&args.nats_url).await?;
    let cancellation_token = create_shutdown_cancellation_token();

    let config_bucket = messaging
        .config()
        .await
        .context("unable to retrieve the config bucket")?;

    // TODO: Provide a more elegant way to retrieve this and add support for live reload
    let competition_config = config_bucket
        .get::<CompetitionConfig>("competition")
        .await
        .context("unable to retrieve the competition config")?
        .context("the competition config does not exist")?;
    let flag_format = Regex::new(&competition_config.flag_format)
        .context("unable to parse the flag format regex")?;

    info!("using the flag format: `{flag_format}`");
    info!(
        "subscribing to execution requests for exploit: {}",
        &args.exploit
    );
    let executions_wq = messaging
        .executions_wq()
        .await
        .context("unable to get the execution work queue")?;
    let stream = executions_wq
        .subscribe(
            Some(format!("exploit:{}", &args.exploit)),
            Some(format!("executions.{}.request", args.exploit)),
            AckPolicy::Explicit,
            DeliverPolicy::New,
        )
        .await
        .context("unable to subscribe to execution requests")?;
    pin!(stream);

    let worker_count = args.workers.unwrap_or_else(|| 2 * num_cpus::get());
    let semaphore = Arc::new(Semaphore::new(worker_count));
    info!("using a maximum of {worker_count} workers");

    let (tx, rx) = async_channel::bounded::<Job>(worker_count);

    let callback = RunnerCallbackImpl {
        flags: Box::new(
            messaging
                .flags()
                .await
                .context("unable to retrieve the flags bucket")?,
        ),
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
                match stream.next().await.context("end of stream")? {
                    Ok(message) => {
                        let job = Job {
                            request: Box::new(message),
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
