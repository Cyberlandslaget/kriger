use crate::cli::model::ExploitManifest;
use crate::cli::{args, emoji, format_duration_secs, log};
use color_eyre::eyre::{bail, Context, ContextCompat};
use color_eyre::Result;
use console::style;
use futures::stream::select_all;
use futures::StreamExt;
use indicatif::ProgressBar;
use kriger_common::messaging::model::Exploit;
use reqwest::{Method, StatusCode};
use std::process::Stdio;
use std::time::Duration;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::Instant;
use tokio_stream::wrappers::LinesStream;
use tracing::debug;

async fn read_exploit_manifest() -> Result<ExploitManifest> {
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
    println!("  {} Tag: {}", emoji::PACKAGE, style(tag).yellow());

    Ok(true)
}

async fn launch(exploit: &Exploit, rest_url: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .request(Method::PUT, format!("{rest_url}/launch"))
        .json(&exploit)
        .send()
        .await
        .context("Sending /launch request to REST API")?;

    match response.status() {
        StatusCode::OK => Ok(()),
        StatusCode::INTERNAL_SERVER_ERROR => {
            bail!("Error during launch: {}", response.text().await?)
        }
        _ => bail!("unexpected response: {response:?}"),
    }
}

pub(crate) async fn main(args: args::Deploy) -> Result<()> {
    let manifest = match read_exploit_manifest().await {
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
    println!(
        "  {} Preparing to deploy {}",
        emoji::ROCKET,
        style(&manifest.exploit.name).green().bold()
    );

    let date = chrono::Utc::now();
    let tag_version = format!("{}", date.timestamp());
    let tag = format!(
        "{}/kriger-exploits/{}:{}",
        &args.registry, &manifest.exploit.name, tag_version
    );

    let progress = ProgressBar::new_spinner();
    progress.enable_steady_tick(Duration::from_millis(130));
    progress.set_message(format!(
        "{} Building {}...",
        emoji::HAMMER,
        style(&manifest.exploit.name).green().bold()
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
            }
        }
    }

    if !args.no_launch {
        if let Err(err) = launch(
            &Exploit {
                manifest: manifest.exploit,
                image: tag,
            },
            &args.rest_url,
        )
        .await
        {
            progress.finish_and_clear();
            println!(
                "  {} {}: {:?}",
                emoji::CROSS_MARK,
                style("Launch failed").red().bold(),
                err
            );
        }
    }

    Ok(())
}
