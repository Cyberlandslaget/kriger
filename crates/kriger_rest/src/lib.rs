pub mod config;
mod deploy;

use crate::config::Config;
use axum::{routing::put, Router};
use color_eyre::eyre::{Context, Result};
use kriger_common::runtime::AppRuntime;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::info;

struct AppState {
    runtime: AppRuntime,
}

pub async fn main(runtime: AppRuntime, config: Config) -> Result<()> {
    info!("starting rest server");

    let addr: SocketAddr = config
        .rest_listen
        .parse()
        .context("unable to parse the listening address")?;
    let listener = TcpListener::bind(addr)
        .await
        .context("unable to start the rest server, is the port taken?")?;
    info!("listening on {addr:?}");

    let cancellation_token = runtime.cancellation_token.clone();
    let state = AppState { runtime };

    let app = Router::new()
        .route("/launch", put(deploy::launch))
        .with_state(Arc::new(state));
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            cancellation_token.cancelled().await;
        })
        .await
        .context("http server error")?;

    Ok(())
}
