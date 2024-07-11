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

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionRequest {
    #[serde(rename = "a")]
    pub ip_address: String,
    #[serde(rename = "h")]
    pub flag_hint: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionResult {
    #[serde(rename = "e")]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Flag {
    /// The network id of the team that the flag was retrieved from
    #[serde(rename = "t")]
    pub team_id: Option<String>,

    /// The service that stored this flag
    #[serde(rename = "s")]
    pub service: Option<String>,

    /// The exploit that retrieved this flag
    #[serde(rename = "e")]
    pub exploit: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlagSubmissionResult {
    #[serde(rename = "s")]
    pub status: FlagSubmissionStatus,
    #[serde(rename = "p")]
    pub points: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FlagSubmissionStatus {}
