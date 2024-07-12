use kriger_common::messaging;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ExploitManifest {
    /// If specified, the CLI will skip the building step
    pub image: Option<String>,
    pub exploit: messaging::model::ExploitManifest,
}
