use crate::messaging;
use crate::messaging::{Bucket, Message, Messaging, MessagingError};
use crate::utils::data::MapWriter;
use async_nats::jetstream;
use async_nats::jetstream::consumer::{AckPolicy, DeliverPolicy, ReplayPolicy};
use async_nats::jetstream::kv::{CreateErrorKind, Operation, Store};
use async_nats::jetstream::{stream, AckKind, Context};
use async_trait::async_trait;
use dashmap::DashMap;
use futures::{Stream, StreamExt};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::spawn;
use tracing::{debug, info, warn};

const KV_OPERATION: &str = "KV-Operation";
const ALL_KEYS: &str = ">";

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
            messaging::DeliverPolicy::ByStartTime { start_time } => {
                DeliverPolicy::ByStartTime { start_time }
            }
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

        debug!("creating scheduling stream");
        self.context
            .create_stream(stream::Config {
                name: "scheduling".to_string(),
                subjects: vec!["scheduling.>".to_string()],
                discard: stream::DiscardPolicy::Old,
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
        self.context
            .create_key_value(jetstream::kv::Config {
                bucket: "services".to_string(),
                ..Default::default()
            })
            .await?;
        self.context
            .create_key_value(jetstream::kv::Config {
                bucket: "teams".to_string(),
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
        durable_name: Option<String>,
        ack_policy: AckPolicy,
        deliver_policy: DeliverPolicy,
        backoff: Vec<Duration>,
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
                durable_name,
                ack_policy,
                deliver_policy,
                backoff,
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

    async fn list<T>(&self, key_filter: Option<&str>) -> Result<HashMap<String, T>, MessagingError>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        let consumer = self
            .store
            .stream
            .create_consumer(jetstream::consumer::pull::OrderedConfig {
                filter_subject: format!("{}{}", &self.store.prefix, key_filter.unwrap_or(ALL_KEYS)),
                replay_policy: ReplayPolicy::Instant,
                deliver_policy: DeliverPolicy::LastPerSubject,
                ..Default::default()
            })
            .await?;

        let num_pending = consumer.cached_info().num_pending;
        if num_pending == 0 {
            return Ok(HashMap::with_capacity(0));
        }

        let mut stream = consumer.messages().await?;

        let mut map = HashMap::new();
        while let Some(Ok(msg)) = stream.next().await {
            // FIXME: mut is not really required here..
            handle_watch_message(&msg, &mut map, &self.store.prefix);

            let info = msg.info()?;
            debug! {
                pending = info.pending,
                "peeking stream"
            }
            // We've caught up with the history
            if info.pending == 0 {
                break;
            }
        }

        Ok(map)
    }

    async fn watch_key<T>(
        &self,
        key: &str,
        durable_name: Option<String>,
        ack_policy: messaging::AckPolicy,
        deliver_policy: messaging::DeliverPolicy,
        backoff: Vec<Duration>,
    ) -> Result<impl Stream<Item = Result<impl Message<Payload = T>, MessagingError>>, MessagingError>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        self.watch(
            Some(key),
            durable_name,
            ack_policy.into(),
            deliver_policy.into(),
            backoff,
        )
        .await
    }

    async fn watch_all<T>(
        &self,
        durable_name: Option<String>,
        ack_policy: messaging::AckPolicy,
        deliver_policy: messaging::DeliverPolicy,
    ) -> Result<impl Stream<Item = Result<impl Message<Payload = T>, MessagingError>>, MessagingError>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        self.watch(
            None,
            durable_name,
            ack_policy.into(),
            deliver_policy.into(),
            vec![],
        )
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

    async fn subscribe_all<T>(&self) -> Result<Arc<DashMap<String, T>>, MessagingError>
    where
        T: Sized + DeserializeOwned + Send + Sync + 'static,
    {
        let consumer = self
            .store
            .stream
            .create_consumer(jetstream::consumer::pull::OrderedConfig {
                filter_subject: format!("{}>", &self.store.prefix),
                replay_policy: ReplayPolicy::Instant,
                deliver_policy: DeliverPolicy::LastPerSubject,
                ..Default::default()
            })
            .await?;

        let num_pending = consumer.cached_info().num_pending;
        let mut stream = consumer.messages().await?;

        let map = DashMap::<String, T>::new();
        let mut arc = Arc::new(map);

        // Consume all latest messages per key up until the point of the consumer creation.
        // We want to only do this if `num_pending` is greater than zero, otherwise this will wait
        // until the first message arrives.
        if num_pending > 0 {
            debug! {
                pending = num_pending,
                "consuming the initial k/v pairs"
            }
            while let Some(Ok(msg)) = stream.next().await {
                // FIXME: mut is not really required here..
                handle_watch_message(&msg, &mut arc, &self.store.prefix);

                let info = msg.info()?;
                debug! {
                    pending = info.pending,
                    "peeking stream"
                }
                // We've caught up with the history
                if info.pending == 0 {
                    break;
                }
            }
        } else {
            debug!(
                "the consumer does not have any pending messages, skipping the initial population"
            );
        }

        let weak_ref = Arc::downgrade(&arc);

        // TODO: Better error handling? What happens if the subscription drops randomly? Is there a way to recover?
        let prefix = self.store.prefix.clone();
        spawn(async move {
            while weak_ref.strong_count() > 0 {
                // FIXME: This will STILL wait for an element when the weak reference is dropped
                match stream.next().await {
                    Some(Ok(msg)) => {
                        match weak_ref.upgrade() {
                            Some(mut map) => handle_watch_message(&msg, &mut map, &prefix),
                            // There are no strong references anymore, we can stop the subscription
                            None => {
                                debug!("strong references lost");
                                return;
                            }
                        }
                    }
                    Some(Err(err)) => {
                        warn!("subscription error: {err:?}");
                    }
                    // End of stream
                    None => return,
                }
            }
            debug!("strong references lost");
        });

        Ok(arc)
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

    async fn flags(&self) -> Result<impl Bucket + 'static, MessagingError> {
        let store = self.context.get_key_value("flags").await?;
        Ok(NatsBucket { store })
    }

    async fn services(&self) -> Result<impl Bucket, MessagingError> {
        let store = self.context.get_key_value("services").await?;
        Ok(NatsBucket { store })
    }

    async fn teams(&self) -> Result<impl Bucket, MessagingError> {
        let store = self.context.get_key_value("teams").await?;
        Ok(NatsBucket { store })
    }

    async fn executions_wq(&self) -> Result<impl messaging::Stream, MessagingError> {
        let stream = self.context.get_stream("executions_wq").await?;
        Ok(NatsStream { stream })
    }

    async fn scheduling(&self) -> Result<impl messaging::Stream, MessagingError> {
        let stream = self.context.get_stream("scheduling").await?;
        Ok(NatsStream { stream })
    }

    async fn publish<T>(
        &self,
        subject: String,
        payload: &T,
        double_ack: bool,
    ) -> Result<(), MessagingError>
    where
        T: Serialize,
    {
        let serialized_payload = serde_json::to_string(payload)?;
        let fut = self
            .context
            .publish(subject, serialized_payload.into_bytes().into())
            .await?;

        if double_ack {
            fut.await?;
        }

        Ok(())
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

    fn payload(&self) -> &Self::Payload {
        &self.payload
    }

    fn into_payload(self) -> Self::Payload {
        self.payload
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

fn kv_operation_from_message(message: &async_nats::Message) -> Option<Operation> {
    let headers = message.headers.as_ref()?;
    let val = headers.get(KV_OPERATION)?;

    Operation::from_str(val.as_str()).ok()
}

fn handle_watch_message<T>(
    msg: &jetstream::Message,
    map: &mut impl MapWriter<String, T>,
    store_prefix: impl AsRef<str>,
) where
    T: DeserializeOwned,
{
    let operation = kv_operation_from_message(&msg).unwrap_or(Operation::Put);

    let key = msg
        .subject
        .strip_prefix(store_prefix.as_ref())
        .map(|s| s.to_string())
        .unwrap();

    match operation {
        // The put operation will not have any headers in the message
        Operation::Put => match serde_json::from_slice::<T>(msg.payload.as_ref()) {
            Ok(payload) => {
                map.insert(key, payload);
            }
            Err(err) => {
                warn!("malformed message: {err:?}");
            }
        },
        Operation::Delete | Operation::Purge => {
            map.remove(&key);
        }
    }
}
