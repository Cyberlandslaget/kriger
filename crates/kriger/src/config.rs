use clap::ArgAction;
use clap_derive::{Parser, Args};

/// An exploit farm for attack/defense CTFs
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// The path to the configuration file
    #[arg(env, default_value = "config.toml", value_name = "CONFIG_FILE")]
    pub config_file: String,

    #[command(flatten, next_help_heading = "Component selection options")]
    pub components: Components,

    #[command(flatten, next_help_heading = "Common configuration options")]
    pub common: kriger_common::config::Config,

    #[cfg(feature = "controller")]
    #[command(flatten, next_help_heading = "Controller configuration options")]
    pub controller: kriger_controller::config::Config,
}

/// Components
#[derive(Args, Debug)]
#[group()]
pub struct Components {
    /// Enable the kriger-controller component
    #[cfg(feature = "controller")]
    #[arg(env, long, action = ArgAction::Set, default_value_t = true)]
    pub enable_controller: bool,

    /// Enable the kriger-fetcher component
    #[cfg(feature = "fetcher")]
    #[arg(env, long, action = ArgAction::Set, default_value_t = true)]
    pub enable_fetcher: bool,

    /// Enable the kriger-metrics component
    #[cfg(feature = "metrics")]
    #[arg(env, long, action = ArgAction::Set, default_value_t = true)]
    pub enable_metrics: bool,

    /// Enable the kriger-rest component
    #[cfg(feature = "rest")]
    #[arg(env, long, action = ArgAction::Set, default_value_t = true)]
    pub enable_rest: bool,

    /// Enable the kriger-runner component
    #[cfg(feature = "runner")]
    #[arg(env, long, action = ArgAction::Set, default_value_t = false)]
    pub enable_runner: bool,

    /// Enable the kriger-submitter component
    #[cfg(feature = "submitter")]
    #[arg(env, long, action = ArgAction::Set, default_value_t = true)]
    pub enable_submitter: bool,

    /// Enable the kriger-ws component
    #[cfg(feature = "ws")]
    #[arg(env, long, action = ArgAction::Set, default_value_t = false)]
    pub enable_ws: bool,
}
