use clap_derive::{Args, Parser};

/// An exploit farm for attack/defense CTFs
#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) struct Args {
    #[command(flatten, next_help_heading = "Component selection options")]
    pub components: Components,

    #[command(flatten, next_help_heading = "Common configuration options")]
    pub common: kriger_common::config::Config,

    #[cfg(feature = "controller")]
    #[command(flatten, next_help_heading = "Controller configuration options")]
    pub controller: kriger_controller::config::Config,

    #[cfg(feature = "metrics")]
    #[command(flatten, next_help_heading = "Metrics exporter configuration options")]
    pub metrics: kriger_metrics::args::Args,

    #[cfg(feature = "rest")]
    #[command(flatten, next_help_heading = "REST API server configuration options")]
    pub rest: kriger_rest::config::Config,

    #[cfg(feature = "ws")]
    #[command(flatten, next_help_heading = "WebSocket server configuration options")]
    pub ws: kriger_ws::config::Config,
}

/// Components
#[derive(Args, Debug)]
#[group()]
pub(crate) struct Components {
    /// Enable the default components for a simple single-instance setup
    #[arg(env, long)]
    pub single: bool,

    /// Enable the kriger-controller component
    #[cfg(feature = "controller")]
    #[arg(env, long)]
    pub enable_controller: bool,

    /// Enable the kriger-fetcher component
    #[cfg(feature = "fetcher")]
    #[arg(env, long)]
    pub enable_fetcher: bool,

    /// Enable the kriger-metrics component
    #[cfg(feature = "metrics")]
    #[arg(env, long)]
    pub enable_metrics: bool,

    /// Enable the kriger-rest component
    #[cfg(feature = "rest")]
    #[arg(env, long)]
    pub enable_rest: bool,

    /// Enable the kriger-scheduler component
    #[cfg(feature = "scheduler")]
    #[arg(env, long)]
    pub enable_scheduler: bool,

    /// Enable the kriger-submitter component
    #[cfg(feature = "submitter")]
    #[arg(env, long)]
    pub enable_submitter: bool,

    /// Enable the kriger-ws component
    #[cfg(feature = "ws")]
    #[arg(env, long)]
    pub enable_ws: bool,
}
