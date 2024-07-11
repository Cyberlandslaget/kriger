use clap_derive::Parser;

/// An exploit farm for attack/defense CTFs
#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) struct Deploy {}
