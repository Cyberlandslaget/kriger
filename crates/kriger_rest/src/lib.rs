// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

pub mod config;
mod routes;
mod support;

use crate::config::Config;
use axum::http::header::InvalidHeaderValue;
use axum::http::HeaderValue;
use axum::routing::{get, post};
use axum::{http, routing::put, Router};
use color_eyre::eyre;
use color_eyre::eyre::Context;
use kriger_common::server::runtime::AppRuntime;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::cors;
use tower_http::trace::TraceLayer;
use tracing::info;

struct AppState {
    runtime: AppRuntime,
}

pub async fn main(runtime: AppRuntime, config: Config) -> eyre::Result<()> {
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

    let origins = config
        .rest_cors_origins
        .into_iter()
        .map(|origin| origin.parse())
        .collect::<Result<Vec<HeaderValue>, InvalidHeaderValue>>()
        .context("unable to parse cors origins")?;

    let app = Router::new()
        .route("/exploits", get(routes::exploits::get_exploits))
        .route("/exploits/{name}", put(routes::exploits::update_exploit))
        .route(
            "/exploits/{name}/execute",
            post(routes::exploits::execute_exploit),
        )
        .route("/flags", post(routes::flags::submit_flags))
        .route(
            "/competition/services",
            get(routes::competition::get_services),
        )
        .route("/competition/teams", get(routes::competition::get_teams))
        .route(
            "/competition/flag_hints",
            get(routes::competition::get_flag_hints),
        )
        .route("/config/server", get(routes::config::get_server_config))
        .layer(TraceLayer::new_for_http())
        .layer(
            cors::CorsLayer::new()
                .allow_origin(origins)
                .allow_methods(cors::Any)
                .allow_headers([
                    http::header::ORIGIN,
                    http::header::CONTENT_TYPE,
                    http::header::ACCEPT,
                ]),
        )
        .with_state(Arc::new(state));

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            cancellation_token.cancelled().await;
        })
        .await
        .context("http server error")?;

    Ok(())
}
