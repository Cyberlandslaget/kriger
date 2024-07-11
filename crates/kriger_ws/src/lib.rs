pub mod config;

use crate::config::Config;
use color_eyre::eyre::{bail, Context, Result};
use fastwebsockets::{upgrade, OpCode, WebSocketError};
use http_body_util::Empty;
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use kriger_common::runtime::AppRuntime;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;
use tracing::{info, warn};

pub async fn main(runtime: AppRuntime, config: Config) -> Result<()> {
    info!("starting websocket server");

    let addr: SocketAddr = config
        .ws_listen
        .parse()
        .context("unable to parse the listening address")?;
    let listener = TcpListener::bind(addr)
        .await
        .context("unable to start the websocket server, is the port taken?")?;

    info!("listening on {addr:?}");

    loop {
        let (stream, client_socket) = listener.accept().await?;
        info!("accepted client: {client_socket:?}");
        spawn(async move {
            if let Err(err) = handle_conn(stream).await {
                warn!("connection error: {err:?}");
            }
        });
    }
}
async fn handle_conn(stream: TcpStream) -> Result<()> {
    let io = TokioIo::new(stream);
    let res = Builder::new(TokioExecutor::new())
        .serve_connection_with_upgrades(io, service_fn(server_upgrade))
        .await;
    if let Err(err) = res {
        bail!("connection error: {err:?}");
    }
    Ok(())
}

async fn server_upgrade(
    mut req: Request<Incoming>,
) -> Result<Response<Empty<Bytes>>, WebSocketError> {
    let (response, fut) = upgrade::upgrade(&mut req)?;

    spawn(async move {
        if let Err(err) = tokio::task::unconstrained(handle_client(fut)).await {
            warn!("websocket error: {err:?}");
        }
    });

    Ok(response)
}

async fn handle_client(fut: upgrade::UpgradeFut) -> Result<(), WebSocketError> {
    let mut ws = fastwebsockets::FragmentCollector::new(fut.await?);

    loop {
        let frame = ws.read_frame().await?;
        match frame.opcode {
            OpCode::Close => break,
            OpCode::Text | OpCode::Binary => {
                ws.write_frame(frame).await?;
            }
            _ => {}
        }
    }

    Ok(())
}
