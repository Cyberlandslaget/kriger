// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use clap_derive::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
#[group(skip)]
pub struct Config {
    /// The socket address to listen to
    #[arg(env, long, default_value = "[::]:8000")]
    pub(crate) rest_listen: String,

    /// The origin(s) to allow CORS for
    #[arg(
        env,
        long,
        default_value = "https://kriger.o99.no,http://localhost:5173",
        value_delimiter = ','
    )]
    pub(crate) rest_cors_origins: Vec<String>,
}
