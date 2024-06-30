use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Exploit {
    name: String,
    enabled: bool,
    service: String,
    container: ExploitContainer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExploitContainer {
    image: String,
}
