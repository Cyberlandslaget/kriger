use bollard::image::{BuildImageOptions, BuilderVersion};
use bollard::models::BuildInfo;
use bollard::secret::BuildInfoAux;
use bollard::Docker;
use color_eyre::Result;
use futures::StreamExt;
use tokio::fs;

use crate::cli::args;
use crate::cli::model::ExploitManifest;

async fn read_exploit_manifest() -> Result<ExploitManifest> {
    let raw = fs::read("exploit.toml").await?;
    let toml = std::str::from_utf8(&raw)?;

    Ok(toml::from_str(toml)?)
}

pub(crate) async fn main(args: args::Deploy) -> Result<()> {
    let manifest = match read_exploit_manifest().await {
        Ok(manifest) => manifest,
        Err(err) => {
            println!("unable to read the exploit manifest (exploit.toml)");
            return Err(err);
        }
    };
    println!("\u{1F680} Preparing to deploy {}", &manifest.exploit.name);

    let docker = Docker::connect_with_local_defaults()?;

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

    println!("\u{1F528} Building... (ctx: {} B)", &tar.len());

    // FIXME: Currently "broken": https://github.com/fussybeaver/bollard/issues/428
    let mut stream = docker.build_image(opt, None, Some(tar.into()));
    while let Some(res) = stream.next().await {
        match res {
            Ok(BuildInfo {
                aux: Some(BuildInfoAux::BuildKit(status)),
                ..
            }) => {
                for v in status.vertexes {
                    println!("{}: {:?}", &v.name, &v.completed);
                    if !v.error.is_empty() {
                        println!("  -> {}", &v.error);
                    }
                }
            }
            Ok(msg) => {
                println!("{msg:?}");
            }
            Err(err) => {
                println!("{err}");
            }
        }
    }

    Ok(())
}
