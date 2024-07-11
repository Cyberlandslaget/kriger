use crate::config::Config;
use crate::messaging::nats::NatsMessaging;
use std::sync::Arc;

/// Common state for components
#[derive(Clone)]
pub struct AppRuntime {
    pub config: Arc<Config>,
    pub messaging: Arc<NatsMessaging>,
}
