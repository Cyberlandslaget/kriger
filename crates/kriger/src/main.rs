use clap::Parser;
use color_eyre::eyre;
use color_eyre::eyre::Context;
use tracing::metadata::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, Registry};

mod args;

#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "server")]
mod server;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let main_args = match args::Args::try_parse() {
        Ok(args) => args,
        Err(err) => {
            // eyre will format this in an unwanted way
            eprintln!("{err}");
            return Ok(());
        }
    };

    match main_args.command {
        #[cfg(feature = "server")]
        args::Commands::Server(args) => {
            init_tracing(true)?;
            server::main(args).await
        }
        #[cfg(feature = "server")]
        args::Commands::Runner(args) => {
            init_tracing(true)?;
            kriger_runner::main(args).await
        }
        #[cfg(feature = "cli")]
        args::Commands::Deploy(args) => {
            init_tracing(false)?;
            cli::deploy::main(args).await
        }
        #[cfg(feature = "cli")]
        args::Commands::Create(args) => {
            init_tracing(false)?;
            cli::create::main(args).await
        }
    }
}

fn init_tracing(use_otel: bool) -> eyre::Result<()> {
    let registry = Registry::default().with(
        tracing_subscriber::fmt::layer().with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        ),
    );

    #[cfg(feature = "otel")]
    {
        if use_otel {
            use opentelemetry::global;
            use opentelemetry::trace::TracerProvider;
            use opentelemetry::KeyValue;
            use opentelemetry_sdk::runtime;
            use opentelemetry_sdk::trace::BatchConfig;
            use opentelemetry_sdk::Resource;
            use opentelemetry_semantic_conventions::attribute::SERVICE_NAME;

            let provider = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(opentelemetry_otlp::new_exporter().tonic())
                // TODO: Sampling?
                .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(
                    Resource::new(vec![KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME"))]),
                ))
                .with_batch_config(BatchConfig::default())
                .install_batch(runtime::Tokio)
                .context("unable to construct a tracing pipeline")?;
            global::set_tracer_provider(provider.clone());
            let tracer = provider.tracer("kriger");

            registry
                .with(
                    tracing_opentelemetry::layer()
                        .with_tracer(tracer)
                        .with_filter(LevelFilter::DEBUG),
                )
                .init();
        } else {
            registry.init();
        }
    }
    #[cfg(not(feature = "otel"))]
    registry.init();

    Ok(())
}
