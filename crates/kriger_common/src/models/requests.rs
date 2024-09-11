use serde::Deserialize;

#[derive(Deserialize)]
pub struct FlagHintQuery {
    pub service: String,
}
