// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use clap_derive::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
#[group(skip)]
pub struct Args {}
