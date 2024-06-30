use anyhow::Result;
use futures::StreamExt;
use tracing::{info, warn};
use kriger_common::messaging::{Message, Messaging};
use kriger_common::runtime::AppRuntime;

pub async fn main(runtime: AppRuntime) -> Result<()> {
    info!("starting runner");

    Ok(())
}
