use crate::messaging;
use crate::messaging::{model, Bucket, Message, Messaging, MessagingError};
use async_nats::jetstream;
use async_nats::jetstream::consumer::{AckPolicy, DeliverPolicy, ReplayPolicy};
use async_nats::jetstream::kv::Store;
use async_nats::jetstream::{stream, AckKind, Context};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::de::DeserializeOwned;
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
                bucket: "exploits".to_string(),
                ..Default::default()
            })
            .await?;

        info!("nats migration complete");
        Ok(())
    }

    async fn subscribe<T>(
        &self,
        stream_name: &str,
        durable_name: Option<String>,
        filter_subject: Option<String>,
    ) -> Result<impl Stream<Item = Result<NatsMessage<T>, MessagingError>>, MessagingError>
    where
        T: Sized + DeserializeOwned,
    {
        let stream = self.context.get_stream(stream_name).await?;
        subscribe_stream(
            &stream,
            jetstream::consumer::pull::Config {
                durable_name,
                // Requires all messages to be individually ACK'd
                ack_policy: AckPolicy::Explicit, // TODO: Make this configurable?
                replay_policy: ReplayPolicy::Instant,
                filter_subject: filter_subject.unwrap_or_default(),
                deliver_policy: DeliverPolicy::New,
                ..Default::default()
            },
        )
        .await
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
}

impl Messaging for NatsMessaging {
    async fn exploits(&self) -> Result<impl Bucket, MessagingError> {
        let store = self.context.get_key_value("exploits").await?;
        Ok(NatsBucket { store })
    }

    async fn subscribe_execution_requests(
        &self,
        exploit_name: &str,
    ) -> Result<
        impl Stream<Item = Result<impl Message<Payload = model::ExecutionRequest>, MessagingError>>,
        MessagingError,
    > {
        self.subscribe(
            "executions_wq",
            Some(format!("exploit:{exploit_name}")),
            Some(format!("executions.{exploit_name}.request")),
        )
        .await
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
    // message once the AckWait period has been exceeded.
    Ok(messages.map(|res| NatsMessage::<T>::from(res?)))
}
