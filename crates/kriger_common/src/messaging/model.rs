use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// The submitter configuration. This will be dynamically checked by the submitter at runtime
    /// to avoid having to model it in this crate.
    pub submitter: serde_json::Value,
    /// The fetcher configuration.
    pub fetcher: serde_json::Value,
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
    pub workers: Option<i32>,
    pub enabled: bool,
    pub resources: ExploitResources,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExploitResources {
    pub cpu_request: Option<String>,
    pub mem_request: Option<String>,
    pub cpu_limit: String,
    pub mem_limit: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub has_hint: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Team {
    pub name: Option<String>,
    pub ip_address: Option<String>,
    /// A map of service IP addresses. This is only used in situations where services have different
    /// IP addresses. If an entry does not exist, the [ip_address] field is used.
    pub services: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionRequest {
    #[serde(rename = "a")]
    pub ip_address: String,
    #[serde(rename = "h")]
    pub flag_hint: Option<serde_json::Value>,
    /// The Team ID that this execution is targeted towards. This should only be optional for
    /// manual/emergency runs.
    #[serde(rename = "t")]
    pub team_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionResult {
    #[serde(rename = "e")]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlagSubmission {
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
#[repr(u8)]
pub enum FlagSubmissionStatus {
    Ok = 1,
    Duplicate = 2,
    Own = 3,
    Old = 4,
    Invalid = 5,
    /// Server refused flag. Pre- or post-competition.
    /// Submitters should retry this status.
    Error = 6,
    Unknown(String) = 7,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tick {
    #[serde(rename = "i")]
    pub tick: i32,
    /// Milliseconds since Unix Epoch in UTC
    #[serde(rename = "t")]
    pub timestamp: i32,
}
