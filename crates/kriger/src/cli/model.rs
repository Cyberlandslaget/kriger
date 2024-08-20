use kriger_common::models;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ExploitManifest {
    /// If specified, the CLI will skip the building step
    pub image: Option<String>,
    pub exploit: models::ExploitManifest,
}
