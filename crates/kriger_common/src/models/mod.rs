///! This module contains shared models that are used across various data layers.
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;

pub mod requests;
pub mod responses;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exploit {
    pub manifest: ExploitManifest,
    pub image: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExploitManifest {
    pub name: String,
    pub service: String,
    pub replicas: i32,
    pub workers: Option<i32>,
    pub enabled: bool,
    pub resources: ExploitResources,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExploitResources {
    pub cpu_request: Option<String>,
    pub mem_request: Option<String>,
    pub cpu_limit: String,
    pub mem_limit: String,
    pub timeout: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub name: String,
    pub has_hint: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub name: Option<String>,
    pub ip_address: Option<String>,
    /// A map of service IP addresses. This is only used in situations where services have different
    /// IP addresses. If an entry does not exist, the [ip_address] field is used.
    pub services: HashMap<String, String>,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum FlagSubmissionStatus {
    Ok = 1,
    Duplicate = 2,
    Own = 3,
    Nop = 4,
    Old = 5,
    Invalid = 6,
    /// The server explicitly requests the flag to be resubmitted.
    /// This can be due to the fact that the flag is not yet valid.
    /// Submitters should retry this status.
    Resubmit = 7,
    /// Server refused flag. Pre- or post-competition.
    /// Submitters should retry this status.
    Error = 8,
    /// Unknown response. Submitters should definitely retry this status.
    Unknown = 200,
}

impl FlagSubmissionStatus {
    pub fn should_retry(&self) -> bool {
        match self {
            FlagSubmissionStatus::Resubmit
            | FlagSubmissionStatus::Error
            | FlagSubmissionStatus::Unknown => true,
            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlagHint {
    pub team_id: String,
    pub service: String,
    pub hint: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub competition: CompetitionConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompetitionConfig {
    /// The start time of the competition in UTC
    pub start: chrono::DateTime<chrono::Utc>,
    /// Tick/round length in seconds
    pub tick: u64,
    /// The start tick in ticks. This indicates the first ticking round between T+0 and T+tick.
    pub tick_start: i64,
    /// The validity of flags in rounds
    pub flag_validity: u32,
    /// The regular expression for the flag format
    pub flag_format: String,
    /// The team id of the NOP team
    pub nop_team: Option<String>,
    /// The team id of the self team
    pub self_team: Option<String>,
}
