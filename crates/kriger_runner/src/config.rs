use clap_derive::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
#[group(skip)]
pub struct Config {
    /// The name of the exploit that the runner will be responsible for
    #[arg(env, long)]
    pub runner_exploit: Option<String>,

    /// The command to execute the exploit, without arguments
    #[arg(env, long)]
    pub runner_exploit_command: Option<String>,

    /// The arguments to pass to the exploit command
    #[arg(env, long)]
    pub runner_exploit_args: Option<String>,

    /// The maximum amount of workers/executions to handle at any given time. If omitted, the default worker count will be 2*cpu.
    #[arg(env, long)]
    pub runner_workers: Option<usize>,
}
