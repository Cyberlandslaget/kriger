use clap_derive::Parser;

#[cfg(debug_assertions)]
const DEFAULT_REST_URL: &str = "http://localhost:8000";
#[cfg(not(debug_assertions))]
const DEFAULT_REST_URL: &str = "https://kriger.o99.no/api";

#[cfg(debug_assertions)]
const DEFAULT_REGISTRY: &str = "localhost:5000";
#[cfg(not(debug_assertions))]
const DEFAULT_REGISTRY: &str = "r.o99.no";

/// An exploit farm for attack/defense CTFs
#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) struct Deploy {
    /// Do not immediately launch the exploit
    #[arg(long)]
    pub(crate) no_launch: bool,

    /// URL for REST API
    #[arg(env, long, default_value = DEFAULT_REST_URL)]
    pub(crate) rest_url: String,

    /// The registry to push the image to
    #[arg(env, long, default_value = DEFAULT_REGISTRY)]
    pub(crate) registry: String,
}
