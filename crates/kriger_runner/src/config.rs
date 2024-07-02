use clap_derive::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
#[group(skip)]
pub struct Config {
    /// The name of the exploit that the runner will be responsible for
    #[arg(env, long)]
    pub runner_exploit: Option<String>,

    /// The maximum amount of workers/executions to handle at any given time. If omitted, the default worker count will be 2*cpu.
    #[arg(env, long)]
    pub runner_workers: Option<usize>,
}