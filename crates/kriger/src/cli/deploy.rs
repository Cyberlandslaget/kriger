use std::process::Stdio;
use std::time::Duration;

use bollard::image::{BuildImageOptions, BuilderVersion};
use bollard::models::BuildInfo;
use bollard::secret::BuildInfoAux;
use bollard::Docker;
use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::Result;
use console::style;
use futures::stream::select_all;
use futures::StreamExt;
use indicatif::ProgressBar;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::Instant;
use tokio_stream::wrappers::LinesStream;

use crate::cli::model::ExploitManifest;
use crate::cli::{args, emoji, format_duration_secs, log};

async fn read_exploit_manifest() -> Result<ExploitManifest> {
    let raw = fs::read("exploit.toml").await?;
    let toml = std::str::from_utf8(&raw)?;

    Ok(toml::from_str(toml)?)
}

async fn build_image_bollard(docker: &Docker) -> Result<()> {
    let start = Instant::now();

    let opt = BuildImageOptions {
        t: "kriger-exploit:test",
        dockerfile: "Dockerfile",
        version: BuilderVersion::BuilderBuildKit,
        pull: true,
        rm: true,
        forcerm: true,
        session: Some("kriger-cli".to_string()),
        ..Default::default()
    };

    // TODO: Honor dockerignore
    let mut tar = tar::Builder::new(Vec::new());
    tar.append_dir_all(".", ".")?;
    tar.finish()?;
    let tar = tar.into_inner()?;

    let progress = ProgressBar::new_spinner();
    progress.enable_steady_tick(Duration::from_millis(130));

    progress.set_message(format!(
        "{} Building... (ctx: {} B)",
        emoji::HAMMER,
        &tar.len()
    ));

    // FIXME: Currently "broken": https://github.com/fussybeaver/bollard/issues/428
    // TODO: Retrieve the final image digest and return an error if the build failed
    let mut stream = docker.build_image(opt, None, Some(tar.into()));
    while let Some(res) = stream.next().await {
        match res {
            Ok(BuildInfo {
                aux: Some(BuildInfoAux::BuildKit(status)),
                ..
            }) => {
                for v in status.vertexes {
                    log(
                        &progress,
                        format!("{}: {:?}", style(&v.name).blue(), &v.completed),
                    );
                    if !v.error.is_empty() {
                        log(
                            &progress,
                            format!("  {} {}", style("->").blue(), style(&v.error).red()),
                        );
                    }
                }
            }
            Ok(msg) => {
                log(&progress, format!("{msg:?}"));
            }
            Err(err) => {
                log(&progress, format!("{err}"));
            }
        }
    }

    progress.finish_and_clear();

    let elapsed = start.elapsed();
    println!(
        "  {} Built in {}",
        emoji::SPARKLES,
        style(format_duration_secs(&elapsed)).bold().green()
    );

    Ok(())
}

// TODO: Eventually move to bollard if things work?
async fn build_image(
    progress: &ProgressBar,
    manifest: &ExploitManifest,
    tag: &str,
) -> Result<bool> {
    let start = Instant::now();

    // TODO: Verify if this works on all relevant OSes?
    let mut child = Command::new("docker")
        .args(&["buildx", "build", "--push", "--pull", "--tag", tag, "."])
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
        "  {} Built in {}",
        emoji::SPARKLES,
        style(format_duration_secs(&elapsed)).bold().green()
    );

    Ok(true)
}

async fn push_image(progress: &ProgressBar, manifest: &ExploitManifest, tag: &str) -> Result<bool> {
    let start = Instant::now();

    // TODO: Verify if this works on all relevant OSes?
    let mut child = Command::new("docker")
        .args(&["image", "push", tag])
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
        "  {} Pushed in {}",
        emoji::SPARKLES,
        style(format_duration_secs(&elapsed)).bold().green()
    );

    Ok(true)
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
        "r.o99.no/kriger-exploits/{}:{}",
        &manifest.exploit.name, tag_version
    );

    let progress = ProgressBar::new_spinner();
    progress.enable_steady_tick(Duration::from_millis(130));
    progress.set_message(format!("{} Building...", emoji::HAMMER));

    match build_image(&progress, &manifest, &tag).await {
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
                return Ok(());
            }
        }
    }

    // let progress = ProgressBar::new_spinner();
    // progress.enable_steady_tick(Duration::from_millis(130));
    // progress.set_message(format!("{} Pushing...", emoji::ROCKET));
    // 
    // match push_image(&progress, &manifest, &tag).await {
    //     Err(err) => {
    //         progress.finish_and_clear();
    //         println!(
    //             "  {} {}",
    //             emoji::CROSS_MARK,
    //             style("Push failed").red().bold()
    //         );
    //         return Err(err);
    //     }
    //     Ok(success) => {
    //         if !success {
    //             progress.finish_and_clear();
    //             println!(
    //                 "  {} {}",
    //                 emoji::CROSS_MARK,
    //                 style("Push failed").red().bold()
    //             );
    //             return Ok(());
    //         }
    //     }
    // }

    Ok(())
}
