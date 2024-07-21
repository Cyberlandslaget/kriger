use clap_derive::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
#[group(skip)]
pub struct Config {
    /// The URL to the NATS/JetStream server
    #[arg(env, long, default_value = "nats://127.0.0.1:4222")]
    pub nats_url: String,

    /// The socket address to listen to
    #[arg(env, long, default_value = "[::]:8000")]
    pub(crate) rest_listen: String,
}
