pub mod config;
mod metrics;

use crate::config::Config;
use crate::metrics::{ControllerMetrics, ExploitLabels};
use async_nats::jetstream::consumer::{AckPolicy, DeliverPolicy};
use color_eyre::eyre::{Context, Result};
use futures::StreamExt;
use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{
    Capabilities, Container, EnvVar, LocalObjectReference, PodSpec, PodTemplateSpec,
    ResourceRequirements, SecurityContext,
};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};
use kriger_common::messaging::nats::MessageWrapper;
use kriger_common::messaging::Bucket;
use kriger_common::models;
use kriger_common::server::runtime::AppRuntime;
use kube::api::{Patch, PatchParams};
use kube::{Api, Client};
use std::collections::BTreeMap;
use std::ops::DerefMut;
use std::time::Duration;
use tokio::{pin, select};
use tracing::{error, info, warn};

pub async fn main(runtime: AppRuntime, config: Config) -> Result<()> {
    info!("starting controller");

    let metrics = ControllerMetrics::default();
    metrics.register(runtime.metrics_registry.write().await.deref_mut());

    // This will construct a Kubernetes client with the default kubeconfig file or the in-cluster
    // configuration.
    let client = Client::try_default()
        .await
        .context("unable to construct a kubernetes client")?;
    let deployments: Api<Deployment> =
        Api::namespaced(client, &config.controller_exploit_namespace);

    // TODO: Handle deleted exploits?

    let exploits = runtime.messaging.exploits();

    // This watches for new exploits and exploit updates. Upon startup, the consumer will receive a
    // replay of all exploits, allowing the controller to reconcile exploits that may've been missed.
    // Technically, we can use a durable consumer here, but this approach allows us to quickly fix
    // provisioning issue with the underlying orchestration platform.
    let exploits_stream = exploits
        .watch_key(
            "*",
            None,
            AckPolicy::Explicit,
            Duration::from_secs(60), // TODO: Adjust
            DeliverPolicy::LastPerSubject,
        )
        .await
        .context("unable to watch exploits")?;
    pin!(exploits_stream);

    loop {
        let res = select! {
            _ = runtime.cancellation_token.cancelled() => return Ok(()),
            res = exploits_stream.next() => res
        };
        match res {
            Some(Ok(message)) => {
                let labels = &ExploitLabels {
                    exploit: message.payload.manifest.name.clone(),
                };
                metrics.requests.get_or_create(&labels).inc();
                match handle_message(&deployments, message, &runtime, &config).await {
                    Ok(_) => {
                        metrics.complete.get_or_create(&labels).inc();
                    }
                    Err(error) => {
                        warn! {
                            ?error,
                            "unable to handle message"
                        }
                        metrics.error.get_or_create(&labels).inc();
                    }
                }
            }
            Some(Err(error)) => {
                warn! {
                    ?error,
                    "unable to poll message"
                }
            }
            None => {
                // End of stream
            }
        };
    }
}

async fn handle_message(
    deployments: &Api<Deployment>,
    message: MessageWrapper<models::Exploit>,
    runtime: &AppRuntime,
    config: &Config,
) -> Result<()> {
    let exploit = &message.payload;

    info! {
        exploit.name = exploit.manifest.name,
        "reconciling exploit"
    }
    message.progress().await?;
    match reconcile(&deployments, exploit, runtime, config).await {
        Ok(..) => {
            message.ack().await?;
        }
        Err(err) => {
            error!(
                "reconciliation error for exploit: {}: {:?}",
                exploit.manifest.name, err
            );
            message.nak(Some(Duration::from_secs(2))).await?;
        }
    };
    Ok(())
}

async fn reconcile(
    deployments: &Api<Deployment>,
    exploit: &models::Exploit,
    runtime: &AppRuntime,
    config: &Config,
) -> Result<()> {
    let mut labels = BTreeMap::<String, String>::new();
    labels.insert("exploit".to_string(), exploit.manifest.name.clone());

    let mut env = vec![
        EnvVar {
            name: "EXPLOIT".to_string(),
            value: Some(exploit.manifest.name.clone()),
            ..Default::default()
        },
        EnvVar {
            name: "SERVICE".to_string(),
            value: Some(exploit.manifest.service.clone()),
            ..Default::default()
        },
        EnvVar {
            name: "FLAG_FORMAT".to_string(),
            value: Some(runtime.config.competition.flag_format.clone()),
            ..Default::default()
        },
        EnvVar {
            name: "NATS_URL".to_string(),
            value: Some(config.controller_nats_svc_url.clone()),
            ..Default::default()
        },
        EnvVar {
            name: "TIMEOUT".to_string(),
            value: Some(exploit.manifest.resources.timeout.to_string()),
            ..Default::default()
        },
        EnvVar {
            name: "OTEL_EXPORTER_OTLP_ENDPOINT".to_string(),
            value: Some(config.controller_otlp_endpoint.clone()),
            ..Default::default()
        },
        EnvVar {
            name: "OTEL_SERVICE_NAME".to_string(),
            value: Some(exploit.manifest.name.clone()),
            ..Default::default()
        },
    ];

    if let Some(workers) = &exploit.manifest.workers {
        env.push(EnvVar {
            name: "WORKERS".to_string(),
            value: Some(workers.to_string()),
            ..Default::default()
        });
    }

    let mut requests = BTreeMap::new();
    let mut limits = BTreeMap::new();
    if config.controller_resource_limits {
        if let Some(cpu_request) = exploit.manifest.resources.cpu_request.clone() {
            requests.insert("cpu".to_string(), Quantity(cpu_request));
        }
        if let Some(mem_request) = exploit.manifest.resources.mem_request.clone() {
            requests.insert("memory".to_string(), Quantity(mem_request));
        }
        limits.insert(
            "cpu".to_string(),
            Quantity(exploit.manifest.resources.cpu_limit.clone()),
        );
        limits.insert(
            "memory".to_string(),
            Quantity(exploit.manifest.resources.mem_limit.clone()),
        );
    }

    let spec = DeploymentSpec {
        replicas: Some(exploit.manifest.replicas),
        selector: LabelSelector {
            match_labels: Some(labels.clone()),
            ..Default::default()
        },
        template: PodTemplateSpec {
            metadata: Some(ObjectMeta {
                labels: Some(labels.clone()),
                ..Default::default()
            }),
            spec: Some(PodSpec {
                containers: vec![Container {
                    name: "exploit".to_string(),
                    image: Some(exploit.image.clone()),
                    env: Some(env),
                    resources: Some(ResourceRequirements {
                        claims: None,
                        requests: Some(requests),
                        limits: Some(limits),
                    }),
                    security_context: Some(SecurityContext {
                        allow_privilege_escalation: Some(false),
                        capabilities: Some(Capabilities {
                            add: None,
                            drop: Some(vec!["ALL".to_string()]),
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                image_pull_secrets: Some(vec![LocalObjectReference {
                    name: Some("registry".to_string()),
                }]),
                automount_service_account_token: Some(false),
                enable_service_links: Some(false),
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    };
    let deployment = Deployment {
        metadata: ObjectMeta {
            name: Some(exploit.manifest.name.clone()),
            labels: Some(labels.clone()),
            ..Default::default()
        },
        spec: Some(spec),
        ..Default::default()
    };
    let patch_params = PatchParams {
        field_manager: Some("kriger-controller".to_string()),
        ..Default::default()
    };

    deployments
        .patch(
            &exploit.manifest.name,
            &patch_params,
            &Patch::Apply(deployment),
        )
        .await?;
    info!(
        "created a deployment for exploit: {}",
        &exploit.manifest.name
    );

    Ok(())
}
