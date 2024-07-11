use anyhow::Result;
use clap::Parser;
use kriger_common::messaging::nats::NatsMessaging;
use kriger_common::runtime::AppRuntime;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::{info, warn};

mod config;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = config::Args::try_parse()?;

    info!("initializing messaging");
    let messaging = NatsMessaging::new(&args.common.nats_url).await?;

    // TODO: Move this somewhere else
    messaging.do_migration().await?;

    let runtime = AppRuntime {
        config: Arc::new(args.common),
        messaging: Arc::new(messaging),
    };

    info!("starting components");
    let mut set = JoinSet::new();

    #[cfg(feature = "controller")]
    if args.components.enable_controller {
        set.spawn(kriger_controller::main(runtime.clone(), args.controller));
    }
    #[cfg(feature = "fetcher")]
    if args.components.enable_fetcher {
        set.spawn(kriger_fetcher::main(runtime.clone()));
    }
    #[cfg(feature = "metrics")]
    if args.components.enable_metrics {
        set.spawn(kriger_metrics::main());
    }
    #[cfg(feature = "rest")]
    if args.components.enable_rest {
        set.spawn(kriger_rest::main());
    }
    #[cfg(feature = "runner")]
    if args.components.enable_runner {
        set.spawn(kriger_runner::main(runtime.clone(), args.runner));
    }
    #[cfg(feature = "submitter")]
    if args.components.enable_submitter {
        set.spawn(kriger_submitter::main());
    }
    #[cfg(feature = "ws")]
    if args.components.enable_ws {
        set.spawn(kriger_ws::main());
    }

    if set.is_empty() {
        warn!("no components enabled, see --help for a list of components");
    }

    while let Some(res) = set.join_next().await {
        // Propagate error
        res??;
    }

    Ok(())
}
