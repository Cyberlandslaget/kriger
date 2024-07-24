pub mod args;

use crate::args::Args;
use color_eyre::eyre::{Context, Result};
use futures::StreamExt;
use kriger_common::messaging::model::ExecutionRequest;
use kriger_common::messaging::{AckPolicy, DeliverPolicy, Message, Messaging, Stream};
use kriger_common::runtime::AppRuntime;
use opentelemetry::metrics::MeterProvider;
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::runtime;
use std::time::Duration;
use tokio::{pin, select};
use tracing::{info, instrument, warn};

fn init_metrics() -> opentelemetry::metrics::Result<SdkMeterProvider> {
    opentelemetry_otlp::new_pipeline()
        .metrics(runtime::Tokio)
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_period(Duration::from_secs(10))
        .with_delta_temporality()
        .build()
}

#[instrument(skip_all)]
pub async fn main(runtime: AppRuntime, args: Args) -> Result<()> {
    info!("starting metrics exporter");

    let executions_wq = runtime
        .messaging
        .executions_wq()
        .await
        .context("unable to retrieve the execution work queue")?;

    // We're not using a durable name here. Make sure to only run a single instance of the metrics exporter.
    let execution_requests = executions_wq
        .subscribe::<ExecutionRequest>(
            None,
            Some("executions.*.request".to_string()),
            AckPolicy::None,
            DeliverPolicy::New,
        )
        .await
        .context("unable to subscribe to execution requests")?;
    pin!(execution_requests);

    // Environment variables:
    // OTEL_SERVICE_NAME
    // OTEL_EXPORTER_OTLP_METRICS_ENDPOINT
    let metrics = init_metrics().context("unable to initialize otlp metrics pipeline")?;

    let meter = metrics.meter("kriger");
    let execution_requests_counter = meter
        .u64_counter("kriger.execution.requests")
        .with_description("The number of execution requests")
        .with_unit("{request}")
        .init();

    loop {
        select! {
            // TODO: Investigate why the cancellation token is not working properly here.
            _ = runtime.cancellation_token.cancelled() => {
                info!("shutting down metrics ");
                metrics.shutdown()?;
                return Ok(());
            }
            res = execution_requests.next() => {
                match res {
                    Some(Ok(message)) => {
                        let payload = message.payload();
                        let mut labels = Vec::new();
                        if let Some(team_id) = &payload.team_id {
                            labels.push(KeyValue::new("team_id", team_id.clone()));
                        }
                        execution_requests_counter.add(1, &labels);
                    }
                    Some(Err(error)) => {
                        warn! {
                            ?error,
                            "unable to poll message"
                        }
                    }
                    None => {
                        // End of stream
                    }
                }
            }
        }
    }
}
