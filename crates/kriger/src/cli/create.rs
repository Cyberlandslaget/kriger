use crate::cli;
use crate::cli::{args, emoji};
use color_eyre::eyre;
use color_eyre::eyre::{bail, Context};
use console::style;
use kriger_common::models;

// TODO: Handle the error in a user friendly way
pub(crate) async fn main(args: args::Create) -> eyre::Result<()> {
    let exploit_name = inquire_text(args.name, "Exploit name:")?;

    let exists = tokio::fs::metadata(&exploit_name)
        .await
        .map_or_else(|_| false, |m| m.is_dir());

    if exists {
        bail!("the exploit already exists");
    }

    let manifest = create_manifest(&exploit_name);
    let manifest_toml =
        toml::to_string_pretty(&manifest).context("unable to serialize the exploit manifest")?;

    tokio::fs::create_dir(&exploit_name)
        .await
        .context("unable to create the exploit directory")?;

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

fn create_manifest(name: &str) -> cli::model::ExploitManifest {
    cli::model::ExploitManifest {
        image: None,
        exploit: models::ExploitManifest {
            name: name.to_string(),
            service: "".to_string(),
            replicas: 1,
            workers: Some(4),
            enabled: true,
            resources: models::ExploitResources {
                cpu_request: Some("1".to_string()),
                mem_request: Some("256Mi".to_string()),
                cpu_limit: "2".to_string(),
                mem_limit: "512Mi".to_string(),
                timeout: 10,
            },
        },
    }
}
