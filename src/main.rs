use anyhow::Result;
use clap::Parser;

use ptto::{cli::Cli, commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::dispatch(cli).await
}
