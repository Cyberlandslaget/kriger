use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Exploit {
    pub name: String,
    pub enabled: bool,
    pub service: String,
    pub replicas: i32,
    pub container: ExploitContainer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExploitContainer {
    pub image: String,
    // TODO: Add resource limits
}
