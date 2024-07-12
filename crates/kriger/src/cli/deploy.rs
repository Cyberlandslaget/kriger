use std::time::Duration;

use bollard::image::{BuildImageOptions, BuilderVersion};
use bollard::models::BuildInfo;
use bollard::secret::BuildInfoAux;
use bollard::Docker;
use color_eyre::Result;
use console::style;
use futures::StreamExt;
use indicatif::ProgressBar;
use tokio::fs;
use tokio::time::Instant;

use crate::cli::model::ExploitManifest;
use crate::cli::{args, emoji, format_duration_secs, log};

async fn read_exploit_manifest() -> Result<ExploitManifest> {
    let raw = fs::read("exploit.toml").await?;
    let toml = std::str::from_utf8(&raw)?;

    Ok(toml::from_str(toml)?)
}

async fn build_image(docker: &Docker) -> Result<()> {
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

pub(crate) async fn main(args: args::Deploy) -> Result<()> {
    let manifest = match read_exploit_manifest().await {
        Ok(manifest) => manifest,
        Err(err) => {
            println!("unable to read the exploit manifest (exploit.toml)");
            return Err(err);
        }
    };
    println!(
        "  {} Preparing to deploy {}",
        emoji::ROCKET,
        style(&manifest.exploit.name).green().bold()
    );

    let docker = Docker::connect_with_local_defaults()?;
    build_image(&docker).await?;

    Ok(())
}
