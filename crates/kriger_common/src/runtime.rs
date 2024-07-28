use crate::config::Config;
use crate::messaging::nats::NatsMessaging;
use futures::future::select_all;
use futures::FutureExt;
use std::sync::Arc;
use tokio::signal::unix::SignalKind;
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
        // TODO: Support Windows?
        let mut signals: Vec<signal::unix::Signal> = [
            signal::unix::signal(SignalKind::terminate()),
            signal::unix::signal(SignalKind::interrupt()),
        ]
        .into_iter()
        .filter_map(|maybe_signal| match maybe_signal {
            Ok(signal) => Some(signal),
            Err(error) => {
                error! {
                    ?error,
                    "unable to listen for shutdown signal"
                }
                None
            }
        })
        .collect();

        let signal_futures = signals.iter_mut().map(|signal| signal.recv().boxed());
        select_all(signal_futures).await;

        signal_cancellation_token.cancel();
        info!("shutdown signal received");
    });
    return cancellation_token;
}
