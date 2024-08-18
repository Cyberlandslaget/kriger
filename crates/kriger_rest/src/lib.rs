pub mod config;
mod routes;
mod support;

use crate::config::Config;
use axum::http::header::InvalidHeaderValue;
use axum::http::HeaderValue;
use axum::routing::get;
use axum::{routing::put, Router};
use color_eyre::eyre;
use color_eyre::eyre::Context;
use kriger_common::runtime::AppRuntime;
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
        .route("/exploits/:name", put(routes::exploits::update_exploit))
        .route(
            "/competition/services",
            get(routes::competition::get_services),
        )
        .route("/competition/teams", get(routes::competition::get_teams))
        .route(
            "/config/competition",
            get(routes::config::get_competition_config),
        )
        .layer(TraceLayer::new_for_http())
        .layer(cors::CorsLayer::new().allow_origin(origins))
        .with_state(Arc::new(state));

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            cancellation_token.cancelled().await;
        })
        .await
        .context("http server error")?;

    Ok(())
}
