use anyhow::Result;
use clap::Parser;

mod args;

#[cfg(feature = "server")]
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let main_args = args::Args::try_parse()?;

    match main_args.command {
        #[cfg(feature = "server")]
        args::Commands::Server(args) => server::main(args).await,
    }
}
