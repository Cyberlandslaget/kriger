use clap_derive::Parser;

#[cfg(debug_assertions)]
const DEFAULT_REST_URL: &str = "http://localhost:8000";
#[cfg(not(debug_assertions))]
const DEFAULT_REST_URL: &str = "https://kriger.o99.no/api";

#[cfg(debug_assertions)]
const DEFAULT_REGISTRY: &str = "localhost:5000";
#[cfg(not(debug_assertions))]
const DEFAULT_REGISTRY: &str = "r.o99.no";

/// Deploy an exploit to the attack farm.
#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) struct Deploy {
    /// Do not deploy the exploit. This will only build the exploit and push it to the registry.
    #[arg(long)]
    pub(crate) no_deploy: bool,

    /// URL for REST API
    #[arg(env, long, default_value = DEFAULT_REST_URL)]
    pub(crate) rest_url: String,

    /// The registry to push the image to
    #[arg(env, long, default_value = DEFAULT_REGISTRY)]
    pub(crate) registry: String,
}

/// Create a new exploit based on a template.
#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) struct Create {
    /// URL for REST API
    #[arg(env, long, default_value = DEFAULT_REST_URL)]
    pub(crate) rest_url: String,

    #[arg(long)]
    /// The service name that the exploit should target
    pub(crate) service: Option<String>,

    /// The exploit's name
    pub(crate) name: Option<String>,
}
