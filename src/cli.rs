use clap::*;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub op: Op,
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbosity: u8,
}
#[derive(Debug, Subcommand)]
pub enum Op {
    Set { brightness: u8 },
    Get,
}
