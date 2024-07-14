pub mod config;

use crate::config::Config;
use color_eyre::eyre::{Context, Result};
use futures::StreamExt;
use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{Capabilities, Container, EnvVar, PodSecurityContext, PodSpec, PodTemplateSpec, SecurityContext};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};
use kriger_common::messaging::model::Exploit;
use kriger_common::messaging::{AckPolicy, Bucket, DeliverPolicy, Message, Messaging};
use kriger_common::runtime::AppRuntime;
use kube::api::{Patch, PatchParams};
use kube::{Api, Client};
use std::collections::BTreeMap;
use tokio::pin;
use tracing::{info, warn};

pub async fn main(runtime: AppRuntime, config: Config) -> Result<()> {
    info!("starting controller");

    // This will construct a Kubernetes client with the default kubeconfig file or the in-cluster
    // configuration.
    let client = Client::try_default()
        .await
        .context("unable to construct a kubernetes client")?;
    let deployments: Api<Deployment> =
        Api::namespaced(client, &config.controller_exploit_namespace);

    // TODO: Handle deleted exploits?

    let exploits = runtime
        .messaging
        .exploits()
        .await
        .context("unable to retrieve exploits bucket")?;

    // This watches for new exploits and exploit updates. Upon startup, the consumer will receive a
    // replay of all exploits, allowing the controller to reconcile exploits that may've been missed.
    // Technically, we can use a durable consumer here, but this approach allows us to quickly fix
    // provisioning issue with the underlying orchestration platform.
    let exploits_stream = exploits
        .watch_all::<Exploit>(AckPolicy::Explicit, DeliverPolicy::Last)
        .await
        .context("unable to watch exploits")?;
    pin!(exploits_stream);

    while let Some(res) = exploits_stream.next().await {
        match res {
            Ok(message) => {
                if let Err(err) = handle_message(
                    &deployments,
                    message,
                    config.controller_nats_svc_url.clone(),
                )
                    .await
                {
                    warn!("unable to handle message: {err:?}");
                }
            }
            Err(err) => warn!("unable to parse exploit: {err:?}"),
        }
    }

    Ok(())
}

async fn handle_message(
    deployments: &Api<Deployment>,
    message: impl Message<Payload=Exploit>,
    nats_url: String,
) -> Result<()> {
    let exploit = message.payload();
    info!("reconciling exploit: {}", exploit.manifest.name);
    message.progress().await?;
    match reconcile(&deployments, exploit, nats_url).await {
        Ok(..) => {
            message.ack().await?;
        }
        Err(err) => {
            warn!(
                "reconciliation error for exploit: {}: {:?}",
                exploit.manifest.name, err
            );
            message.nak().await?;
        }
    };
    Ok(())
}

async fn reconcile(
    deployments: &Api<Deployment>,
    exploit: &Exploit,
    nats_url: String,
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
            name: "NATS_URL".to_string(),
            value: Some(nats_url),
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
