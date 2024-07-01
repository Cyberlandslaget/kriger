use clap::ArgAction;
use clap_derive::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
#[group(skip)]
pub struct Config {
    /// The Kubernetes namespace to schedule exploits in
    #[arg(
        env, long, action = ArgAction::Set, default_value = "kriger-exploits"
    )]
    pub controller_exploit_namespace: String,
}