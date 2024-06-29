use anyhow::Result;
use clap::Parser;
use tokio::task::JoinSet;

mod config;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = config::Args::try_parse()?;

    let mut set = JoinSet::new();

    #[cfg(feature = "controller")]
    if args.components.enable_controller {
        set.spawn(kriger_controller::main());
    }
    #[cfg(feature = "fetcher")]
    if args.components.enable_fetcher {
        set.spawn(kriger_fetcher::main());
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
        set.spawn(kriger_runner::main());
    }
    #[cfg(feature = "submitter")]
    if args.components.enable_submitter {
        set.spawn(kriger_submitter::main());
    }
    #[cfg(feature = "ws")]
    if args.components.enable_ws {
        set.spawn(kriger_ws::main());
    }

    while let Some(res) = set.join_next().await {
        // Propagate error
        res??;
    }

    Ok(())
}
