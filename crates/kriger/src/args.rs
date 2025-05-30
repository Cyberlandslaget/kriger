// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use clap_derive::{Parser, Subcommand};

/// An exploit farm for attack/defense CTFs
#[derive(Parser, Debug)]
#[command(version, about)]
#[command(propagate_version = true)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Run the server components
    #[cfg(feature = "server")]
    Server(crate::server::args::Args),
    /// Run the runner component
    #[cfg(feature = "server")]
    Runner(kriger_runner::args::Args),
    /// Deploy an exploit
    #[cfg(feature = "cli")]
    Deploy(crate::cli::args::Deploy),
    /// Create an exploit
    #[cfg(feature = "cli")]
    Create(crate::cli::args::Create),
    /// Manually submit flag(s)
    #[cfg(feature = "cli")]
    Submit(crate::cli::args::Submit),
    /// Exploit-related commands
    #[cfg(feature = "cli")]
    #[command(subcommand)]
    Exploit(crate::cli::args::ExploitCommand),
}
