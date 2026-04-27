use anyhow::Result;

use crate::cli::{Cli, Command};

pub async fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Init { target } => init(&target).await,
        Command::Deploy { domain } => deploy(&domain).await,
        Command::Logs { service } => logs(&service).await,
        Command::GenerateKey => generate_key().await,
    }
}

async fn init(target: &str) -> Result<()> {
    println!("[ptto] bootstrap planned for {target}");
    println!("[ptto] next: install Caddy and prepare systemd units");
    Ok(())
}

async fn deploy(domain: &str) -> Result<()> {
    println!("[ptto] deploy pipeline planned for domain {domain}");
    println!("[ptto] next: build binary, transfer over SSH, reload service");
    Ok(())
}

async fn logs(service: &str) -> Result<()> {
    println!("[ptto] log streaming planned for service {service}");
    Ok(())
}

async fn generate_key() -> Result<()> {
    println!("[ptto] key generation hook planned for CI/CD");
    Ok(())
}
