// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::extract::{FromRequest, FromRequestParts, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use kriger_common::messaging::MessagingError;
use kriger_common::models;

#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
#[non_exhaustive]
pub(crate) enum AppError {
    #[error("Bad input: {0}")]
    BadInput(&'static str),
    #[error("Internal messaging error")]
    MessagingError(#[from] MessagingError),
    #[error("{0}")]
    JsonRejection(#[from] JsonRejection),
    #[error("{0}")]
    QueryRejection(#[from] QueryRejection),
    #[error("Not found")]
    NotFound,
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match &self {
            // User errors
            AppError::BadInput(_) => StatusCode::BAD_REQUEST,
            AppError::JsonRejection(_) => StatusCode::BAD_REQUEST,
            AppError::QueryRejection(_) => StatusCode::BAD_REQUEST,

            // General errors
            AppError::NotFound => StatusCode::NOT_FOUND,

            // Server errors
            AppError::MessagingError(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let res: models::responses::AppResponse<()> = models::responses::AppResponse::Error {
            message: self.to_string(),
        };
        let mut res = Json(res).into_response();
        *res.status_mut() = self.status_code();
        res
    }
}

// Create our own JSON extractor by wrapping `axum::Json`. This makes it easy to override the
// rejection and provide our own which formats errors to match our application.
//
// `axum::Json` responds with plain text if the input is invalid.
#[derive(FromRequest)]
#[from_request(via(Json), rejection(AppError))]
pub(crate) struct AppJson<T>(pub(crate) T);

impl<T> IntoResponse for AppJson<T>
where
    Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}

#[derive(FromRequestParts)]
#[from_request(via(Query), rejection(AppError))]
pub(crate) struct AppQuery<T>(pub(crate) T);

impl<T> IntoResponse for AppQuery<T>
where
    Query<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        Query(self.0).into_response()
    }
}

impl<T> std::ops::Deref for AppQuery<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
