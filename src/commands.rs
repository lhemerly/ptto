use std::path::Path;

use anyhow::Result;

use crate::{
    cli::{Cli, Command},
    ssh::SshClient,
};

pub fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Init { target, dry_run } => init(&target, dry_run),
        Command::Deploy {
            domain,
            target,
            artifact,
            dry_run,
        } => deploy(&domain, &target, &artifact, dry_run),
        Command::Logs { service } => logs(&service),
        Command::GenerateKey => generate_key(),
    }
}

fn init(target: &str, dry_run: bool) -> Result<()> {
    println!("[ptto] bootstrap starting for {target}");
    let ssh = SshClient::new(target, dry_run);
    ssh.run("echo '[ptto] SSH connectivity check succeeded'")?;
    println!("[ptto] ssh engine ready for next phase (server init)");
    Ok(())
}

fn deploy(domain: &str, target: &str, artifact: &str, dry_run: bool) -> Result<()> {
    println!("[ptto] deploy pipeline planned for domain {domain}");
    let ssh = SshClient::new(target, dry_run);
    ssh.copy_file(Path::new(artifact), "/tmp/ptto-app")?;
    println!("[ptto] artifact staged over ssh at /tmp/ptto-app");
    println!("[ptto] next: build wrapper + systemd + caddy wiring");
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
