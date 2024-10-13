// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FlagHintQuery {
    pub service: String,
}

#[derive(Serialize, Deserialize)]
pub struct FlagSubmitRequest {
    pub flags: Vec<String>,
}
