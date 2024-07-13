use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompetitionConfig {
    /// The start time of the competition in UTC
    pub start: chrono::DateTime<chrono::Utc>,
    /// Tick/round length in seconds
    pub tick: u64,
    /// The start tick in ticks. This is usually 0.
    pub tick_start: i32,
    /// The validity of flags in rounds
    pub flag_validity: u32,
    /// The regular expression for the flag format
    pub flag_format: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Exploit {
    pub manifest: ExploitManifest,
    pub image: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExploitManifest {
    pub name: String,
    pub service: String,
    pub replicas: i32,
    pub enabled: bool,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Tick {
    #[serde(rename = "i")]
    pub tick: i32,
    /// Milliseconds since Unix Epoch in UTC
    #[serde(rename = "t")]
    pub timestamp: i32,
}
