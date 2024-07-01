use clap::Args;

#[derive(Args, Debug)]
#[group(skip)]
pub struct Config {
    /// The URL to the NATS/JetStream server
    #[arg(env, long, default_value = "nats://127.0.0.1:4222")]
    pub nats_url: String,
}