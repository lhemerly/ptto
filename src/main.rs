use anyhow::Result;
use clap::Parser;

use ptto::{cli::Cli, commands};

fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::dispatch(cli)
}
