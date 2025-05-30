// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

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
            cli::commands::deploy::main(args).await
        }
        #[cfg(feature = "cli")]
        args::Commands::Create(args) => {
            init_tracing(false)?;
            cli::commands::create::main(args).await
        }
        #[cfg(feature = "cli")]
        args::Commands::Submit(args) => {
            init_tracing(false)?;
            cli::commands::submit::main(args).await
        }
        #[cfg(feature = "cli")]
        args::Commands::Exploit(args) => {
            init_tracing(false)?;
            cli::commands::exploit::main(args).await
        }
    }
}

fn init_tracing(use_otel: bool) -> eyre::Result<()> {
    let registry = Registry::default().with(
        tracing_subscriber::fmt::layer().with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy()
                .add_directive("h2=info".parse()?)
                .add_directive("async_nats=info".parse()?)
                .add_directive("tower=info".parse()?),
        ),
    );

    #[cfg(feature = "otel")]
    {
        if use_otel {
            use opentelemetry::global;
            use opentelemetry::trace::TracerProvider;
            use opentelemetry_sdk::runtime;
            use opentelemetry_sdk::trace::BatchConfig;

            // TODO: Sampling?
            let provider = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(opentelemetry_otlp::new_exporter().tonic())
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
