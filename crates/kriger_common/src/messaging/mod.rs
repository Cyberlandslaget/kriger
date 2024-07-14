use std::fmt::Debug;
use std::future::Future;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::messaging::nats::NatsMessage;

pub mod model;
pub mod nats;

pub enum AckPolicy {
    Explicit,
    None,
}

pub enum DeliverPolicy {
    All,
    Last,
    New,
    LastPerSubject,
}

#[derive(thiserror::Error, Debug)]
pub enum MessagingError {
    #[error("nats kv error")]
    NatsKeyValueError(#[from] async_nats::jetstream::context::KeyValueError),
    #[error("nats kv entry error")]
    NatsKeyValueEntryError(#[from] async_nats::jetstream::kv::EntryError),
    #[error("nats kv put error")]
    NatsKeyValuePutError(#[from] async_nats::jetstream::kv::PutError),
    #[error("nats kv create error")]
    NatsKeyValueCreateError(#[from] async_nats::jetstream::kv::CreateError),
    #[error("nats consumer error")]
    NatsConsumerError(#[from] async_nats::jetstream::stream::ConsumerError),
    #[error("nats stream error")]
    NatsStreamError(#[from] async_nats::jetstream::consumer::StreamError),
    #[error("nats messages error")]
    NatsMessagesError(#[from] async_nats::jetstream::consumer::pull::MessagesError),
    #[error("nats ordered error")]
    NatsOrderedError(#[from] async_nats::jetstream::consumer::pull::OrderedError),
    #[error("nats watch error")]
    NatsWatchError(#[from] async_nats::jetstream::kv::WatchError),
    #[error("nats watcher error")]
    NatsWatcherError(#[from] async_nats::jetstream::kv::WatcherError),
    #[error("nats get stream error")]
    NatsGetStreamError(#[from] async_nats::jetstream::context::GetStreamError),
    /// Some key/value operations will return this error. E.g. upon calling create on an element
    /// that already exists due to optimistic concurrency control.
    #[error("key value conflict error")]
    KeyValueConflictError,
    #[error("serde_json serialization error")]
    SerdeJson(#[from] serde_json::Error),
    #[error("generic error")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub trait Messaging: Clone {
    fn config(&self) -> impl Future<Output = Result<impl Bucket, MessagingError>>;

    fn exploits(&self) -> impl Future<Output = Result<impl Bucket, MessagingError>>;

    fn flags(&self) -> impl Future<Output = Result<impl Bucket, MessagingError>>;

    fn executions_wq(&self) -> impl Future<Output = Result<impl Stream, MessagingError>>;
}

pub trait Bucket: Clone + 'static {
    fn get<T>(&self, key: &str) -> impl Future<Output = Result<Option<T>, MessagingError>> + Send
    where
        T: DeserializeOwned + Send + Sync + 'static;

    fn watch_key<T>(
        &self,
        key: &str,
        ack_policy: AckPolicy,
        deliver_policy: DeliverPolicy,
    ) -> impl Future<
        Output = Result<
            impl futures::Stream<Item = Result<impl Message<Payload = T>, MessagingError>> + Send,
            MessagingError,
        >,
    > + Send
    where
        T: DeserializeOwned + Send + Sync + 'static;

    fn watch_all<T>(
        &self,
        ack_policy: AckPolicy,
        deliver_policy: DeliverPolicy,
    ) -> impl Future<
        Output = Result<
            impl futures::Stream<Item = Result<impl Message<Payload = T>, MessagingError>> + Send,
            MessagingError,
        >,
    > + Send
    where
        T: DeserializeOwned + Send + Sync + 'static;

    fn put<T>(
        &self,
        key: &str,
        body: &T,
    ) -> impl Future<Output = Result<(), MessagingError>> + Send
    where
        T: Serialize + Send + Sync;

    fn create<T>(
        &self,
        key: &str,
        body: &T,
    ) -> impl Future<Output = Result<(), MessagingError>> + Send
    where
        T: Serialize + Send + Sync;
}

pub trait Stream: Clone {
    fn subscribe<T>(
        &self,
        durable_name: Option<String>,
        filter_subject: Option<String>,
        ack_policy: AckPolicy,
        deliver_policy: DeliverPolicy,
    ) -> impl Future<
        Output = Result<
            impl futures::Stream<Item = Result<NatsMessage<T>, MessagingError>> + Send,
            MessagingError,
        >,
    > + Send
    where
        T: DeserializeOwned + Send + Sync + 'static;
}

// Assuming that this trait can be 'static. If not, remove bound here and fix lifetime issues in kriger_runner::main.
#[async_trait]
pub trait Message: Send + 'static {
    type Payload: Send;

    fn payload(&self) -> &Self::Payload;

    /// Acknowledges a message was completely handled.
    async fn ack(&self) -> Result<(), MessagingError>;

    /// Signals that the message will not be processed now and processing can move onto the next message, NAK’d message will be retried.
    async fn nak(&self) -> Result<(), MessagingError>;

    /// When sent before the AckWait period indicates that work is ongoing and the period should be extended by another equal to AckWait.
    async fn progress(&self) -> Result<(), MessagingError>;

    /// Instructs the server to stop redelivery of a message without acknowledging it as successfully processed.
    async fn term(&self) -> Result<(), MessagingError>;
}
