use crate::messaging::{model, Message, Messaging, MessagingError};
use async_nats::jetstream;
use async_nats::jetstream::consumer::{AckPolicy, DeliverPolicy, ReplayPolicy};
use async_nats::jetstream::{stream, AckKind, Context};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::de::DeserializeOwned;
use std::time::Duration;
use tracing::{debug, info};

#[derive(Clone)]
pub struct NatsMessaging {
    context: Context,
}

impl NatsMessaging {
    pub async fn new<S: AsRef<str>>(nats_url: S) -> anyhow::Result<Self> {
        Ok(Self {
            context: create_jetstream_context(nats_url).await?,
        })
    }

    pub async fn do_migration(&self) -> anyhow::Result<()> {
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

    async fn subscribe_stream<T>(
        &self,
        stream: stream::Stream,
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
        self.subscribe_stream(
            stream,
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

    async fn watch<T>(
        &self,
        bucket: &str,
        key: Option<&str>,
        deliver_policy: DeliverPolicy,
    ) -> Result<impl Stream<Item = Result<NatsMessage<T>, MessagingError>>, MessagingError>
    where
        T: Sized + DeserializeOwned,
    {
        let store = self.context.get_key_value(bucket).await?;
        self.subscribe_stream(
            store.stream,
            jetstream::consumer::pull::Config {
                // Requires all messages to be individually ACK'd
                ack_policy: AckPolicy::Explicit, // TODO: Make this configurable?
                replay_policy: ReplayPolicy::Instant,
                filter_subject: key
                    .map_or(Default::default(), |key| format!("{}{}", store.prefix, key)),
                deliver_policy,
                ..Default::default()
            },
        )
        .await
    }

    async fn watch_all<T>(
        &self,
        bucket: &str,
    ) -> Result<impl Stream<Item = Result<NatsMessage<T>, MessagingError>>, MessagingError>
    where
        T: Sized + DeserializeOwned,
    {
        self.watch_all_with_deliver_policy(bucket, DeliverPolicy::All)
            .await
    }

    async fn watch_all_with_deliver_policy<T>(
        &self,
        bucket: &str,
        deliver_policy: DeliverPolicy,
    ) -> Result<impl Stream<Item = Result<NatsMessage<T>, MessagingError>>, MessagingError>
    where
        T: Sized + DeserializeOwned,
    {
        self.watch(bucket, None, deliver_policy).await
    }

    async fn watch_key<T>(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<impl Stream<Item = Result<NatsMessage<T>, MessagingError>>, MessagingError>
    where
        T: Sized + DeserializeOwned,
    {
        self.watch_key_with_deliver_policy(bucket, key, DeliverPolicy::Last)
            .await
    }

    async fn watch_key_with_deliver_policy<T>(
        &self,
        bucket: &str,
        key: &str,
        deliver_policy: DeliverPolicy,
    ) -> Result<impl Stream<Item = Result<NatsMessage<T>, MessagingError>>, MessagingError>
    where
        T: Sized + DeserializeOwned,
    {
        self.watch(bucket, Some(key), deliver_policy).await
    }
}

impl Messaging for NatsMessaging {
    async fn watch_exploit(
        &self,
        name: &str,
    ) -> Result<
        impl Stream<Item = Result<impl Message<Payload = model::Exploit>, MessagingError>>,
        MessagingError,
    > {
        self.watch_key("exploits", name).await
    }

    async fn watch_exploits(
        &self,
    ) -> Result<
        impl Stream<Item = Result<impl Message<Payload = model::Exploit>, MessagingError>>,
        MessagingError,
    > {
        self.watch_all("exploits").await
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

async fn create_jetstream_context<S: AsRef<str>>(nats_url: S) -> anyhow::Result<Context> {
    let client = async_nats::connect(nats_url.as_ref()).await?;
    let ctx = jetstream::new(client);
    Ok(ctx)
}
