use crate::models::FlagSubmissionStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FlagHint {
    #[serde(rename = "t")]
    pub team_id: String,
    #[serde(rename = "s")]
    pub service: String,
    #[serde(rename = "h")]
    pub hint: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionRequest {
    #[serde(rename = "a")]
    pub ip_address: String,
    #[serde(rename = "h", skip_serializing_if = "Option::is_none")]
    pub flag_hint: Option<serde_json::Value>,
    /// The Team ID that this execution is targeted towards. This should only be optional for
    /// manual/emergency runs.
    #[serde(rename = "t", skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionResult {
    #[serde(rename = "e")]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlagSubmission {
    /// The flag itself
    #[serde(rename = "f")]
    pub flag: String,

    /// The network id of the team that the flag was retrieved from
    #[serde(rename = "t", skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,

    /// The service that stored this flag
    #[serde(rename = "s", skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,

    /// The exploit that retrieved this flag
    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    pub exploit: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlagSubmissionResult {
    #[serde(rename = "f")]
    pub flag: String,

    /// The network id of the team that the flag was retrieved from
    #[serde(rename = "t", skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,

    /// The service that stored this flag
    #[serde(rename = "s", skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,

    /// The exploit that retrieved this flag
    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    pub exploit: Option<String>,

    #[serde(rename = "r")]
    pub status: FlagSubmissionStatus,

    #[serde(rename = "p", skip_serializing_if = "Option::is_none")]
    pub points: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SchedulingTick {
    #[serde(rename = "i")]
    pub tick: i64,
}
