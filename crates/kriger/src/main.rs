use clap::Parser;
use color_eyre::eyre::Result;

mod args;

#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "server")]
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
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
        args::Commands::Server(args) => server::main(args).await,
        #[cfg(feature = "server")]
        args::Commands::Runner(args) => kriger_runner::main(args).await,
        #[cfg(feature = "cli")]
        args::Commands::Deploy(args) => cli::deploy::main(args).await,
    }
}
