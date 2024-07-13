use crate::messaging;
use crate::messaging::{Bucket, Message, Messaging, MessagingError};
use async_nats::jetstream;
use async_nats::jetstream::consumer::{AckPolicy, DeliverPolicy, ReplayPolicy};
use async_nats::jetstream::kv::{CreateErrorKind, Store};
use async_nats::jetstream::{stream, AckKind, Context};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
use tracing::{debug, info};

impl Into<AckPolicy> for messaging::AckPolicy {
    fn into(self) -> AckPolicy {
        match self {
            messaging::AckPolicy::Explicit => AckPolicy::Explicit,
            messaging::AckPolicy::None => AckPolicy::None,
        }
    }
}

impl Into<DeliverPolicy> for messaging::DeliverPolicy {
    fn into(self) -> DeliverPolicy {
        match self {
            messaging::DeliverPolicy::All => DeliverPolicy::All,
            messaging::DeliverPolicy::Last => DeliverPolicy::Last,
            messaging::DeliverPolicy::New => DeliverPolicy::New,
            messaging::DeliverPolicy::LastPerSubject => DeliverPolicy::LastPerSubject,
        }
    }
}

#[derive(Clone)]
pub struct NatsMessaging {
    context: Context,
}

impl NatsMessaging {
    pub async fn new<S: AsRef<str>>(nats_url: S) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            context: create_jetstream_context(nats_url).await?,
        })
    }

    pub async fn do_migration(&self) -> color_eyre::eyre::Result<()> {
        info!("creating jetstream streams");

        debug!("creating executions_wq stream");

        self.context
            .create_stream(stream::Config {
                name: "executions_wq".to_string(),
                subjects: vec!["executions.*.request".to_string()],
                discard: stream::DiscardPolicy::Old,
                // Important: this will provide idempotency for execution requests
                duplicate_window: Duration::from_secs(6 * 60), // TODO: Use data from config
                max_age: Duration::from_secs(6 * 60),          // TODO: Use data from config
                ..Default::default()
            })
            .await?;

        info!("creating kev/value buckets");

        debug!("creating exploits bucket");
        self.context
            .create_key_value(jetstream::kv::Config {
                bucket: "config".to_string(),
                ..Default::default()
            })
            .await?;
        self.context
            .create_key_value(jetstream::kv::Config {
                bucket: "exploits".to_string(),
                ..Default::default()
            })
            .await?;
        self.context
            .create_key_value(jetstream::kv::Config {
                bucket: "flags".to_string(),
                ..Default::default()
            })
            .await?;

        info!("nats migration complete");
        Ok(())
    }
}

#[derive(Clone)]
struct NatsBucket {
    store: Store,
}

impl NatsBucket {
    async fn watch<T>(
        &self,
        key: Option<&str>,
        ack_policy: AckPolicy,
        deliver_policy: DeliverPolicy,
    ) -> Result<impl Stream<Item = Result<NatsMessage<T>, MessagingError>>, MessagingError>
    where
        T: Sized + DeserializeOwned,
    {
        subscribe_stream(
            &self.store.stream,
            jetstream::consumer::pull::Config {
                replay_policy: ReplayPolicy::Instant,
                filter_subject: key.map_or(Default::default(), |key| {
                    format!("{}{}", &self.store.prefix, key)
                }),
                ack_policy,
                deliver_policy,
                ..Default::default()
            },
        )
        .await
    }
}

impl Bucket for NatsBucket {
    async fn get<T>(&self, key: &str) -> Result<Option<T>, MessagingError>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        match self.store.get(key).await? {
            Some(bytes) => Ok(serde_json::from_slice(bytes.as_ref())?),
            None => Ok(None),
        }
    }

    async fn watch_key<T>(
        &self,
        key: &str,
        ack_policy: messaging::AckPolicy,
        deliver_policy: messaging::DeliverPolicy,
    ) -> Result<impl Stream<Item = Result<impl Message<Payload = T>, MessagingError>>, MessagingError>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        self.watch(Some(key), ack_policy.into(), deliver_policy.into())
            .await
    }

    async fn watch_all<T>(
        &self,
        ack_policy: messaging::AckPolicy,
        deliver_policy: messaging::DeliverPolicy,
    ) -> Result<impl Stream<Item = Result<impl Message<Payload = T>, MessagingError>>, MessagingError>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        self.watch(None, ack_policy.into(), deliver_policy.into())
            .await
    }

    async fn put<T>(&self, key: &str, body: &T) -> Result<(), MessagingError>
    where
        T: Serialize + Send + Sync,
    {
        self.store
            .put(key, serde_json::to_vec(body)?.into())
            .await?;
        Ok(())
    }

    async fn create<T>(&self, key: &str, body: &T) -> Result<(), MessagingError>
    where
        T: Serialize + Send + Sync,
    {
        let res = self
            .store
            .create(key, serde_json::to_vec(body)?.into())
            .await;
        if let Err(err) = res {
            return match err.kind() {
                CreateErrorKind::AlreadyExists => Err(MessagingError::KeyValueConflictError),
                _ => Err(err.into()),
            };
        }
        Ok(())
    }
}

#[derive(Clone)]
struct NatsStream {
    stream: stream::Stream,
}

impl messaging::Stream for NatsStream {
    async fn subscribe<T>(
        &self,
        durable_name: Option<String>,
        filter_subject: Option<String>,
        ack_policy: messaging::AckPolicy,
        deliver_policy: messaging::DeliverPolicy,
    ) -> Result<impl Stream<Item = Result<NatsMessage<T>, MessagingError>>, MessagingError>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        subscribe_stream(
            &self.stream,
            jetstream::consumer::pull::Config {
                durable_name,
                replay_policy: ReplayPolicy::Instant,
                filter_subject: filter_subject.unwrap_or_default(),
                ack_policy: ack_policy.into(),
                deliver_policy: deliver_policy.into(),
                ..Default::default()
            },
        )
        .await
    }
}

impl Messaging for NatsMessaging {
    async fn config(&self) -> Result<impl Bucket, MessagingError> {
        let store = self.context.get_key_value("config").await?;
        Ok(NatsBucket { store })
    }

    async fn exploits(&self) -> Result<impl Bucket, MessagingError> {
        let store = self.context.get_key_value("exploits").await?;
        Ok(NatsBucket { store })
    }

    async fn flags(&self) -> Result<impl Bucket, MessagingError> {
        let store = self.context.get_key_value("flags").await?;
        Ok(NatsBucket { store })
    }

    async fn executions_wq(&self) -> Result<impl messaging::Stream, MessagingError> {
        let stream = self.context.get_stream("executions_wq").await?;
        Ok(NatsStream { stream })
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

// The usage of async_trait is required here to make the trait object-safe.
#[async_trait]
impl<T: DeserializeOwned + Send + Sync + 'static> Message for NatsMessage<T> {
    type Payload = T;

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

async fn create_jetstream_context<S: AsRef<str>>(nats_url: S) -> color_eyre::eyre::Result<Context> {
    let client = async_nats::connect(nats_url.as_ref()).await?;
    let ctx = jetstream::new(client);
    Ok(ctx)
}

async fn subscribe_stream<T>(
    stream: &stream::Stream,
    consumer_config: jetstream::consumer::pull::Config,
) -> Result<impl Stream<Item = Result<NatsMessage<T>, MessagingError>>, MessagingError>
where
    T: Sized + DeserializeOwned,
{
    let consumer = stream.create_consumer(consumer_config).await?;
    let messages = consumer.messages().await?;
    // TODO: Nak/Term messages that failed to parse
    // This shouldn't really happen though. Regardless, NATS will automatically redeliver the
    // message once the AckWait period has been exceeded. However, these messages can overwhelm
    // a consumer (by exceeding its pending messages limit) which will cause it to hang until the
    // AckWait period has been exceeded.
    Ok(messages.map(|res| NatsMessage::<T>::from(res?)))
}
