use axum::response::IntoResponse;
use axum::{extract::State, Json};
use kriger_common::messaging::{model, Bucket, Messaging};
use std::sync::Arc;

use crate::support::{AppError, AppResponse};
use crate::AppState;

pub(crate) async fn get_services(
    state: State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let services_bucket = state.runtime.messaging.services().await?;
    let services: Vec<model::Service> = services_bucket.list(None).await?.into_values().collect();

    Ok(Json(AppResponse::Ok(services)))
}

pub(crate) async fn get_teams(state: State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let teams_bucket = state.runtime.messaging.teams().await?;

    // We return a map of team network id to the team data
    let teams = teams_bucket.list::<model::Team>(None).await?;

    Ok(Json(AppResponse::Ok(teams)))
}
