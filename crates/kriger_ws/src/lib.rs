pub mod config;

use crate::config::Config;
use color_eyre::eyre;
use color_eyre::eyre::{bail, Context};
use fastwebsockets::{upgrade, Frame, Payload, WebSocketError};
use flume::Sender;
use futures::stream::{select_all, StreamExt};
use http_body_util::Empty;
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use kriger_common::messaging::model::{FlagSubmission, FlagSubmissionResult, SchedulingTick};
use kriger_common::messaging::{AckPolicy, Bucket, DeliverPolicy, Message, Messaging, Stream};
use kriger_common::server::runtime::AppRuntime;
use serde::Serialize;
use std::net::SocketAddr;
use time::OffsetDateTime;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;
use tokio::{select, spawn};
use tracing::{debug, info, warn};

pub async fn main(runtime: AppRuntime, config: Config) -> eyre::Result<()> {
    info!("starting websocket server");

    let addr: SocketAddr = config
        .ws_listen
        .parse()
        .context("unable to parse the listening address")?;
    let listener = TcpListener::bind(addr)
        .await
        .context("unable to start the websocket server, is the port taken?")?;

    info!("listening on {addr:?}");

    let cancellation_token = runtime.cancellation_token.clone();

    loop {
        select! {
            _ = cancellation_token.cancelled() => {
                return Ok(());
            },
            res = listener.accept() => {
               let (stream, client_socket) = res?;
                info! {
                    ?client_socket,
                    "accepted a client"
                }
                let runtime_clone = runtime.clone();
                spawn(async move {
                    if let Err(error) = handle_conn(stream, runtime_clone).await {
                        warn! {
                            ?error,
                            "connection handling error"
                        }
                    }
                });
            }
        }
    }
}

async fn handle_conn(stream: TcpStream, runtime: AppRuntime) -> eyre::Result<()> {
    let io = TokioIo::new(stream);
    let res = Builder::new(TokioExecutor::new())
        .serve_connection_with_upgrades(
            io,
            service_fn(move |req| {
                return handle_request(req, runtime.clone());
            }),
        )
        .await;
    if let Err(err) = res {
        bail!("connection error: {err:?}");
    }
    Ok(())
}

async fn handle_request(
    mut req: Request<Incoming>,
    runtime: AppRuntime,
) -> Result<Response<Empty<Bytes>>, WebSocketError> {
    // FIXME: This is kind of ugly
    let from: Option<i64> = req
        .uri()
        .query()
        .map_or(None, |query| {
            form_urlencoded::parse(query.as_bytes())
                .into_owned()
                .collect::<Vec<(String, String)>>()
                .into_iter()
                .find(|(key, _)| key == "from")
        })
        .and_then(|(_, value)| value.parse().ok());
    let (response, fut) = upgrade::upgrade(&mut req)?;

    spawn(async move {
        if let Err(error) = tokio::task::unconstrained(handle_client(fut, runtime, from)).await {
            warn! {
                ?error,
                "websocket error"
            }
        }
    });

    Ok(response)
}

async fn handle_client(
    fut: upgrade::UpgradeFut,
    runtime: AppRuntime,
    from: Option<i64>,
) -> Result<(), WebSocketError> {
    let ws = fut.await?;
    let (_ws_rx, mut ws_tx) = ws.split(tokio::io::split);

    // FIXME: Should probably handle backpressure somehow, eg. the websocket connection being slow.
    let (tx, rx) = flume::unbounded::<WebSocketEvent>();

    let mut set = JoinSet::new();
    // TODO: We may want to share a set of consumer for all WS connections to reduce I/O
    // However, clients will most likely connect at different points in time and we want to
    // replay relevant messages to them.
    set.spawn(subscribe_all(runtime, from, tx));
    set.spawn(async move {
        let mut rx = rx.into_stream();
        while let Some(msg) = rx.next().await {
            match serde_json::to_vec(&msg) {
                Ok(bytes) => {
                    ws_tx
                        .write_frame(Frame::text(Payload::Owned(bytes)))
                        .await?;
                }
                Err(err) => {
                    bail!("serialization error: {err:?}");
                }
            }
        }
        Ok(())
    });
    // TODO: Handle reads and disconnect messages and a few other things

    while let Some(Ok(res)) = set.join_next().await {
        if let Err(error) = res {
            warn! {
                ?error,
                "unexpected error"
            }
        }

        // We abort when the first future returns
        set.abort_all();
        return Ok(());
    }

    Ok(())
}

async fn subscribe_all(
    runtime: AppRuntime,
    from: Option<i64>,
    tx: Sender<WebSocketEvent>,
) -> eyre::Result<()> {
    let deliver_policy = match from {
        Some(timestamp) => {
            match OffsetDateTime::from_unix_timestamp_nanos((timestamp as i128) * 1_000_000) {
                Ok(start_time) => DeliverPolicy::ByStartTime { start_time },
                _ => DeliverPolicy::New,
            }
        }
        None => DeliverPolicy::New,
    };
    debug! {
        ?deliver_policy,
        "consuming messages with deliver policy"
    }

    let flags = runtime
        .messaging
        .flags()
        .await
        .context("unable to retrieve flags bucket")?;

    let scheduling = runtime
        .messaging
        .scheduling()
        .await
        .context("unable to retrieve the scheduling stream")?;

    // TODO: Deliver messages from the last N ticks
    let flag_submissions_stream = flags
        .watch_key::<FlagSubmission>(
            "*.submit",
            None,
            AckPolicy::None,
            Default::default(),
            deliver_policy,
            vec![],
        )
        .await
        .context("unable to watch flags")?
        .filter_map(|res| async {
            res.ok().map(|msg| WebSocketEvent {
                published: msg.published(),
                payload: WebSocketPayload::FlagSubmission(msg.into_payload()),
            })
        });
    let flag_results_stream = flags
        .watch_key::<FlagSubmissionResult>(
            "*.result",
            None,
            AckPolicy::None,
            Default::default(),
            deliver_policy,
            vec![],
        )
        .await
        .context("unable to watch flags")?
        .filter_map(|res| async {
            res.ok().map(|msg| WebSocketEvent {
                published: msg.published(),
                payload: WebSocketPayload::FlagSubmissionResult(msg.into_payload()),
            })
        });
    let scheduling_start_stream = scheduling
        .subscribe::<SchedulingTick>(
            None,
            Some("scheduling.start".to_string()),
            AckPolicy::None,
            DeliverPolicy::Last,
        )
        .await
        .context("unable to subscribe to scheduling start messages")?
        .filter_map(|res| async {
            res.ok().map(|msg| WebSocketEvent {
                published: msg.published(),
                payload: WebSocketPayload::SchedulingStart(msg.into_payload()),
            })
        });

    let mut fused_stream = select_all(vec![
        flag_submissions_stream.boxed(),
        flag_results_stream.boxed(),
        scheduling_start_stream.boxed(),
    ]);
    while let Some(event) = fused_stream.next().await {
        tx.send_async(event).await.context("send error")?;
    }

    Ok(())
}

#[derive(Serialize, Debug)]
struct WebSocketEvent {
    /// Unix timestamp in UTC
    #[serde(rename = "p")]
    published: i64,
    #[serde(flatten)]
    payload: WebSocketPayload,
}

#[derive(Serialize, Debug)]
#[serde(tag = "t", content = "d", rename_all = "snake_case")]
enum WebSocketPayload {
    FlagSubmission(FlagSubmission),
    FlagSubmissionResult(FlagSubmissionResult),
    SchedulingStart(SchedulingTick),
}
