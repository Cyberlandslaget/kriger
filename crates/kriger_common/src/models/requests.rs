use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FlagHintQuery {
    pub service: String,
}
