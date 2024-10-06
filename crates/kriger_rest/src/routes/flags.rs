use crate::support::{AppError, AppJson};
use crate::AppState;
use axum::response::IntoResponse;
use axum::{extract::State, Json};
use kriger_common::messaging::model;
use kriger_common::models;
use std::sync::Arc;

pub(crate) async fn submit_flags(
    state: State<Arc<AppState>>,
    AppJson(request): AppJson<models::requests::FlagSubmitRequest>,
) -> Result<impl IntoResponse, AppError> {
    let flags_svc = state.runtime.messaging.flags();
    
    // FIXME: Probably parallelize this, but whatever
    for flag in request.flags {
        flags_svc
            .submit_flag(&model::FlagSubmission {
                flag,
                team_id: None,
                service: None,
                exploit: None,
            })
            .await?;
    }

    Ok(Json(models::responses::AppResponse::Ok(())))
}
