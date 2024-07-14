use clap_derive::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
#[group(skip)]
pub struct Args {}
