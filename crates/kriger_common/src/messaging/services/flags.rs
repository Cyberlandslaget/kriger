use crate::messaging;
use crate::messaging::model;
use crate::messaging::nats::{
    fetch, publish_with_id, subscribe, subscribe_ordered, Fetcher, MessageWrapper,
    MessagingServiceError,
};
use async_nats::jetstream;
use base64::Engine;
use futures::Stream;
use std::time::Duration;

const SUBMISSIONS_SUBJECT_PREFIX: &str = "flags.submit.";
const SUBMISSION_RESULTS_SUBJECT: &str = "flags.result";

#[derive(Clone)]
pub struct FlagsService {
    pub(crate) context: jetstream::Context,
    pub(crate) flags_submissions_stream: jetstream::stream::Stream,
    pub(crate) flags_results_stream: jetstream::stream::Stream,
}

impl FlagsService {
    pub async fn submit_flag(
        &self,
        message: &model::FlagSubmission,
    ) -> Result<jetstream::context::PublishAckFuture, messaging::MessagingError> {
        let encoded_flag =
            base64::engine::general_purpose::STANDARD_NO_PAD.encode(message.flag.as_bytes());
        publish_with_id(
            &self.context,
            format!("{}{}", SUBMISSIONS_SUBJECT_PREFIX, encoded_flag),
            message.flag.as_str(),
            message,
        )
        .await
    }

    pub async fn submit_flag_result(
        &self,
        message: &model::FlagSubmissionResult,
    ) -> Result<jetstream::context::PublishAckFuture, messaging::MessagingError> {
        publish_with_id(
            &self.context,
            SUBMISSION_RESULTS_SUBJECT,
            message.flag.as_str(),
            message,
        )
        .await
    }

    pub async fn subscribe_submissions(
        &self,
        durable_name: Option<String>,
    ) -> Result<
        impl Stream<Item = Result<MessageWrapper<model::FlagSubmission>, MessagingServiceError>> + Sized,
        messaging::MessagingError,
    > {
        subscribe(
            &self.flags_submissions_stream,
            jetstream::consumer::pull::Config {
                durable_name,
                deliver_policy: jetstream::consumer::DeliverPolicy::New,
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                filter_subject: format!("{}*", SUBMISSIONS_SUBJECT_PREFIX),
                ..Default::default()
            },
        )
        .await
    }

    pub async fn subscribe_submissions_ordered(
        &self,
        deliver_policy: jetstream::consumer::DeliverPolicy,
    ) -> Result<
        impl Stream<Item = Result<MessageWrapper<model::FlagSubmission>, MessagingServiceError>> + Sized,
        messaging::MessagingError,
    > {
        subscribe_ordered(
            &self.flags_submissions_stream,
            Some(format!("{}*", SUBMISSIONS_SUBJECT_PREFIX)),
            deliver_policy,
        )
        .await
    }

    pub async fn subscribe_submission_results_ordered(
        &self,
        deliver_policy: jetstream::consumer::DeliverPolicy,
    ) -> Result<
        impl Stream<
                Item = Result<MessageWrapper<model::FlagSubmissionResult>, MessagingServiceError>,
            > + Sized,
        messaging::MessagingError,
    > {
        subscribe_ordered(
            &self.flags_results_stream,
            Some(SUBMISSION_RESULTS_SUBJECT.to_string()),
            deliver_policy,
        )
        .await
    }

    pub async fn submissions_fetcher(
        &self,
        durable_name: Option<String>,
        limit: usize,
    ) -> Result<impl Fetcher<model::FlagSubmission>, messaging::MessagingError> {
        let consumer_config = jetstream::consumer::pull::Config {
            deliver_policy: jetstream::consumer::DeliverPolicy::New,
            ack_policy: jetstream::consumer::AckPolicy::Explicit,
            ack_wait: Duration::from_secs(60),
            filter_subject: format!("{}*", SUBMISSIONS_SUBJECT_PREFIX),
            replay_policy: Default::default(),
            metadata: Default::default(),
            durable_name,
            ..Default::default()
        };
        fetch(&self.flags_submissions_stream, consumer_config, limit).await
    }
}
