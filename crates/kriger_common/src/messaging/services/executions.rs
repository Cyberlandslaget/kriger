use crate::messaging;
use crate::messaging::model;
use crate::messaging::nats::{
    publish, subscribe, subscribe_ordered, MessageWrapper, MessagingServiceError,
};
use async_nats::jetstream;

const EXECUTIONS_SUBJECT_PREFIX: &str = "executions.";
const EXECUTION_REQUEST: &str = "request";
const EXECUTION_RESULT: &str = "result";

pub struct ExecutionsService {
    pub(crate) context: jetstream::Context,
    pub(crate) executions_wq_stream: jetstream::stream::Stream,
    pub(crate) executions_stream: jetstream::stream::Stream,
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

    pub async fn publish_execution_result(
        &self,
        exploit_name: &str,
        message: &model::ExecutionResult,
    ) -> Result<jetstream::context::PublishAckFuture, messaging::MessagingError> {
        publish(
            &self.context,
            format_result_subject(Some(exploit_name)),
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

    pub async fn subscribe_execution_requests_ordered(
        &self,
        exploit_name: Option<&str>,
        deliver_policy: jetstream::consumer::DeliverPolicy,
    ) -> Result<
        impl futures::Stream<
                Item = Result<MessageWrapper<model::ExecutionRequest>, MessagingServiceError>,
            > + Sized,
        messaging::MessagingError,
    > {
        subscribe_ordered(
            &self.executions_wq_stream,
            Some(format_request_subject(exploit_name)),
            deliver_policy,
        )
        .await
    }

    pub async fn subscribe_execution_results_ordered(
        &self,
        exploit_name: Option<&str>,
        deliver_policy: jetstream::consumer::DeliverPolicy,
    ) -> Result<
        impl futures::Stream<
                Item = Result<MessageWrapper<model::ExecutionResult>, MessagingServiceError>,
            > + Sized,
        messaging::MessagingError,
    > {
        subscribe_ordered(
            &self.executions_stream,
            Some(format_result_subject(exploit_name)),
            deliver_policy,
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

#[inline]
fn format_result_subject(exploit_name: Option<&str>) -> String {
    format!(
        "{}{}.{}",
        EXECUTIONS_SUBJECT_PREFIX,
        exploit_name.unwrap_or("*"),
        EXECUTION_RESULT
    )
}
