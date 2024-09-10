use color_eyre::eyre::{self, Context, ContextCompat};
use futures::Future;
use indicatif::ProgressBar;
use models::CliConfig;
use std::{path::PathBuf, time::Duration};

pub(crate) mod args;
pub(crate) mod commands;
mod emoji;
mod models;

#[cfg(not(debug_assertions))]
const CONFIG_FILE_NAME: &str = "cli.toml";
#[cfg(debug_assertions)]
const CONFIG_FILE_NAME: &str = "cli.dev.toml";

fn log(p: &ProgressBar, message: String) {
    p.suspend(|| {
        println!("  {message}");
    });
}

fn format_duration_secs(duration: &Duration) -> String {
    let secs_fractional = duration.as_millis() as f32 / 1000f32;
    format!("{secs_fractional:.2}s")
}

/// Displays a spinner in the console while the future is running. The caller is responsible for
/// displaying a message signifying the completion.
async fn with_spinner<F, Fut, T, E>(message: &'static str, f: F) -> Result<T, E>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(130));
    pb.set_message(message);

    let res = f().await;
    pb.finish_and_clear();

    res
}

fn get_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|path| path.join("kriger"))
}

fn get_config_file() -> Option<PathBuf> {
    get_config_dir().map(|path| path.join(CONFIG_FILE_NAME))
}

async fn read_cli_config() -> eyre::Result<CliConfig> {
    let path = get_config_file().context("unable to locate the config directory")?;
    let content = tokio::fs::read_to_string(path)
        .await
        .context("unable to read the config file")?;
    let config: CliConfig = toml::from_str(&content).context("unable to parse the config file")?;

    Ok(config)
}
