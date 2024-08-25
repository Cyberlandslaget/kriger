use clap_derive::Parser;

/// Deploy an exploit to the attack farm.
#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) struct Deploy {
    /// Do not deploy the exploit. This will only build the exploit and push it to the registry.
    #[arg(long)]
    pub(crate) no_deploy: bool,
}

/// Create a new exploit based on a template.
#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) struct Create {
    #[arg(long)]
    /// The service name that the exploit should target
    pub(crate) service: Option<String>,

    /// The exploit's name
    pub(crate) name: Option<String>,
}
