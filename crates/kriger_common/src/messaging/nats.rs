use std::time::Duration;
use async_nats::jetstream;
use async_nats::jetstream::{AckKind, Context, stream};
use async_nats::jetstream::consumer::{DeliverPolicy, ReplayPolicy};
use futures::{Stream, StreamExt};
use serde::de::DeserializeOwned;
use tracing::{debug, info};
use crate::messaging::{Message, Messaging, MessagingError, model};

#[derive(Clone)]
pub struct NatsMessaging {
    context: Context,
}

impl NatsMessaging {
    pub async fn new<S: AsRef<str>>(nats_url: S) -> anyhow::Result<Self> {
        Ok(Self {
            context: create_jetstream_context(nats_url).await?
        })
    }

    pub async fn do_migration(&self) -> anyhow::Result<()> {
        info!("creating jetstream streams");

        debug!("creating execution_schedule stream");
        self.context.create_stream(stream::Config {
            name: "execution_schedule".to_string(),
            retention: stream::RetentionPolicy::WorkQueue,
            subjects: vec!["execution_schedule.>".to_string()],
            discard: stream::DiscardPolicy::Old,
            duplicate_window: Duration::from_secs(6 * 60), // TODO: Use data from config
            max_age: Duration::from_secs(6 * 60), // TODO: Use data from config
            ..Default::default()
        }).await?;

        info!("creating kev/value buckets");

        debug!("creating exploits bucket");
        self.context.create_key_value(jetstream::kv::Config {
            bucket: "exploits".to_string(),
            ..Default::default()
        }).await?;

        info!("nats migration complete");
        Ok(())
    }

    async fn watch<T>(&self, bucket: &str, key: Option<&str>, deliver_policy: DeliverPolicy) -> Result<impl Stream<Item=Result<NatsMessage<T>, MessagingError>>, MessagingError>
    where
        T: Sized + DeserializeOwned,
    {
        let store = self.context.get_key_value(bucket).await?;

        let consumer = store.stream.create_consumer(
            jetstream::consumer::pull::OrderedConfig {
                replay_policy: ReplayPolicy::Instant,
                filter_subject: key.map_or(Default::default(), |key| format!("{}{}", store.prefix, key)),
                deliver_policy,
                ..Default::default()
            }
        ).await?;
        let messages = consumer.messages().await?;
        Ok(messages.map(|res| {
            NatsMessage::<T>::from(res?)
        }))
    }

    async fn watch_all<T>(&self, bucket: &str) -> Result<impl Stream<Item=Result<NatsMessage<T>, MessagingError>>, MessagingError>
    where
        T: Sized + DeserializeOwned,
    {
        self.watch_all_with_deliver_policy(bucket, DeliverPolicy::All).await
    }

    async fn watch_all_with_deliver_policy<T>(&self, bucket: &str, deliver_policy: DeliverPolicy) -> Result<impl Stream<Item=Result<NatsMessage<T>, MessagingError>>, MessagingError>
    where
        T: Sized + DeserializeOwned,
    {
        self.watch(bucket, None, deliver_policy).await
    }

    async fn watch_key<T>(&self, bucket: &str, key: &str) -> Result<impl Stream<Item=Result<NatsMessage<T>, MessagingError>>, MessagingError>
    where
        T: Sized + DeserializeOwned,
    {
        self.watch_key_with_deliver_policy(bucket, key, DeliverPolicy::Last).await
    }

    async fn watch_key_with_deliver_policy<T>(&self, bucket: &str, key: &str, deliver_policy: DeliverPolicy) -> Result<impl Stream<Item=Result<NatsMessage<T>, MessagingError>>, MessagingError>
    where
        T: Sized + DeserializeOwned,
    {
        self.watch(bucket, Some(key), deliver_policy).await
    }
}

impl Messaging for NatsMessaging {
    async fn watch_exploit(&self, name: &str) -> Result<impl Stream<Item=Result<impl Message<model::Exploit>, MessagingError>>, MessagingError> {
        self.watch_key("exploits", name).await
    }

    async fn watch_exploits(&self) -> Result<impl Stream<Item=Result<impl Message<model::Exploit>, MessagingError>>, MessagingError> {
        self.watch_all("exploits").await
    }
}

pub struct NatsMessage<T: DeserializeOwned> {
    payload: T,
    message: jetstream::Message,
}

impl<T: DeserializeOwned> NatsMessage<T> {
    pub fn from(message: jetstream::Message) -> Result<Self, MessagingError> {
        let payload = serde_json::from_slice(message.payload.as_ref())?;

        Ok(Self { payload, message })
    }
}

impl<T: DeserializeOwned> Message<T> for NatsMessage<T> {
    fn payload(&self) -> &T {
        &self.payload
    }

    async fn ack(&self) -> Result<(), MessagingError> {
        self.message.ack().await?;
        Ok(())
    }

    async fn nak(&self) -> Result<(), MessagingError> {
        self.message.ack_with(AckKind::Nak(None)).await?;
        Ok(())
    }

    async fn progress(&self) -> Result<(), MessagingError> {
        self.message.ack_with(AckKind::Progress).await?;
        Ok(())
    }

    async fn term(&self) -> Result<(), MessagingError> {
        self.message.ack_with(AckKind::Term).await?;
        Ok(())
    }
}

async fn create_jetstream_context<S: AsRef<str>>(nats_url: S) -> anyhow::Result<Context> {
    let client = async_nats::connect(nats_url.as_ref()).await?;
    let ctx = jetstream::new(client);
    Ok(ctx)
}

