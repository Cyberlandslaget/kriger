// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use crate::support::{AppError, AppQuery};
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

pub(crate) async fn get_flag_hints(
    state: State<Arc<AppState>>,
    query: AppQuery<models::requests::FlagHintQuery>,
) -> Result<impl IntoResponse, AppError> {
    let data_svc = state.runtime.messaging.data();

    let flag_hints: Vec<models::FlagHint> = data_svc
        .get_flag_hints(Some(&query.service))
        .await?
        .into_iter()
        .map(|hint| models::FlagHint {
            team_id: hint.payload.team_id,
            service: hint.payload.service,
            hint: hint.payload.hint,
        })
        .collect();

    Ok(Json(models::responses::AppResponse::Ok(flag_hints)))
}
