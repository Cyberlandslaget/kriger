use std::fmt::{Debug};
use std::future::Future;

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
    #[error("serde_json serialization error")]
    SerdeJson(#[from] serde_json::Error),
    #[error("generic error")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub trait Messaging: Clone {
    fn watch_exploit(&self, name: &str) -> impl Future<Output=Result<impl Stream<Item=Result<impl Message<model::Exploit>, MessagingError>>, MessagingError>>;
    
    fn watch_exploits(&self) -> impl Future<Output=Result<impl Stream<Item=Result<impl Message<model::Exploit>, MessagingError>>, MessagingError>>;
}

pub trait Message<T: Sized> {
    fn payload(&self) -> &T;

    /// Acknowledges a message was completely handled.
    fn ack(&self) -> impl Future<Output=Result<(), MessagingError>>;

    /// Signals that the message will not be processed now and processing can move onto the next message, NAK’d message will be retried.
    fn nak(&self) -> impl Future<Output=Result<(), MessagingError>>;

    /// When sent before the AckWait period indicates that work is ongoing and the period should be extended by another equal to AckWait.
    fn progress(&self) -> impl Future<Output=Result<(), MessagingError>>;

    /// Instructs the server to stop redelivery of a message without acknowledging it as successfully processed.
    fn term(&self) -> impl Future<Output=Result<(), MessagingError>>;
}
