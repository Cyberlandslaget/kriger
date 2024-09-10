use crate::messaging::nats::{MessageWrapper, MessagingServiceError};
use async_nats::jetstream;
use dashmap::DashMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

pub mod model;
pub mod nats;
pub mod services;

#[derive(thiserror::Error, Debug)]
pub enum MessagingError {
    #[error("nats kv error")]
    NatsKeyValueError(#[from] jetstream::context::KeyValueError),
    #[error("nats kv entry error")]
    NatsKeyValueEntryError(#[from] jetstream::kv::EntryError),
    #[error("nats kv put error")]
    NatsKeyValuePutError(#[from] jetstream::kv::PutError),
    #[error("nats kv create error")]
    NatsKeyValueCreateError(#[from] jetstream::kv::CreateError),
    #[error("nats consumer error")]
    NatsConsumerError(#[from] jetstream::stream::ConsumerError),
    #[error("nats stream error")]
    NatsStreamError(#[from] jetstream::consumer::StreamError),
    #[error("nats messages error")]
    NatsMessagesError(#[from] jetstream::consumer::pull::MessagesError),
    #[error("nats ordered error")]
    NatsOrderedError(#[from] jetstream::consumer::pull::OrderedError),
    #[error("nats watch error")]
    NatsWatchError(#[from] jetstream::kv::WatchError),
    #[error("nats watcher error")]
    NatsWatcherError(#[from] jetstream::kv::WatcherError),
    #[error("nats get stream error")]
    NatsGetStreamError(#[from] jetstream::context::GetStreamError),
    /// Some key/value operations will return this error. E.g. upon calling create on an element
    /// that already exists due to optimistic concurrency control.
    #[error("nats jetstream publish error")]
    NatsJetStreamPublishError(#[from] jetstream::context::PublishError),
    #[error("nats request error")]
    NatsRequestError(#[from] jetstream::context::RequestError),
    #[error("nats batch error")]
    NatsBatchError(#[from] jetstream::consumer::pull::BatchError),
    #[error("key value conflict error")]
    KeyValueConflictError,
    #[error("serde_json serialization error")]
    SerdeJson(#[from] serde_json::Error),
    #[error("generic error")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub trait Bucket<T>: Clone + 'static
where
    T: DeserializeOwned + Serialize + Send + Sync + 'static,
{
    fn get(&self, key: &str) -> impl Future<Output = Result<Option<T>, MessagingError>> + Send;

    fn list(
        &self,
        key_filter: Option<&str>,
    ) -> impl Future<Output = Result<HashMap<String, T>, MessagingError>> + Send;

    fn watch_key(
        &self,
        key: &str,
        durable_name: Option<String>,
        ack_policy: jetstream::consumer::AckPolicy,
        ack_wait: Duration,
        deliver_policy: jetstream::consumer::DeliverPolicy,
    ) -> impl Future<
        Output = Result<
            impl futures::Stream<Item = Result<MessageWrapper<T>, MessagingServiceError>> + Send,
            MessagingError,
        >,
    > + Send;

    fn watch_all(
        &self,
        durable_name: Option<String>,
        ack_policy: jetstream::consumer::AckPolicy,
        ack_wait: Duration,
        deliver_policy: jetstream::consumer::DeliverPolicy,
    ) -> impl Future<
        Output = Result<
            impl futures::Stream<Item = Result<MessageWrapper<T>, MessagingServiceError>> + Send,
            MessagingError,
        >,
    > + Send;

    fn put(&self, key: &str, body: &T) -> impl Future<Output = Result<(), MessagingError>> + Send;

    fn create(
        &self,
        key: &str,
        body: &T,
    ) -> impl Future<Output = Result<(), MessagingError>> + Send;

    /// Subscribes to all entries in bucket. The subscription will be spawned using [spawn]
    /// and will be cancelled once the [Arc] is dropped.
    fn subscribe_all(
        &self,
    ) -> impl Future<Output = Result<Arc<DashMap<String, T>>, MessagingError>> + Send;
}
