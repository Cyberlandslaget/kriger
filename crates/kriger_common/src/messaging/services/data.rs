// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use crate::messaging;
use crate::messaging::nats::{list_stream, publish_with_id, subscribe};
use crate::messaging::{model, MessagingError};
use crate::messaging::{MessageWrapper, MessagingServiceError};
use async_nats::jetstream;
use base64::Engine;
use std::time::Duration;

const DATA_FLAG_HINTS_SUBJECT_PREFIX: &str = "data.flag_hints.";

pub struct DataService {
    pub(crate) context: jetstream::Context,
    pub(crate) data_stream: jetstream::stream::Stream,
}

impl DataService {
    pub async fn publish_flag_hint(
        &self,
        message: &model::FlagHint,
    ) -> Result<jetstream::context::PublishAckFuture, messaging::MessagingError> {
        let serialized_hint = serde_json::to_string(&message.hint)?;
        let encoded_hint = base64::engine::general_purpose::STANDARD_NO_PAD.encode(serialized_hint);
        let encoded_service =
            base64::engine::general_purpose::STANDARD_NO_PAD.encode(&message.service);

        let id = format!("{}.{}.{}", &encoded_service, &message.team_id, encoded_hint);
        publish_with_id(
            &self.context,
            format_subject(
                Some(encoded_service.as_str()),
                Some(message.team_id.as_str()),
            ),
            id.as_str(),
            message,
        )
        .await
    }

    pub async fn subscribe_flag_hint(
        &self,
        durable_name: Option<String>,
    ) -> Result<
        impl futures::Stream<Item = Result<MessageWrapper<model::FlagHint>, MessagingServiceError>>,
        messaging::MessagingError,
    > {
        subscribe(
            &self.data_stream,
            jetstream::consumer::pull::Config {
                durable_name,
                deliver_policy: jetstream::consumer::DeliverPolicy::New,
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                // TODO: Un-hardcode
                ack_wait: Duration::from_secs(60),
                filter_subject: format_subject(None, None),
                ..Default::default()
            },
        )
        .await
    }

    pub async fn get_flag_hints(
        &self,
        service_name: Option<&str>,
    ) -> Result<Vec<MessageWrapper<model::FlagHint>>, MessagingError> {
        let encoded_service =
            service_name.map(|name| base64::engine::general_purpose::STANDARD_NO_PAD.encode(name));
        list_stream(
            &self.data_stream,
            Some(format_subject(encoded_service.as_deref(), None)),
        )
        .await
    }
}

#[inline]
fn format_subject(encoded_service: Option<&str>, team_id: Option<&str>) -> String {
    format!(
        "{}{}.{}",
        DATA_FLAG_HINTS_SUBJECT_PREFIX,
        encoded_service.unwrap_or("*"),
        team_id.unwrap_or("*")
    )
}
