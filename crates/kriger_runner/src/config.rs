use clap_derive::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
#[group(skip)]
pub struct Config {
    /// The URL to the NATS/JetStream server
    #[arg(env, long, default_value = "nats://127.0.0.1:4222")]
    pub nats_url: String,

    /// The name of the service that the runner will be exploiting
    #[arg(env, long)]
    pub service: Option<String>,

    /// The name of the exploit that the runner will be responsible for
    #[arg(env, long)]
    pub exploit: String,

    /// The maximum amount of workers/executions to handle at any given time. If omitted, the default worker count will be 2*cpu.
    #[arg(env, long)]
    pub workers: Option<usize>,

    /// The timeout, in seconds
    #[arg(env, long, default_value = "30")]
    pub timeout: u64,

    /// The command to execute
    pub command: String,

    /// The arguments to pass to the command
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}
