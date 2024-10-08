use crate::cli::{args, read_cli_config};
use color_eyre::eyre;
use color_eyre::eyre::{bail, Context};
use console::style;
use kriger_common::client::KrigerClient;
use kriger_common::models;
use regex::Regex;

pub(crate) async fn main(args: args::Submit) -> eyre::Result<()> {
    let cli_config = read_cli_config().await?;
    let client = KrigerClient::new(cli_config.client.rest_url);

    let server_config = match client.get_server_config().await? {
        models::responses::AppResponse::Ok(team_map) => team_map,
        models::responses::AppResponse::Error { message } => bail!(message),
    };
    let flag_format =
        Regex::new(&server_config.competition.flag_format).context("invalid regex")?;

    let flags: Vec<String> = flag_format
        .find_iter(&args.input)
        .map(|flag| flag.as_str().to_string())
        .collect();
    let flag_count = flags.len();

    client
        .submit_flags(flags)
        .await
        .context("unable to submit flags")?;

    eprintln!(
        "{}",
        style(format!("Queued {} flags for submission", flag_count)).green()
    );

    Ok(())
}
