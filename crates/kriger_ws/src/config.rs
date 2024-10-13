// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use clap_derive::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
#[group(skip)]
pub struct Config {
    /// The socket address to listen to
    #[arg(env, long, default_value = "[::]:8001")]
    pub(crate) ws_listen: String,
}
