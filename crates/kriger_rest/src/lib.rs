pub mod config;

use crate::config::Config;
use axum::Router;
use color_eyre::eyre::{Context, Result};
use kriger_common::runtime::AppRuntime;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

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

    let app = Router::new();
    axum::serve(listener, app)
        .await
        .context("http server error")?;

    Ok(())
}
