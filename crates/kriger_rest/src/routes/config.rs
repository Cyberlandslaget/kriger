// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use crate::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use kriger_common::models;
use std::ops::Deref;
use std::sync::Arc;

pub(crate) async fn get_server_config(state: State<Arc<AppState>>) -> impl IntoResponse {
    let config = state.runtime.config.deref().clone();
    let config: models::AppConfig = config.into();
    Json(models::responses::AppResponse::Ok(config))
}
