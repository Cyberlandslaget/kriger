use clap_derive::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
#[group(skip)]
pub struct Config {
    /// The Kubernetes namespace to schedule exploits in
    #[arg(env, long, default_value = "kriger-exploits")]
    pub controller_exploit_namespace: String,

    /// The NATS service URL to pass to exploit runners
    #[arg(env, long, default_value = "nats://nats:4222")]
    pub controller_nats_svc_url: String,

    /// Allow the controller to set resource limits
    #[arg(env, long, default_value_t = false)]
    pub controller_resource_limits: bool,
}
