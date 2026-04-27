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

    for command in caddy_init_commands() {
        ssh.run(&command)?;
    }

    println!("[ptto] server init complete (Caddy installed and started)");
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

fn caddy_init_commands() -> Vec<String> {
    vec![
        concat!(
            "set -eu; ",
            "if [ \"$(id -u)\" -eq 0 ]; then SUDO=\"\"; ",
            "elif command -v sudo >/dev/null 2>&1; then ",
            "if sudo -n true >/dev/null 2>&1; then SUDO=\"sudo\"; ",
            "else echo \"[ptto] error: passwordless sudo is required for non-interactive init\"; exit 1; fi; ",
            "else echo \"[ptto] error: root or sudo is required\"; exit 1; fi; ",
            "if command -v caddy >/dev/null 2>&1; then ",
            "echo \"[ptto] Caddy already installed\"; ",
            "else ",
            "if ! command -v apt-get >/dev/null 2>&1; then ",
            "echo \"[ptto] error: apt-get is required (Ubuntu/Debian)\"; exit 1; ",
            "fi; ",
            "$SUDO apt-get update; ",
            "$SUDO apt-get install -y debian-keyring debian-archive-keyring apt-transport-https curl gnupg; ",
            "$SUDO mkdir -p /usr/share/keyrings; ",
            "tmp_gpg=\"$(mktemp)\"; tmp_list=\"$(mktemp)\"; ",
            "trap 'rm -f \"$tmp_gpg\" \"$tmp_list\"' EXIT; ",
            "curl -1sLf https://dl.cloudsmith.io/public/caddy/stable/gpg.key -o \"$tmp_gpg\"; ",
            "curl -1sLf https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt -o \"$tmp_list\"; ",
            "$SUDO gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg \"$tmp_gpg\"; ",
            "$SUDO mv \"$tmp_list\" /etc/apt/sources.list.d/caddy-stable.list; ",
            "$SUDO apt-get update; ",
            "$SUDO apt-get install -y caddy; ",
            "fi"
        )
        .to_string(),
        concat!(
            "set -eu; ",
            "if [ \"$(id -u)\" -eq 0 ]; then SUDO=\"\"; ",
            "elif command -v sudo >/dev/null 2>&1; then ",
            "if sudo -n true >/dev/null 2>&1; then SUDO=\"sudo\"; ",
            "else echo \"[ptto] error: passwordless sudo is required for non-interactive init\"; exit 1; fi; ",
            "else echo \"[ptto] error: root or sudo is required\"; exit 1; fi; ",
            "$SUDO systemctl enable --now caddy; ",
            "$SUDO systemctl status caddy --no-pager --lines=0"
        )
        .to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::caddy_init_commands;

    #[test]
    fn caddy_init_contains_install_and_service_steps() {
        let commands = caddy_init_commands();
        assert_eq!(commands.len(), 2);
        assert!(commands[0].contains("apt-get install -y caddy"));
        assert!(commands[1].contains("systemctl enable --now caddy"));
        assert!(commands[0].contains("sudo -n true"));
        assert!(commands[1].contains("sudo -n true"));
        assert!(commands[0]
            .contains("curl -1sLf https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt -o"));
    }
}
