// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use crate::messaging;
use crate::messaging::model;
use crate::messaging::nats::{
    publish, subscribe, subscribe_ordered, MessageWrapper, MessagingServiceError,
};
use async_nats::jetstream;

const SCHEDULING_TICK_SUBJECT: &str = "scheduling.tick";
const SCHEDULING_REQUEST_SUBJECT: &str = "scheduling.request";

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

    pub async fn publish_request(
        &self,
        message: &model::SchedulingRequest,
    ) -> Result<jetstream::context::PublishAckFuture, messaging::MessagingError> {
        publish(&self.context, SCHEDULING_REQUEST_SUBJECT, message).await
    }

    pub async fn subscribe_ticks_ordered(
        &self,
        deliver_policy: jetstream::consumer::DeliverPolicy,
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

    pub async fn subscribe_requests(
        &self,
        durable_name: Option<String>,
    ) -> Result<
        impl futures::Stream<
            Item = Result<MessageWrapper<model::SchedulingRequest>, MessagingServiceError>,
        >,
        messaging::MessagingError,
    > {
        subscribe(
            &self.scheduling_stream,
            jetstream::consumer::pull::Config {
                durable_name,
                deliver_policy: jetstream::consumer::DeliverPolicy::New,
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                filter_subject: SCHEDULING_REQUEST_SUBJECT.to_string(),
                ..Default::default()
            },
        )
        .await
    }
}
