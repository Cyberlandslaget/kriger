use crate::messaging::services::data::DataService;
use crate::messaging::services::executions::ExecutionsService;
use crate::messaging::services::flags::FlagsService;
use crate::messaging::services::scheduling::SchedulingService;
use crate::messaging::{Bucket, MessagingError};
use crate::models;
use crate::server::runtime::AppConfig;
use crate::utils::data::MapWriter;
use async_nats::jetstream;
use async_nats::jetstream::consumer::{AckPolicy, DeliverPolicy, ReplayPolicy};
use async_nats::jetstream::kv::{CreateErrorKind, Operation, Store};
use dashmap::DashMap;
use futures::StreamExt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use time::UtcOffset;
use tokio::spawn;
use tracing::{debug, error, info, warn};

const KV_OPERATION: &str = "KV-Operation";
const ALL_KEYS: &str = ">";

#[derive(Clone)]
pub struct NatsMessaging {
    context: jetstream::Context,
    exploits_store: Store,
    services_store: Store,
    teams_store: Store,
    executions_wq_stream: jetstream::stream::Stream,
    executions_stream: jetstream::stream::Stream,
    data_stream: jetstream::stream::Stream,
    flags_submissions_stream: jetstream::stream::Stream,
    flags_results_stream: jetstream::stream::Stream,
    scheduling_stream: jetstream::stream::Stream,
}

impl NatsMessaging {
    pub async fn new<S: AsRef<str>>(
        nats_url: S,
        app_config: Option<&AppConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        let context = create_jetstream_context(nats_url).await?;

        // TODO: Move this somewhere else?
        if let Some(app_config) = app_config {
            do_migration(&context, app_config).await?;
        }

        let exploits_store = context.get_key_value("exploits").await?;
        let services_store = context.get_key_value("services").await?;
        let teams_store = context.get_key_value("teams").await?;

        let executions_wq_stream = context.get_stream("executions_wq").await?;
        let executions_stream = context.get_stream("executions").await?;
        let data_stream = context.get_stream("data").await?;
        let flags_submissions_stream = context.get_stream("flag_submissions").await?;
        let flags_results_stream = context.get_stream("flag_results").await?;
        let scheduling_stream = context.get_stream("scheduling").await?;

        Ok(Self {
            context,
            exploits_store,
            services_store,
            teams_store,
            executions_wq_stream,
            executions_stream,
            data_stream,
            flags_submissions_stream,
            flags_results_stream,
            scheduling_stream,
        })
    }

    pub fn exploits(&self) -> impl Bucket<models::Exploit> {
        NatsBucket {
            store: self.exploits_store.clone(),
        }
    }

    pub fn services(&self) -> impl Bucket<models::Service> {
        NatsBucket {
            store: self.services_store.clone(),
        }
    }

    pub fn teams(&self) -> impl Bucket<models::Team> {
        NatsBucket {
            store: self.teams_store.clone(),
        }
    }

    pub fn data(&self) -> DataService {
        DataService {
            context: self.context.clone(),
            data_stream: self.data_stream.clone(),
        }
    }

    pub fn flags(&self) -> FlagsService {
        FlagsService {
            context: self.context.clone(),
            flags_submissions_stream: self.flags_submissions_stream.clone(),
            flags_results_stream: self.flags_results_stream.clone(),
        }
    }

    pub fn executions(&self) -> ExecutionsService {
        ExecutionsService {
            context: self.context.clone(),
            executions_wq_stream: self.executions_wq_stream.clone(),
            executions_stream: self.executions_stream.clone(),
        }
    }

    pub fn scheduling(&self) -> SchedulingService {
        SchedulingService {
            context: self.context.clone(),
            scheduling_stream: self.scheduling_stream.clone(),
        }
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
        ack_wait: Duration,
        deliver_policy: DeliverPolicy,
    ) -> Result<
        impl futures::Stream<Item = Result<MessageWrapper<T>, MessagingServiceError>>,
        MessagingError,
    >
    where
        T: Sized + DeserializeOwned,
    {
        subscribe(
            &self.store.stream,
            jetstream::consumer::pull::Config {
                replay_policy: ReplayPolicy::Instant,
                filter_subject: key.map_or(Default::default(), |key| {
                    format!("{}{}", &self.store.prefix, key)
                }),
                durable_name,
                ack_policy,
                ack_wait,
                deliver_policy,
                ..Default::default()
            },
        )
        .await
    }
}

impl<T> Bucket<T> for NatsBucket
where
    T: DeserializeOwned + Serialize + Send + Sync + 'static,
{
    async fn get(&self, key: &str) -> Result<Option<T>, MessagingError>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        match self.store.get(key).await? {
            Some(bytes) => Ok(serde_json::from_slice(bytes.as_ref())?),
            None => Ok(None),
        }
    }

    async fn list(&self, key_filter: Option<&str>) -> Result<HashMap<String, T>, MessagingError>
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
        while let Some(maybe_msg) = stream.next().await {
            let msg = maybe_msg?;
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

    async fn watch_key(
        &self,
        key: &str,
        durable_name: Option<String>,
        ack_policy: AckPolicy,
        ack_wait: Duration,
        deliver_policy: DeliverPolicy,
    ) -> Result<
        impl futures::Stream<Item = Result<MessageWrapper<T>, MessagingServiceError>>,
        MessagingError,
    > {
        self.watch(
            Some(key),
            durable_name,
            ack_policy.into(),
            ack_wait,
            deliver_policy.into(),
        )
        .await
    }

    async fn watch_all(
        &self,
        durable_name: Option<String>,
        ack_policy: AckPolicy,
        ack_wait: Duration,
        deliver_policy: DeliverPolicy,
    ) -> Result<
        impl futures::Stream<Item = Result<MessageWrapper<T>, MessagingServiceError>>,
        MessagingError,
    >
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        self.watch(
            None,
            durable_name,
            ack_policy.into(),
            ack_wait,
            deliver_policy.into(),
        )
        .await
    }

    async fn put(&self, key: &str, body: &T) -> Result<(), MessagingError>
    where
        T: Serialize + Send + Sync,
    {
        self.store
            .put(key, serde_json::to_vec(body)?.into())
            .await?;
        Ok(())
    }

    async fn create(&self, key: &str, body: &T) -> Result<(), MessagingError>
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

    async fn subscribe_all(&self) -> Result<Arc<DashMap<String, T>>, MessagingError>
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
            while let Some(maybe_msg) = stream.next().await {
                let msg = maybe_msg?;
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

async fn create_jetstream_context<S: AsRef<str>>(
    nats_url: S,
) -> color_eyre::eyre::Result<jetstream::Context> {
    let client = async_nats::connect(nats_url.as_ref()).await?;
    let ctx = jetstream::new(client);
    Ok(ctx)
}

async fn do_migration(
    context: &jetstream::Context,
    app_config: &AppConfig,
) -> color_eyre::eyre::Result<()> {
    info!("creating jetstream streams");

    let max_age = app_config.competition.tick * (app_config.competition.flag_validity as u64 + 1);
    debug! {
        max_age,
        "using max_age"
    }

    debug!("creating executions_wq stream");
    context
        .create_stream(jetstream::stream::Config {
            name: "executions_wq".to_string(),
            subjects: vec!["executions.*.request".to_string()],
            discard: jetstream::stream::DiscardPolicy::Old,
            // Important: this will provide idempotency for execution requests
            duplicate_window: Duration::from_secs(max_age),
            max_age: Duration::from_secs(max_age),
            ..Default::default()
        })
        .await?;

    debug!("creating executions stream");
    context
        .create_stream(jetstream::stream::Config {
            name: "executions".to_string(),
            subjects: vec!["executions.*.result".to_string()],
            discard: jetstream::stream::DiscardPolicy::Old,
            ..Default::default()
        })
        .await?;

    debug!("creating scheduling stream");
    context
        .create_stream(jetstream::stream::Config {
            name: "scheduling".to_string(),
            subjects: vec!["scheduling.>".to_string()],
            discard: jetstream::stream::DiscardPolicy::Old,
            ..Default::default()
        })
        .await?;

    debug!("creating data stream");
    context
        .create_stream(jetstream::stream::Config {
            name: "data".to_string(),
            subjects: vec!["data.>".to_string()],
            discard: jetstream::stream::DiscardPolicy::Old,
            duplicate_window: Duration::from_secs(max_age),
            max_age: Duration::from_secs(max_age),
            ..Default::default()
        })
        .await?;

    debug!("creating flag_submissions stream");
    context
        .create_stream(jetstream::stream::Config {
            name: "flag_submissions".to_string(),
            subjects: vec!["flags.submit.*".to_string()],
            // Use subject-based deduplication since it is more suitable for large timeframes.
            // See https://nats.io/blog/new-per-subject-discard-policy/
            discard: jetstream::stream::DiscardPolicy::New,
            discard_new_per_subject: true,
            max_messages_per_subject: 1,
            ..Default::default()
        })
        .await?;

    debug!("creating flag_results stream");
    // Submission results do not need to be indexed by the flag.
    context
        .create_stream(jetstream::stream::Config {
            name: "flag_results".to_string(),
            subjects: vec!["flags.result".to_string()],
            discard: jetstream::stream::DiscardPolicy::Old,
            ..Default::default()
        })
        .await?;

    info!("creating kev/value buckets");

    debug!("creating exploits bucket");
    context
        .create_key_value(jetstream::kv::Config {
            bucket: "exploits".to_string(),
            ..Default::default()
        })
        .await?;
    context
        .create_key_value(jetstream::kv::Config {
            bucket: "services".to_string(),
            ..Default::default()
        })
        .await?;
    context
        .create_key_value(jetstream::kv::Config {
            bucket: "teams".to_string(),
            ..Default::default()
        })
        .await?;

    info!("nats migration complete");
    Ok(())
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
            Err(error) => {
                warn! {
                    ?error,
                    "malformed message received"
                }
            }
        },
        Operation::Delete | Operation::Purge => {
            map.remove(&key);
        }
    }
}

pub struct MessageWrapper<T> {
    pub inner: jetstream::Message,
    pub info: MessageInfo,
    pub payload: T,
}

/// JetStream message info without strings
pub struct MessageInfo {
    /// The stream sequence number associated with this message
    pub stream_sequence: u64,
    /// The consumer sequence number associated with this message
    pub consumer_sequence: u64,
    /// The number of delivery attempts for this message
    pub delivered: i64,
    /// the number of messages known by the server to be pending to this consumer
    pub pending: u64,
    /// the time that this message was received by the server from its publisher
    pub published: time::OffsetDateTime,
}

impl MessageInfo {
    pub fn published_millis(&self) -> i64 {
        (self
            .published
            .to_offset(UtcOffset::UTC)
            .unix_timestamp_nanos()
            / 1_000_000) as i64
    }
}

impl From<jetstream::message::Info<'_>> for MessageInfo {
    fn from(value: jetstream::message::Info<'_>) -> Self {
        Self {
            stream_sequence: value.stream_sequence,
            consumer_sequence: value.consumer_sequence,
            delivered: value.delivered,
            pending: value.pending,
            published: value.published,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MessagingServiceError {
    #[error("message processing error: {error}")]
    ProcessingError {
        message: jetstream::Message,
        error: MessagingError,
    },
    #[error("messaging error: {0}")]
    Error(MessagingError),
}

impl Into<MessagingError> for MessagingServiceError {
    fn into(self) -> MessagingError {
        match self {
            MessagingServiceError::ProcessingError { error, .. } => error,
            MessagingServiceError::Error(error) => error,
        }
    }
}

impl<T: DeserializeOwned> MessageWrapper<T> {
    fn from(message: jetstream::Message) -> Result<Self, MessagingServiceError> {
        let payload = match serde_json::from_slice(message.payload.as_ref()) {
            Ok(payload) => payload,
            Err(error) => {
                return Err(MessagingServiceError::ProcessingError {
                    error: error.into(),
                    message,
                });
            }
        };

        let info = match message.info() {
            Ok(info) => info,
            Err(error) => {
                return Err(MessagingServiceError::ProcessingError {
                    error: error.into(),
                    message,
                });
            }
        };
        let info: MessageInfo = info.into();

        Ok(Self {
            inner: message,
            info,
            payload,
        })
    }

    pub async fn ack(&self) -> Result<(), MessagingError> {
        self.inner.ack().await?;
        Ok(())
    }

    pub async fn nak(&self, retry_in: Option<Duration>) -> Result<(), MessagingError> {
        self.inner
            .ack_with(jetstream::AckKind::Nak(retry_in))
            .await?;
        Ok(())
    }

    pub async fn progress(&self) -> Result<(), MessagingError> {
        self.inner.ack_with(jetstream::AckKind::Progress).await?;
        Ok(())
    }

    pub async fn term(&self) -> Result<(), MessagingError> {
        self.inner.ack_with(jetstream::AckKind::Term).await?;
        Ok(())
    }

    pub async fn retry_linear(&self, delay: Duration, retries: i64) -> Result<(), MessagingError> {
        if self.info.delivered > retries {
            self.inner.ack_with(jetstream::AckKind::Term).await?;
            return Ok(());
        }

        self.inner
            .ack_with(jetstream::AckKind::Nak(Some(delay)))
            .await?;

        Ok(())
    }
}

pub trait Fetcher<T> {
    fn next(
        &mut self,
    ) -> impl Future<
        Output = Result<
            impl futures::Stream<Item = Result<MessageWrapper<T>, MessagingServiceError>> + Send + Sync,
            MessagingError,
        >,
    > + Send
           + Sync;
}

pub struct StreamFetcher {
    consumer: jetstream::consumer::PullConsumer,
    limit: usize,
}

impl<T: DeserializeOwned> Fetcher<T> for StreamFetcher {
    async fn next(
        &mut self,
    ) -> Result<
        impl futures::Stream<Item = Result<MessageWrapper<T>, MessagingServiceError>>,
        MessagingError,
    > {
        let stream = self
            .consumer
            .fetch()
            .max_messages(self.limit)
            .messages()
            .await?;
        let stream = map_message_stream(stream);
        Ok(stream)
    }
}

pub async fn publish_with_id<S, I, B>(
    context: &jetstream::Context,
    subject: S,
    id: I,
    payload: &B,
) -> Result<jetstream::context::PublishAckFuture, MessagingError>
where
    S: async_nats::subject::ToSubject,
    I: async_nats::header::IntoHeaderValue,
    B: Serialize + ?Sized,
{
    let mut headers = async_nats::HeaderMap::new();
    headers.insert(async_nats::header::NATS_MESSAGE_ID, id);

    let value = serde_json::to_vec(payload)?;
    let fut = context
        .publish_with_headers(subject, headers, value.into())
        .await?;

    Ok(fut)
}

pub async fn publish<S, B>(
    context: &jetstream::Context,
    subject: S,
    payload: &B,
) -> Result<jetstream::context::PublishAckFuture, MessagingError>
where
    S: async_nats::subject::ToSubject,
    B: Serialize + ?Sized,
{
    let value = serde_json::to_vec(payload)?;
    let fut = context.publish(subject, value.into()).await?;

    Ok(fut)
}

pub fn map_message_stream<E, S, T>(
    stream: S,
) -> impl futures::Stream<Item = Result<MessageWrapper<T>, MessagingServiceError>>
where
    E: Into<MessagingError>,
    S: futures::Stream<Item = Result<jetstream::Message, E>>,
    T: DeserializeOwned + ?Sized,
{
    stream.map(|res| match res {
        Ok(message) => MessageWrapper::<T>::from(message),
        Err(error) => Err(MessagingServiceError::Error(error.into())),
    })
}

pub async fn subscribe_ordered<T>(
    stream: &jetstream::stream::Stream,
    subject_filter: Option<String>,
    deliver_policy: DeliverPolicy,
) -> Result<
    impl futures::Stream<Item = Result<MessageWrapper<T>, MessagingServiceError>>,
    MessagingError,
>
where
    T: DeserializeOwned + ?Sized,
{
    let filter_subject = subject_filter.unwrap_or("".to_string());
    let consumer = stream
        .create_consumer(jetstream::consumer::pull::OrderedConfig {
            replay_policy: ReplayPolicy::Instant,
            filter_subject,
            deliver_policy,
            ..Default::default()
        })
        .await?;

    let stream = map_message_stream(consumer.messages().await?);
    Ok(stream)
}

pub async fn subscribe<T>(
    stream: &jetstream::stream::Stream,
    config: jetstream::consumer::pull::Config,
) -> Result<
    impl futures::Stream<Item = Result<MessageWrapper<T>, MessagingServiceError>>,
    MessagingError,
>
where
    T: DeserializeOwned + ?Sized,
{
    let consumer = stream.create_consumer(config).await?;
    let stream = map_message_stream(consumer.messages().await?);
    Ok(stream)
}

pub async fn fetch<T>(
    stream: &jetstream::stream::Stream,
    config: jetstream::consumer::pull::Config,
    limit: usize,
) -> Result<impl Fetcher<T>, MessagingError>
where
    T: DeserializeOwned + ?Sized,
{
    let consumer = stream.create_consumer(config).await?;
    let subscription = StreamFetcher { consumer, limit };
    Ok(subscription)
}

pub async fn list_stream<T>(
    stream: &jetstream::stream::Stream,
    subject_filter: Option<String>,
) -> Result<Vec<MessageWrapper<T>>, MessagingError>
where
    T: DeserializeOwned + ?Sized,
{
    let filter_subject = subject_filter.unwrap_or("".to_string());
    let consumer = stream
        .create_consumer(jetstream::consumer::pull::OrderedConfig {
            replay_policy: ReplayPolicy::Instant,
            deliver_policy: DeliverPolicy::All,
            filter_subject,
            ..Default::default()
        })
        .await?;

    let num_pending = consumer.cached_info().num_pending;
    if num_pending == 0 {
        return Ok(vec![]);
    }

    let mut buf = Vec::with_capacity(num_pending as usize); // Let's hope that this never overflows
    let mut stream = map_message_stream(consumer.messages().await?);
    while let Some(maybe_message) = stream.next().await {
        let message = maybe_message.map_err(Into::<MessagingError>::into)?;
        let pending = message.info.pending;
        buf.push(message);

        if pending == 0 {
            break;
        }
    }

    Ok(buf)
}
