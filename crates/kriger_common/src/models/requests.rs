use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FlagHintQuery {
    pub service: String,
}

#[derive(Serialize, Deserialize)]
pub struct FlagSubmitRequest {
    pub flags: Vec<String>,
}
