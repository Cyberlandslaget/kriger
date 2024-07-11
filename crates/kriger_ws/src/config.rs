use clap_derive::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
#[group(skip)]
pub struct Config {
    /// The socket address to listen to
    #[arg(env, long, default_value = "[::]:8001")]
    pub(crate) ws_listen: String,
}
