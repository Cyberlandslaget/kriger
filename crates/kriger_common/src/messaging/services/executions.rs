use crate::messaging;
use crate::messaging::model;
use crate::messaging::nats::{publish, subscribe, MessageWrapper, MessagingServiceError};
use async_nats::jetstream;

const EXECUTIONS_SUBJECT_PREFIX: &str = "executions.";
const EXECUTION_REQUEST: &str = "request";

pub struct ExecutionsService {
    pub(crate) context: jetstream::Context,
    pub(crate) executions_wq_stream: jetstream::stream::Stream,
}

impl ExecutionsService {
    pub async fn publish_execution_request(
        &self,
        exploit_name: &str,
        message: &model::ExecutionRequest,
    ) -> Result<jetstream::context::PublishAckFuture, messaging::MessagingError> {
        publish(
            &self.context,
            format_request_subject(Some(exploit_name)),
            message,
        )
        .await
    }

    pub async fn subscribe_execution_requests(
        &self,
        durable_name: Option<String>,
        exploit_name: Option<&str>,
    ) -> Result<
        impl futures::Stream<
            Item = Result<MessageWrapper<model::ExecutionRequest>, MessagingServiceError>,
        >,
        messaging::MessagingError,
    > {
        subscribe(
            &self.executions_wq_stream,
            jetstream::consumer::pull::Config {
                durable_name,
                deliver_policy: jetstream::consumer::DeliverPolicy::New,
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                filter_subject: format_request_subject(exploit_name),
                ..Default::default()
            },
        )
        .await
    }
}

#[inline]
fn format_request_subject(exploit_name: Option<&str>) -> String {
    format!(
        "{}{}.{}",
        EXECUTIONS_SUBJECT_PREFIX,
        exploit_name.unwrap_or("*"),
        EXECUTION_REQUEST
    )
}
