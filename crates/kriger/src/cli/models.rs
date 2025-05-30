// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use kriger_common::models;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct CliConfig {
    pub client: CliClientConfig,
    pub registry: CliRegistryConfig,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CliClientConfig {
    pub rest_url: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CliRegistryConfig {
    pub secure: bool,
    pub registry: String,
    /// If true, the registry will be used for exploit templates too
    pub custom_templates: bool,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ExploitManifest {
    /// If specified, the CLI will skip the building step
    pub image: Option<String>,
    pub exploit: InnerExploitManifest,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct InnerExploitManifest {
    pub name: String,
    pub service: String,
    pub replicas: i32,
    pub workers: Option<i32>,
    pub enabled: bool,
    pub resources: ExploitResources,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ExploitResources {
    pub cpu_request: Option<String>,
    pub mem_request: Option<String>,
    pub cpu_limit: String,
    pub mem_limit: String,
    pub timeout: u32,
}

impl Into<models::ExploitManifest> for InnerExploitManifest {
    fn into(self) -> models::ExploitManifest {
        models::ExploitManifest {
            name: self.name,
            service: self.service,
            replicas: self.replicas,
            workers: self.workers,
            enabled: self.enabled,
            resources: self.resources.into(),
        }
    }
}

impl Into<models::ExploitResources> for ExploitResources {
    fn into(self) -> models::ExploitResources {
        models::ExploitResources {
            cpu_request: self.cpu_request,
            mem_request: self.mem_request,
            cpu_limit: self.cpu_limit,
            mem_limit: self.mem_limit,
            timeout: self.timeout,
        }
    }
}
