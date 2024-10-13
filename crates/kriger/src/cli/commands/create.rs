// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use std::path::Path;

use crate::cli::models::CliConfig;
use crate::cli::{self, read_cli_config, with_spinner};
use crate::cli::{args, emoji};
use color_eyre::eyre::{self, bail};
use color_eyre::eyre::{Context, ContextCompat};
use console::style;
use futures::TryStreamExt;
use kriger_common::client::KrigerClient;
use kriger_common::models;
use tokio::runtime::Handle;
use tokio::task;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tokio_util::io::SyncIoBridge;

const OCI_TEMPLATE_LAYER_MEDIA_TYPE: &str = "application/vnd.kriger.exploit.template.v1.tar+gzip";

// TODO: Handle the error in a user friendly way
pub(crate) async fn main(args: args::Create) -> eyre::Result<()> {
    let cli_config = read_cli_config().await?;

    let exploit_name = inquire_text(args.name, "Exploit name:")?;

    let exists = tokio::fs::metadata(&exploit_name)
        .await
        .map_or_else(|_| false, |m| m.is_dir());

    if exists {
        println!(
            "  {} {}",
            emoji::CROSS_MARK,
            style("The exploit already exists").red().bold()
        );
        return Ok(());
    }

    let service = match inquire_service(args.service, &cli_config).await? {
        Some(service) => service,
        None => return Ok(()), // Should be handled by inquire_service
    };

    let client = create_oci_client(&cli_config);
    let template_tags = get_template_tags(&client, &cli_config).await?;

    let template_select = inquire::Select::new("Template:", template_tags.tags);
    let template = template_select.prompt()?;

    tokio::fs::create_dir(&exploit_name)
        .await
        .context("unable to create the exploit directory")?;

    handle_template_download(&client, &cli_config, &template, &exploit_name).await?;

    let manifest = create_manifest(&exploit_name, service);
    let manifest_toml =
        toml::to_string_pretty(&manifest).context("unable to serialize the exploit manifest")?;

    tokio::fs::write(format!("{}/exploit.toml", &exploit_name), manifest_toml)
        .await
        .context("unable to write the exploit manifest")?;

    println!(
        "  {} {} {}",
        emoji::CHECK_MARK,
        style("Exploit created:").green().bold(),
        &exploit_name
    );

    Ok(())
}

fn inquire_text<S: AsRef<str>>(opt: Option<String>, message: S) -> eyre::Result<String> {
    if let Some(value) = opt {
        return Ok(value);
    }
    let value = inquire::prompt_text(message)?;
    Ok(value)
}

async fn inquire_service(
    service: Option<String>,
    cli_config: &CliConfig,
) -> eyre::Result<Option<String>> {
    let service = match service {
        Some(service) => service,
        None => {
            let client = KrigerClient::new(cli_config.client.rest_url.clone());
            let maybe_services = with_spinner("Fetching competition services", || {
                client.get_competition_services()
            })
            .await
            .context("unable to fetch competition services")?;
            let services: Vec<String> = match maybe_services {
                models::responses::AppResponse::Ok(services) => {
                    services.into_iter().map(|svc| svc.name).collect()
                }
                models::responses::AppResponse::Error { message } => {
                    println!(
                        "  {} {}: {}",
                        emoji::CROSS_MARK,
                        style("Unable to fetch services").red().bold(),
                        message
                    );
                    return Ok(None);
                }
            };
            let select = inquire::Select::new("Service:", services);
            select.prompt()?
        }
    };
    Ok(Some(service))
}

fn create_manifest(name: &str, service: String) -> cli::models::ExploitManifest {
    cli::models::ExploitManifest {
        image: None,
        exploit: cli::models::InnerExploitManifest {
            name: name.to_string(),
            service,
            replicas: 1,
            workers: Some(4),
            enabled: true,
            resources: cli::models::ExploitResources {
                cpu_request: Some("1".to_string()),
                mem_request: Some("256Mi".to_string()),
                cpu_limit: "2".to_string(),
                mem_limit: "512Mi".to_string(),
                timeout: 10,
            },
        },
    }
}

fn create_oci_client(config: &CliConfig) -> oci_distribution::Client {
    let protocol = match config.registry.secure {
        true => oci_distribution::client::ClientProtocol::Https,
        false => oci_distribution::client::ClientProtocol::Http,
    };
    let client_config = oci_distribution::client::ClientConfig {
        protocol,
        ..Default::default()
    };
    oci_distribution::Client::new(client_config)
}

fn create_oci_client_auth(config: &CliConfig) -> oci_distribution::secrets::RegistryAuth {
    oci_distribution::secrets::RegistryAuth::Basic(
        config.registry.username.clone(),
        config.registry.password.clone(),
    )
}

async fn get_template_tags(
    client: &oci_distribution::Client,
    config: &CliConfig,
) -> eyre::Result<oci_distribution::client::TagResponse> {
    let reference: oci_distribution::Reference =
        format!("{}/kriger/exploit-templates", &config.registry.registry).parse()?;
    let auth = create_oci_client_auth(&config);

    let tags = client
        .list_tags(&reference, &auth, None, None)
        .await
        .context("unable to list tags")?;
    Ok(tags)
}

async fn handle_template_download(
    client: &oci_distribution::Client,
    config: &CliConfig,
    tag: &str,
    dest: impl AsRef<Path>,
) -> eyre::Result<()> {
    let reference: oci_distribution::Reference = format!(
        "{}/kriger/exploit-templates:{}",
        &config.registry.registry, tag
    )
    .parse()?;
    let auth = create_oci_client_auth(&config);

    let (manifest, _) = with_spinner("Pulling template manifest", || {
        client.pull_manifest(&reference, &auth)
    })
    .await
    .context("unable to pull the template manifest")?;

    let manifest = match manifest {
        oci_distribution::manifest::OciManifest::Image(manifest) => manifest,
        oci_distribution::manifest::OciManifest::ImageIndex(index) => {
            bail!("unexpected oci manifest variant: {index:?}")
        }
    };

    let descriptor = manifest
        .layers
        .into_iter()
        .find(|layer| layer.media_type == OCI_TEMPLATE_LAYER_MEDIA_TYPE)
        .context("unable to locate the oci image layer for the template")?;

    let stream = with_spinner("Initiating download", || {
        client.pull_blob_stream(&reference, &descriptor)
    })
    .await
    .context("unable to initiate blob layer download stream")?
    .into_async_read()
    .compat();

    let handle = Handle::current();

    // TODO: Maybe add a progress bar? However, exploit templates are extremely small in size.
    with_spinner("Downloading and unpacking template", || async {
        task::block_in_place(move || {
            let stream = SyncIoBridge::new_with_handle(stream, handle);
            let tar = flate2::read::GzDecoder::new(stream);
            let mut archive = tar::Archive::new(tar);

            // TODO: Do more unpack hardening here.
            archive.set_overwrite(false);
            archive.unpack(dest)
        })
    })
    .await
    .context("unpack error")?;

    Ok(())
}
