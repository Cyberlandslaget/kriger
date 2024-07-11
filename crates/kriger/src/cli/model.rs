use kriger_common::messaging;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ExploitManifest {
    pub exploit: messaging::model::ExploitManifest,
}
