// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use serde::{Deserialize, Serialize};

/// The structure used to serialize consistent responses to the consumer.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum AppResponse<T: Serialize> {
    #[serde(rename = "data")]
    Ok(T),
    Error {
        message: String,
    },
}
