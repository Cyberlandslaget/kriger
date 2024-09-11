use crate::cli;
use crate::cli::emoji;
use console::style;
use tokio::fs;

pub(crate) mod create;
pub(crate) mod deploy;
pub(crate) mod exploit;

pub(crate) async fn read_exploit_manifest() -> color_eyre::Result<cli::models::ExploitManifest> {
    let raw = fs::read("exploit.toml").await?;
    let toml = std::str::from_utf8(&raw)?;

    Ok(toml::from_str(toml)?)
}

pub(crate) async fn acquire_exploit_manifest() -> Option<cli::models::ExploitManifest> {
    match read_exploit_manifest().await {
        Ok(manifest) => Some(manifest),
        Err(err) => {
            eprintln!(
                "  {} {}",
                emoji::CROSS_MARK,
                style("Unable to read the exploit manifest (exploit.toml)")
                    .red()
                    .bold()
            );
            eprintln!("{err}");
            None
        }
    }
}
