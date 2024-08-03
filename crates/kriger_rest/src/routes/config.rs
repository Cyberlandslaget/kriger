use crate::support::{AppError, AppResponse};
use crate::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use kriger_common::messaging::model::CompetitionConfig;
use kriger_common::messaging::{Bucket, Messaging};
use std::sync::Arc;

pub(crate) async fn get_competition_config(
    state: State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let config_bucket = state.runtime.messaging.config().await?;
    match config_bucket
        .get::<CompetitionConfig>("competition")
        .await?
    {
        Some(config) => Ok(Json(AppResponse::Ok(config))),
        None => Err(AppError::NotFound),
    }
}
