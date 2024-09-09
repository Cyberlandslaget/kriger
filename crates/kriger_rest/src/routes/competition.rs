use crate::support::AppError;
use crate::AppState;
use axum::response::IntoResponse;
use axum::{extract::State, Json};
use kriger_common::messaging::Bucket;
use kriger_common::models;
use std::sync::Arc;

pub(crate) async fn get_services(
    state: State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let services_bucket = state.runtime.messaging.services();
    let services: Vec<models::Service> = services_bucket.list(None).await?.into_values().collect();

    Ok(Json(models::responses::AppResponse::Ok(services)))
}

pub(crate) async fn get_teams(state: State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let teams_bucket = state.runtime.messaging.teams();

    // We return a map of team network id to the team data
    let teams = teams_bucket.list(None).await?;

    Ok(Json(models::responses::AppResponse::Ok(teams)))
}
