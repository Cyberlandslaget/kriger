pub mod config;

use crate::config::Config;
use color_eyre::eyre::{Context, Result};
use futures::StreamExt;
use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{Container, PodSpec, PodTemplateSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};
use kriger_common::messaging::model::Exploit;
use kriger_common::messaging::{Message, Messaging};
use kriger_common::runtime::AppRuntime;
use kube::api::{Patch, PatchParams};
use kube::{Api, Client};
use std::collections::BTreeMap;
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

    // This watches for new exploits and exploit updates. Upon startup, the consumer will receive a
    // replay of all exploits, allowing the controller to reconcile exploits that may've been missed.
    let mut exploits_stream = runtime
        .messaging
        .watch_exploits()
        .await
        .context("unable to watch exploits")?;

    while let Some(res) = exploits_stream.next().await {
        match res {
            Ok(message) => {
                if let Err(err) = handle_message(&deployments, message).await {
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
    message: impl Message<Payload = Exploit>,
) -> Result<()> {
    let exploit = message.payload();
    info!("reconciling exploit: {}", exploit.name);
    message.progress().await?;
    match reconcile(&deployments, exploit).await {
        Ok(..) => {
            message.ack().await?;
        }
        Err(err) => {
            warn!(
                "reconciliation error for exploit: {}: {:?}",
                exploit.name, err
            );
            message.nak().await?;
        }
    };
    Ok(())
}

async fn reconcile(deployments: &Api<Deployment>, exploit: &Exploit) -> Result<()> {
    let mut labels = BTreeMap::<String, String>::new();
    labels.insert("exploit".to_string(), exploit.name.clone());

    let spec = DeploymentSpec {
        replicas: Some(exploit.replicas),
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
                    image: Some(exploit.container.image.clone()),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    };
    let deployment = Deployment {
        metadata: ObjectMeta {
            name: Some(exploit.name.clone()),
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
        .patch(&exploit.name, &patch_params, &Patch::Apply(deployment))
        .await?;
    info!("created a deployment for exploit: {}", &exploit.name);

    Ok(())
}
