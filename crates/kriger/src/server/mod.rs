use color_eyre::eyre;
use kriger_common::messaging::nats::NatsMessaging;
use kriger_common::server::runtime::{create_shutdown_cancellation_token, AppConfig, AppRuntime};
use std::path::Path;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::{info, warn};

pub(crate) mod args;

async fn read_app_config<P: AsRef<Path>>(path: P) -> eyre::Result<AppConfig> {
    let content = tokio::fs::read_to_string(path).await?;
    let config = toml::from_str(&content)?;
    Ok(config)
}

pub(crate) async fn main(args: args::Args) -> eyre::Result<()> {
    let app_config = read_app_config(args.config_file).await?;

    info!("initializing messaging");
    let messaging = NatsMessaging::new(&args.common.nats_url, Some(&app_config)).await?;

    let cancellation_token = create_shutdown_cancellation_token();

    let runtime = AppRuntime {
        config: Arc::new(app_config),
        messaging: Arc::new(messaging),
        cancellation_token,
    };

    info!("starting components");
    let mut set = JoinSet::new();

    #[cfg(feature = "controller")]
    if args.components.enable_controller || args.components.single {
        set.spawn(kriger_controller::main(runtime.clone(), args.controller));
    }
    #[cfg(feature = "fetcher")]
    if args.components.enable_fetcher || args.components.single {
        set.spawn(kriger_fetcher::main(runtime.clone()));
    }
    #[cfg(feature = "metrics")]
    if args.components.enable_metrics {
        // TODO: Consider enabling this by default with --single?
        set.spawn(kriger_metrics::main(runtime.clone(), args.metrics));
    }
    #[cfg(feature = "rest")]
    if args.components.enable_rest || args.components.single {
        set.spawn(kriger_rest::main(runtime.clone(), args.rest));
    }
    #[cfg(feature = "scheduler")]
    if args.components.enable_scheduler || args.components.single {
        set.spawn(kriger_scheduler::main(runtime.clone()));
    }
    #[cfg(feature = "submitter")]
    if args.components.enable_submitter || args.components.single {
        set.spawn(kriger_submitter::main(runtime.clone()));
    }
    #[cfg(feature = "ws")]
    if args.components.enable_ws || args.components.single {
        set.spawn(kriger_ws::main(runtime.clone(), args.ws));
    }

    if set.is_empty() {
        warn!("no components enabled, see --help for a list of components");
        warn!("hint: use --single to enable the default components for a simple setup");
    }

    while let Some(res) = set.join_next().await {
        // Propagate error
        res??;
    }

    Ok(())
}
