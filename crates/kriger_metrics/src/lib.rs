use color_eyre::eyre::Result;
use tracing::info;

pub async fn main() -> Result<()> {
    info!("starting metrics exporter");
    Ok(())
}
