#[cfg(debug_assertions)]
const DEFAULT_NATS_URL: &str = "nats://127.0.0.1:4222";
#[cfg(not(debug_assertions))]
const DEFAULT_NATS_URL: &str = "nats://nats:4222";

#[derive(clap::Args, Debug)]
#[group(skip)]
pub struct RuntimeArgs {
    /// The URL to the NATS/JetStream server
    #[arg(env, long, default_value = DEFAULT_NATS_URL)]
    pub nats_url: String,
}
