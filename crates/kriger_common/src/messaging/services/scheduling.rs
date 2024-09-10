use crate::messaging;
use crate::messaging::model;
use crate::messaging::nats::{publish, subscribe_ordered, MessageWrapper, MessagingServiceError};
use async_nats::jetstream;
use async_nats::jetstream::consumer::DeliverPolicy;

const SCHEDULING_TICK_SUBJECT: &str = "scheduling.tick";

pub struct SchedulingService {
    pub(crate) context: jetstream::Context,
    pub(crate) scheduling_stream: jetstream::stream::Stream,
}

impl SchedulingService {
    pub async fn publish_tick(
        &self,
        message: &model::SchedulingTick,
    ) -> Result<jetstream::context::PublishAckFuture, messaging::MessagingError> {
        publish(&self.context, SCHEDULING_TICK_SUBJECT, message).await
    }

    pub async fn subscribe_ticks_ordered(
        &self,
        deliver_policy: DeliverPolicy,
    ) -> Result<
        impl futures::Stream<
            Item = Result<MessageWrapper<model::SchedulingTick>, MessagingServiceError>,
        >,
        messaging::MessagingError,
    > {
        subscribe_ordered(
            &self.scheduling_stream,
            Some(SCHEDULING_TICK_SUBJECT.to_string()),
            deliver_policy,
        )
        .await
    }
}
