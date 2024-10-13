// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use crate::server::args::OpenMetricsConfig;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use color_eyre::eyre;
use color_eyre::eyre::Context;
use kriger_common::server::runtime::AppRuntime;
use std::net::SocketAddr;
use std::ops::Deref;
use tokio::net::TcpListener;
use tracing::info;

pub(crate) async fn run_metrics_server(
    runtime: AppRuntime,
    args: OpenMetricsConfig,
) -> eyre::Result<()> {
    let cancellation_token = runtime.cancellation_token.clone();
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(runtime);

    let addr: SocketAddr = args
        .openmetrics_listen
        .parse()
        .context("unable to parse the listening address")?;
    let listener = TcpListener::bind(addr)
        .await
        .context("unable to start the rest server, is the port taken?")?;

    info!("listening on {addr:?}");
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            cancellation_token.cancelled().await;
        })
        .await
        .context("openmetrics server error")?;

    Ok(())
}

async fn metrics_handler(runtime: axum::extract::State<AppRuntime>) -> impl IntoResponse {
    let mut buffer = String::new();
    prometheus_client::encoding::text::encode(
        &mut buffer,
        runtime.metrics_registry.read().await.deref(),
    )
    .unwrap();
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "application/openmetrics-text; version=1.0.0; charset=utf-8",
        )],
        buffer,
    )
}
