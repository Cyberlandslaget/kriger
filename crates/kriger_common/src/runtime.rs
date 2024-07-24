use crate::config::Config;
use crate::messaging::nats::NatsMessaging;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Common state for components
#[derive(Clone)]
pub struct AppRuntime {
    pub config: Arc<Config>,
    pub messaging: Arc<NatsMessaging>,
    pub cancellation_token: CancellationToken,
}
