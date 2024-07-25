use crate::config::Config;
use crate::messaging::nats::NatsMessaging;
use std::sync::Arc;
use tokio::{signal, spawn};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

/// Common state for components
#[derive(Clone)]
pub struct AppRuntime {
    pub config: Arc<Config>,
    pub messaging: Arc<NatsMessaging>,
    pub cancellation_token: CancellationToken,
}

pub fn create_shutdown_cancellation_token() -> CancellationToken {
    let cancellation_token = CancellationToken::new();
    let signal_cancellation_token = cancellation_token.clone();
    spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                signal_cancellation_token.cancel();
                info!("shutdown signal received");
            }
            Err(error) => {
                error! {
                    ?error,
                    "unable to listen for shutdown signal"
                }
            }
        }
    });
    return cancellation_token;
}
