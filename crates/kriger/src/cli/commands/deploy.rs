use crate::cli::{self, read_cli_config};
use crate::cli::{args, emoji, format_duration_secs, log};
use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::Result;
use console::style;
use futures::stream::select_all;
use futures::StreamExt;
use indicatif::ProgressBar;
use kriger_common::client::KrigerClient;
use kriger_common::models;
use std::process::Stdio;
use std::time::Duration;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::Instant;
use tokio_stream::wrappers::LinesStream;
use tracing::debug;

async fn read_exploit_manifest() -> Result<cli::models::ExploitManifest> {
    let raw = fs::read("exploit.toml").await?;
    let toml = std::str::from_utf8(&raw)?;

    Ok(toml::from_str(toml)?)
}

// TODO: Eventually move to bollard if things work?
async fn build_image(progress: &ProgressBar, tag: &str) -> Result<bool> {
    let start = Instant::now();

    // TODO: Verify if this works on all relevant OSes?
    debug!("Running: docker buildx build --push --pull --tag {tag} .");
    let mut child = Command::new("docker")
        .args(&[
            "buildx",
            "build",
            "--network=host",
            "--push",
            "--pull",
            "--tag",
            tag,
            ".",
        ])
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
    let cli_manifest = match read_exploit_manifest().await {
        Ok(manifest) => manifest,
        Err(err) => {
            eprintln!(
                "  {} {}",
                emoji::CROSS_MARK,
                style("Unable to read the exploit manifest (exploit.toml)")
                    .red()
                    .bold()
            );
            eprintln!("{err}");
            return Ok(());
        }
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

    // TODO: Set up a buildx instance first
    match build_image(&progress, &tag).await {
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
    let update_res = client
        .update_exploit(&models::Exploit {
            manifest,
            image: tag,
        })
        .await;

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
                style("Deployment requested succcessfully").green().bold()
            );
        }
    }

    Ok(())
}
