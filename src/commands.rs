use std::path::Path;
use std::process::Command as ProcessCommand;
use std::{ffi::OsString, fs};

use anyhow::{bail, Context, Result};

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
            source,
            dry_run,
        } => deploy(&domain, &target, &artifact, &source, dry_run),
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

fn deploy(domain: &str, target: &str, artifact: &str, source: &str, dry_run: bool) -> Result<()> {
    println!("[ptto] deploy pipeline planned for domain {domain}");
    build_go_linux_amd64_binary(source, artifact, dry_run)?;
    let ssh = SshClient::new(target, dry_run);
    ssh.copy_file(Path::new(artifact), "/tmp/ptto-app")?;
    println!("[ptto] artifact staged over ssh at /tmp/ptto-app");

    for command in systemd_deploy_commands() {
        ssh.run(&command)?;
    }

    println!("[ptto] systemd service generated, reloaded, and restarted");
    println!("[ptto] next: caddy routing wiring");
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

fn build_go_linux_amd64_binary(source: &str, artifact: &str, dry_run: bool) -> Result<()> {
    ensure_artifact_parent_dir(artifact)?;
    let command_preview = go_build_command_preview(source, artifact);
    if dry_run {
        println!("[ptto] dry-run: {command_preview}");
        return Ok(());
    }

    println!("[ptto] compiling with: {command_preview}");
    let status = ProcessCommand::new("go")
        .env("GOOS", "linux")
        .env("GOARCH", "amd64")
        .arg("build")
        .arg("-o")
        .arg(artifact)
        .arg(source)
        .status()
        .context("failed to start go build process")?;

    if !status.success() {
        bail!("go build failed with status {status}");
    }

    Ok(())
}

fn ensure_artifact_parent_dir(artifact: &str) -> Result<()> {
    let artifact_path = Path::new(artifact);
    if let Some(parent) = artifact_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create parent directory for artifact output: {}",
                parent.display()
            )
        })?;
    }

    Ok(())
}

fn go_build_command_preview(source: &str, artifact: &str) -> String {
    format!(
        "GOOS=linux GOARCH=amd64 go build -o {} {}",
        shell_quote(artifact),
        shell_quote(source)
    )
}

fn shell_quote(value: &str) -> String {
    let quoted = OsString::from(value);
    format!("{quoted:?}")
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

fn systemd_deploy_commands() -> Vec<String> {
    vec![
        concat!(
            "set -eu; ",
            "if [ \"$(id -u)\" -eq 0 ]; then SUDO=\"\"; ",
            "elif command -v sudo >/dev/null 2>&1; then ",
            "if sudo -n true >/dev/null 2>&1; then SUDO=\"sudo\"; ",
            "else echo \"[ptto] error: passwordless sudo is required for non-interactive deploy\"; exit 1; fi; ",
            "else echo \"[ptto] error: root or sudo is required\"; exit 1; fi; ",
            "$SUDO install -d -m 755 /opt/ptto/bin; ",
            "$SUDO install -m 755 /tmp/ptto-app /opt/ptto/bin/ptto-app"
        )
        .to_string(),
        concat!(
            "set -eu; ",
            "if [ \"$(id -u)\" -eq 0 ]; then SUDO=\"\"; ",
            "elif command -v sudo >/dev/null 2>&1; then ",
            "if sudo -n true >/dev/null 2>&1; then SUDO=\"sudo\"; ",
            "else echo \"[ptto] error: passwordless sudo is required for non-interactive deploy\"; exit 1; fi; ",
            "else echo \"[ptto] error: root or sudo is required\"; exit 1; fi; ",
            "tmp_service=\"$(mktemp)\"; ",
            "trap 'rm -f \"$tmp_service\"' EXIT; ",
            "cat > \"$tmp_service\" <<'EOF'\n",
            "[Unit]\n",
            "Description=ptto app service\n",
            "After=network-online.target\n",
            "Wants=network-online.target\n\n",
            "[Service]\n",
            "Type=simple\n",
            "User=root\n",
            "WorkingDirectory=/opt/ptto\n",
            "ExecStart=/opt/ptto/bin/ptto-app\n",
            "Restart=always\n",
            "RestartSec=2\n",
            "Environment=PORT=8080\n\n",
            "[Install]\n",
            "WantedBy=multi-user.target\n",
            "EOF\n",
            "$SUDO mv \"$tmp_service\" /etc/systemd/system/ptto-app.service; ",
            "$SUDO chmod 644 /etc/systemd/system/ptto-app.service; ",
            "$SUDO systemctl daemon-reload; ",
            "$SUDO systemctl enable --now ptto-app; ",
            "$SUDO systemctl restart ptto-app; ",
            "$SUDO systemctl status ptto-app --no-pager --lines=0"
        )
        .to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        build_go_linux_amd64_binary, caddy_init_commands, ensure_artifact_parent_dir,
        go_build_command_preview, systemd_deploy_commands,
    };

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

    #[test]
    fn systemd_deploy_contains_install_and_reload_steps() {
        let commands = systemd_deploy_commands();
        assert_eq!(commands.len(), 2);
        assert!(commands[0].contains("install -m 755 /tmp/ptto-app /opt/ptto/bin/ptto-app"));
        assert!(commands[1].contains("cat > \"$tmp_service\" <<'EOF'"));
        assert!(commands[1].contains("ExecStart=/opt/ptto/bin/ptto-app"));
        assert!(commands[1].contains("systemctl daemon-reload"));
        assert!(commands[1].contains("systemctl enable --now ptto-app"));
        assert!(commands[1].contains("systemctl restart ptto-app"));
        assert!(commands[0].contains("sudo -n true"));
        assert!(commands[1].contains("sudo -n true"));
    }

    #[test]
    fn go_build_wrapper_is_dry_run_safe() {
        let result = build_go_linux_amd64_binary("./cmd/server", "./app", true);
        assert!(result.is_ok());
    }

    #[test]
    fn go_build_preview_uses_quoted_values() {
        let preview = go_build_command_preview("./cmd/my server", "./dist/my app");
        assert_eq!(
            preview,
            "GOOS=linux GOARCH=amd64 go build -o \"./dist/my app\" \"./cmd/my server\""
        );
    }

    #[test]
    fn ensures_artifact_parent_directory_exists() {
        let temp_dir = tempfile::tempdir().expect("tempdir should be created");
        let artifact_path: PathBuf = temp_dir.path().join("dist").join("app");
        ensure_artifact_parent_dir(
            artifact_path
                .to_str()
                .expect("artifact path should be valid UTF-8"),
        )
        .expect("parent directory should be created");
        assert!(temp_dir.path().join("dist").exists());
    }
}
