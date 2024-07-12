use color_eyre::eyre::Result;
use kriger_common::messaging::nats::NatsMessaging;
use kriger_common::runtime::AppRuntime;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::{info, warn};

pub(crate) mod args;

pub(crate) async fn main(args: args::Args) -> Result<()> {
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
    if args.components.enable_controller || args.components.single {
        set.spawn(kriger_controller::main(runtime.clone(), args.controller));
    }
    #[cfg(feature = "fetcher")]
    if args.components.enable_fetcher || args.components.single {
        set.spawn(kriger_fetcher::main(runtime.clone()));
    }
    #[cfg(feature = "metrics")]
    if args.components.enable_metrics || args.components.single {
        set.spawn(kriger_metrics::main());
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
        set.spawn(kriger_submitter::main());
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
