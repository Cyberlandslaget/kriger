use clap_derive::Parser;

/// An exploit farm for attack/defense CTFs
#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) struct Deploy {
    /// Do not immediately launch the exploit
    #[arg(long)]
    pub(crate) no_launch: bool,

    /// URL for REST API
    #[arg(env, long, default_value = "https://kriger.o99.no:8000")]
    pub(crate) rest_url: String,
}
