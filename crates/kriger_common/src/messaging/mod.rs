use std::fmt::{Debug};
use std::future::Future;
use async_trait::async_trait;

use futures::Stream;

pub mod model;
pub mod nats;

#[derive(thiserror::Error, Debug)]
pub enum MessagingError {
    #[error("nats kv error")]
    NatsKeyValueError(#[from] async_nats::jetstream::context::KeyValueError),
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
    #[error("serde_json serialization error")]
    SerdeJson(#[from] serde_json::Error),
    #[error("generic error")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub trait Messaging: Clone {
    fn watch_exploit(&self, name: &str) -> impl Future<Output=Result<impl Stream<Item=Result<impl Message<Payload=model::Exploit>, MessagingError>>, MessagingError>>;

    fn watch_exploits(&self) -> impl Future<Output=Result<impl Stream<Item=Result<impl Message<Payload=model::Exploit>, MessagingError>>, MessagingError>>;

    fn subscribe_execution_requests(&self, exploit_name: &str) -> impl Future<Output=Result<impl Stream<Item=Result<impl Message<Payload=model::ExecutionRequest>, MessagingError>>, MessagingError>>;
}

#[async_trait]
pub trait Message {
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
