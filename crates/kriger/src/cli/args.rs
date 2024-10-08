use clap_derive::{Parser, Subcommand};

/// Deploy an exploit to the attack farm.
#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) struct Deploy {
    /// Do not deploy the exploit. This will only build the exploit and push it to the registry.
    #[arg(long)]
    pub(crate) no_deploy: bool,

    /// Do not immediately execute the exploit. This will not immediately execute the exploit after deploying.
    #[arg(long)]
    pub(crate) no_execute: bool,
}

/// Create a new exploit based on a template.
#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) struct Create {
    /// The service name that the exploit should target
    #[arg(long)]
    pub(crate) service: Option<String>,

    /// The exploit's name
    pub(crate) name: Option<String>,
}

/// Manually submit a flag
#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) struct Submit {
    /// The input containing the flag(s) to submit
    pub(crate) input: String,
}

#[derive(Subcommand, Debug)]
#[command(version, about)]
pub(crate) enum ExploitCommand {
    /// Retrieve flag hints
    Hints(ExploitHints),
    /// Interactive exploit development
    #[cfg(feature = "runner")]
    Dev(ExploitDev),
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) struct ExploitHints {}

#[derive(Parser, Debug)]
#[command(version, about)]
#[cfg(feature = "runner")]
pub(crate) struct ExploitDev {
    /// The team ID to run the exploit against. Defaults to nop.
    pub team_id: Option<String>,
}
