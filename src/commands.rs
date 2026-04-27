use anyhow::Result;

use crate::cli::{Cli, Command};

pub fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Init { target } => init(&target),
        Command::Deploy { domain } => deploy(&domain),
        Command::Logs { service } => logs(&service),
        Command::GenerateKey => generate_key(),
    }
}

fn init(target: &str) -> Result<()> {
    println!("[ptto] bootstrap planned for {target}");
    println!("[ptto] next: install Caddy and prepare systemd units");
    Ok(())
}

fn deploy(domain: &str) -> Result<()> {
    println!("[ptto] deploy pipeline planned for domain {domain}");
    println!("[ptto] next: build binary, transfer over SSH, reload service");
    Ok(())
}

fn logs(service: &str) -> Result<()> {
    println!("[ptto] log streaming planned for service {service}");
    Ok(())
}

fn generate_key() -> Result<()> {
    println!("[ptto] key generation hook planned for CI/CD");
    Ok(())
}
