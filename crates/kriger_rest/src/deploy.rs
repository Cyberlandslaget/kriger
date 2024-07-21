use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use kriger_common::messaging::{model::Exploit, Bucket, Messaging};
use tracing::error;

use crate::AppState;

pub(crate) async fn launch(
    state: State<Arc<AppState>>,
    exploit: Json<Exploit>,
) -> (StatusCode, String) {
    let exploits_bucket = match state.runtime.messaging.exploits().await {
        Ok(bucket) => bucket,
        Err(err) => {
            let error_msg = format!("Unable to retrieve the exploits bucket: {err:?}");
            error!("{error_msg:?}");
            return (StatusCode::INTERNAL_SERVER_ERROR, error_msg);
        }
    };

    match exploits_bucket
        .put(&exploit.manifest.name, &exploit.0)
        .await
    {
        Ok(()) => (StatusCode::OK, "success".into()),
        Err(err) => {
            let error_msg = format!("Unable to put exploit into bucket: {err:?}");
            error!("{error_msg:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, error_msg)
        }
    }
}
