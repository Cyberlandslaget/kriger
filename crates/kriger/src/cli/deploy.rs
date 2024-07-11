use crate::cli::args;
use crate::cli::model::ExploitManifest;
use color_eyre::Result;
use tokio::fs;

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
    println!("\u{1F680} Deploying {}", &manifest.exploit.name);
    Ok(())
}
