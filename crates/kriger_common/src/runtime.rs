use std::sync::Arc;
use crate::config::Config;
use crate::messaging::nats::NatsMessaging;

/// Common state for components
#[derive(Clone)]
pub struct AppRuntime {
    pub config: Arc<Config>,
    pub messaging: Arc<NatsMessaging>,
}