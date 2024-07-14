mod submitter;

use color_eyre::eyre::Result;
use kriger_common::runtime::AppRuntime;
use tracing::info;

pub async fn main(runtime: AppRuntime) -> Result<()> {
    info!("starting submitter");
    Ok(())
}
