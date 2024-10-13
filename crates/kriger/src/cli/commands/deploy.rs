// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use crate::cli::commands::acquire_exploit_manifest;
use crate::cli::{args, emoji, format_duration_secs, log};
use crate::cli::{read_cli_config, with_spinner};
use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::Result;
use console::style;
use futures::stream::select_all;
use futures::StreamExt;
use indicatif::ProgressBar;
use kriger_common::client::KrigerClient;
use kriger_common::models;
use std::borrow::Cow;
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::Instant;
use tokio_stream::wrappers::LinesStream;
use tracing::debug;

// TODO: Eventually move to bollard if things work?
async fn build_image(
    progress: &ProgressBar,
    tag: &str,
    build_args: &HashMap<&str, &str>,
) -> Result<bool> {
    let start = Instant::now();

    let mut args: Vec<Cow<str>> = vec![
        "buildx".into(),
        "build".into(),
        "--network=host".into(),
        "--push".into(),
        "--pull".into(),
        "--tag".into(),
        tag.into(),
    ];

    for (key, value) in build_args {
        args.push("--build-arg".into());
        args.push(format!("{}={}", key, value).into());
    }

    args.push(".".into());

    let args: Vec<&str> = args.iter().map(|arg| arg.as_ref()).collect();

    debug!("Running: docker {}", args.join(" "));
    let mut child = Command::new("docker")
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("unable to spawn child")?;

    let stdout = child
        .stdout
        .take()
        .context("unable to retrieve a handle to the stdout pipe")?;
    let stderr = child
        .stderr
        .take()
        .context("unable to retrieve a handle to the stderr pipe")?;

    let stdout_stream = LinesStream::new(BufReader::new(stdout).lines());
    let stderr_stream = LinesStream::new(BufReader::new(stderr).lines());

    let mut fused_stream = select_all(vec![stdout_stream.boxed(), stderr_stream.boxed()]);
    while let Some(Ok(line)) = fused_stream.next().await {
        log(&progress, style(line).blue().to_string());
    }

    let exit_status = child.wait().await.context("unable to wait for child")?;
    if !exit_status.success() {
        return Ok(false);
    }

    progress.finish_and_clear();

    let elapsed = start.elapsed();
    println!(
        "\n  {} Built in {}",
        emoji::SPARKLES,
        style(format_duration_secs(&elapsed)).bold().green()
    );
    println!("  {} Tag: {}", emoji::PACKAGE, style(tag).underlined());

    Ok(true)
}

pub(crate) async fn main(args: args::Deploy) -> Result<()> {
    let cli_config = read_cli_config().await?;

    // TODO: Honor the existing image in the CLI manifest
    let cli_manifest = match acquire_exploit_manifest().await {
        Some(manifest) => manifest,
        None => return Ok(()),
    };

    // Convert the CLI manifest format to the common model format
    let manifest: models::ExploitManifest = cli_manifest.exploit.into();

    println!(
        "  {} Preparing to deploy {}",
        emoji::ROCKET,
        style(&manifest.name).green().bold()
    );

    let date = chrono::Utc::now();
    let tag_version = format!("{}", date.timestamp());
    let tag = format!(
        "{}/kriger-exploits/{}:{}",
        &cli_config.registry.registry, &manifest.name, tag_version
    );

    let progress = ProgressBar::new_spinner();
    progress.enable_steady_tick(Duration::from_millis(130));
    progress.set_message(format!(
        "{} Building {}...",
        emoji::HAMMER,
        style(&manifest.name).green().bold()
    ));

    // Prepare build arguments
    let mut build_args: HashMap<&str, &str> = HashMap::new();
    build_args.insert("REGISTRY", &cli_config.registry.registry);

    // TODO: Set up a buildx instance first
    match build_image(&progress, &tag, &build_args).await {
        Err(err) => {
            progress.finish_and_clear();
            println!(
                "  {} {}",
                emoji::CROSS_MARK,
                style("Build failed").red().bold()
            );
            return Err(err);
        }
        Ok(success) => {
            if !success {
                progress.finish_and_clear();
                println!(
                    "  {} {}",
                    emoji::CROSS_MARK,
                    style("Build failed").red().bold()
                );
                // FIXME: Properly propagate error
                return Ok(());
            }
        }
    }

    if args.no_deploy {
        println!(
            "  {} {}",
            emoji::INFORMATION,
            style("The deployment step has been skipped.")
                .yellow()
                .bold()
        );
        return Ok(());
    }

    let progress = ProgressBar::new_spinner();
    progress.enable_steady_tick(Duration::from_millis(130));
    progress.set_message(format!("{} Deploying exploit...", emoji::ROCKET));

    let client = KrigerClient::new(cli_config.client.rest_url);
    let exploit = models::Exploit {
        manifest,
        image: tag,
    };
    let update_res = client.update_exploit(&exploit).await;

    progress.finish_and_clear();
    match update_res {
        Err(err) => {
            println!(
                "  {} {}: {}",
                emoji::CROSS_MARK,
                style("Deployment failed").red().bold(),
                err
            );

            // Propagate this error since it is unexpected
            return Err(err.into());
        }
        Ok(models::responses::AppResponse::Error { message }) => {
            println!(
                "  {} {}: {}",
                emoji::CROSS_MARK,
                style("Deployment failed").red().bold(),
                &message
            );
        }
        Ok(models::responses::AppResponse::Ok(_)) => {
            println!(
                "  {} {}",
                emoji::CHECK_MARK,
                style("Deployment requested successfully").green().bold()
            );
        }
    }

    if args.no_execute {
        println!(
            "  {} {}",
            emoji::INFORMATION,
            style("The execution step has been skipped.")
                .yellow()
                .bold()
        );
        return Ok(());
    }

    // The exploit may be in a stale state at this point, meaning that the old version of the exploit
    // may still be running and it may receive the execution request instead.
    // TODO: Wait for the deployment to actually complete? Eg. the rollout completing
    let execute_res = with_spinner("Scheduling exploit execution", || {
        client.execute_exploit(&exploit.manifest.name)
    })
    .await;

    match execute_res {
        Err(err) => {
            println!(
                "  {} {}: {}",
                emoji::CROSS_MARK,
                style("Exploit execution failed").red().bold(),
                err
            );

            // Propagate this error since it is unexpected
            return Err(err.into());
        }
        Ok(models::responses::AppResponse::Error { message }) => {
            println!(
                "  {} {}: {}",
                emoji::CROSS_MARK,
                style("Exploit execution failed").red().bold(),
                &message
            );
        }
        Ok(models::responses::AppResponse::Ok(_)) => {
            println!(
                "  {} {}",
                emoji::CHECK_MARK,
                style("Exploit execution requested successfully")
                    .green()
                    .bold()
            );
        }
    }

    Ok(())
}
