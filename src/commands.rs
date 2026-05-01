use std::fs;
use std::path::Path;
use std::process::Command as ProcessCommand;

use anyhow::{bail, Context, Result};

use crate::{
    cli::{Cli, Command, DbCommand},
    config::PttoConfig,
    ssh::SshClient,
};

const APP_INTERNAL_PORT: u16 = 8080;
const REMOTE_DB_PATH: &str = "/opt/ptto/data/database.sqlite";

pub fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Init { target, dry_run } => {
            let config = PttoConfig::load()?;
            let target = resolve_target(target, &config)?;
            init(&target, config.ssh_key.as_deref(), dry_run)
        }
        Command::Deploy {
            domain,
            target,
            artifact,
            source,
            dry_run,
        } => {
            let config = PttoConfig::load()?;
            let domain = resolve_domain(domain, &config)?;
            let target = resolve_target(target, &config)?;
            deploy(
                &domain,
                &target,
                config.ssh_key.as_deref(),
                &artifact,
                &source,
                dry_run,
            )
        }
        Command::Logs { service, target } => {
            let config = PttoConfig::load()?;
            let target = resolve_target_for_telemetry(target, &config)?;
            let ssh = SshClient::new(target, config.ssh_key.as_deref(), false);
            logs(&service, &ssh)
        }
        Command::Top { target } => {
            let config = PttoConfig::load()?;
            let target = resolve_target_for_telemetry(target, &config)?;
            let ssh = SshClient::new(target, config.ssh_key.as_deref(), false);
            top(&ssh)
        }
        Command::Traffic { target } => {
            let config = PttoConfig::load()?;
            let target = resolve_target_for_telemetry(target, &config)?;
            let ssh = SshClient::new(target, config.ssh_key.as_deref(), false);
            traffic(&ssh)
        }
        Command::Db { target, command } => {
            let config = PttoConfig::load()?;
            let target = resolve_target_for_db(target, &config)?;
            let ssh = SshClient::new(target, config.ssh_key.as_deref(), false);
            db(command, &ssh)
        }
        Command::GenerateKey => generate_key(),
    }
}

fn db(command: DbCommand, ssh: &SshClient) -> Result<()> {
    match command {
        DbCommand::Shell => {
            println!("[ptto] opening remote sqlite shell at {REMOTE_DB_PATH}");
            ssh.run_interactive(&format!(
                "set -eu; {}; $SUDO sqlite3 {}",
                sudo_prefix("db shell"),
                shell_quote(REMOTE_DB_PATH)
            ))
        }
        DbCommand::Pull { local_path } => {
            println!("[ptto] pulling remote database from {REMOTE_DB_PATH} to {local_path}");
            ensure_artifact_parent_dir(&local_path)?;
            let temp_remote = "/tmp/ptto-db-pull.sqlite";
            ssh.run(&format!(
                concat!("set -eu; ", "{}", "$SUDO install -m 600 {} {};"),
                sudo_prefix("db pull"),
                shell_quote(REMOTE_DB_PATH),
                shell_quote(temp_remote)
            ))?;
            let copy_result = ssh.copy_file_from_remote(temp_remote, Path::new(&local_path));
            let cleanup_result = ssh.run(&format!("set -eu; rm -f {}", shell_quote(temp_remote)));
            copy_result?;
            cleanup_result
        }
        DbCommand::Push { local_path } => {
            println!("[ptto] pushing local database {local_path} to {REMOTE_DB_PATH}");
            let local = Path::new(&local_path);
            if !local.exists() {
                bail!("local database file does not exist: {}", local.display());
            }
            ssh.copy_file(local, "/tmp/ptto-database.sqlite")?;
            ssh.run(&format!(
                concat!(
                    "set -eu; ",
                    "{}",
                    "$SUDO install -d -m 755 /opt/ptto/data; ",
                    "$SUDO systemctl stop ptto-app; ",
                    "trap '$SUDO systemctl start ptto-app' EXIT; ",
                    "tmp_db=\"/opt/ptto/data/.database.sqlite.ptto-tmp-$$\"; ",
                    "$SUDO install -m 640 /tmp/ptto-database.sqlite \"$tmp_db\"; ",
                    "$SUDO mv -f \"$tmp_db\" {}; ",
                    "$SUDO rm -f /tmp/ptto-database.sqlite"
                ),
                sudo_prefix("db push"),
                shell_quote(REMOTE_DB_PATH)
            ))
        }
    }
}

fn init(target: &str, ssh_key: Option<&str>, dry_run: bool) -> Result<()> {
    println!("[ptto] bootstrap starting for {target}");
    let ssh = SshClient::new(target, ssh_key, dry_run);
    ssh.run("echo '[ptto] SSH connectivity check succeeded'")?;

    for command in caddy_init_commands() {
        ssh.run(&command)?;
    }

    println!("[ptto] server init complete (Caddy/goaccess installed and Caddy started)");
    Ok(())
}

fn deploy(
    domain: &str,
    target: &str,
    ssh_key: Option<&str>,
    artifact: &str,
    source: &str,
    dry_run: bool,
) -> Result<()> {
    validate_domain(domain)?;
    println!("[ptto] deploy pipeline planned for domain {domain}");
    build_go_linux_amd64_binary(source, artifact, dry_run)?;
    let ssh = SshClient::new(target, ssh_key, dry_run);
    ssh.copy_file(Path::new(artifact), "/tmp/ptto-app")?;
    println!("[ptto] artifact staged over ssh at /tmp/ptto-app");

    for command in systemd_deploy_commands(APP_INTERNAL_PORT) {
        ssh.run(&command)?;
    }
    for command in caddy_routing_commands(domain, APP_INTERNAL_PORT) {
        ssh.run(&command)?;
    }

    println!("[ptto] systemd service generated, reloaded, and restarted");
    println!("[ptto] caddy routing generated and reloaded");
    Ok(())
}

fn resolve_target(cli_target: Option<String>, config: &PttoConfig) -> Result<String> {
    cli_target
        .or_else(|| config.host.clone())
        .context("missing SSH target: pass --target (deploy) or positional target (init), or set host in .ptto.toml")
}

fn resolve_target_for_db(cli_target: Option<String>, config: &PttoConfig) -> Result<String> {
    cli_target
        .or_else(|| config.host.clone())
        .context("missing SSH target: pass --target to ptto db, or set host in .ptto.toml")
}

fn resolve_target_for_telemetry(cli_target: Option<String>, config: &PttoConfig) -> Result<String> {
    cli_target
        .or_else(|| config.host.clone())
        .context("missing SSH target: pass --target or set host in .ptto.toml")
}

fn resolve_domain(cli_domain: Option<String>, config: &PttoConfig) -> Result<String> {
    cli_domain
        .or_else(|| config.domain.clone())
        .context("missing domain: pass --domain or set domain in .ptto.toml")
}

fn logs(service: &str, ssh: &SshClient) -> Result<()> {
    validate_systemd_unit_name(service)?;
    println!("[ptto] streaming logs for service {service}");
    ssh.run_interactive(&format!(
        "set -eu; {}; $SUDO journalctl -u {} -f --no-pager",
        sudo_prefix("logs"),
        shell_quote(service)
    ))
}

fn validate_systemd_unit_name(service: &str) -> Result<()> {
    if service.is_empty() || service.len() > 256 {
        bail!("invalid service name: expected 1-256 characters");
    }
    if !service
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ':' | '_' | '.' | '@' | '-'))
    {
        bail!("invalid service name: allowed characters are A-Z a-z 0-9 : _ . @ -");
    }
    Ok(())
}

fn top(ssh: &SshClient) -> Result<()> {
    println!("[ptto] opening remote process dashboard");
    ssh.run_interactive(
        "set -eu; if command -v htop >/dev/null 2>&1; then exec htop; elif command -v btop >/dev/null 2>&1; then exec btop; elif command -v top >/dev/null 2>&1; then exec top; else echo '[ptto] error: no top utility found (expected htop, btop, or top)'; exit 1; fi",
    )
}

fn traffic(ssh: &SshClient) -> Result<()> {
    println!("[ptto] streaming caddy access telemetry via goaccess");
    ssh.run_interactive(&format!(
        "set -eu; {}; if ! command -v goaccess >/dev/null 2>&1; then echo '[ptto] error: goaccess is not installed (run ptto init)'; exit 1; fi; if [ -f /var/log/caddy/ptto-access.log ]; then log_file=/var/log/caddy/ptto-access.log; elif [ -f /var/log/caddy/access.log ]; then log_file=/var/log/caddy/access.log; else echo '[ptto] error: no Caddy access log found at /var/log/caddy/ptto-access.log or /var/log/caddy/access.log'; exit 1; fi; $SUDO test -r \"$log_file\"; $SUDO tail -F \"$log_file\" | goaccess --log-format=CADDY -",
        sudo_prefix("traffic")
    ))
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
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn caddy_init_commands() -> Vec<String> {
    vec![
        format!(
            concat!(
                "set -eu; ",
                "{}",
                "if command -v caddy >/dev/null 2>&1; then ",
                "echo \"[ptto] Caddy already installed\"; ",
                "if ! command -v goaccess >/dev/null 2>&1; then ",
                "if ! command -v apt-get >/dev/null 2>&1; then ",
                "echo \"[ptto] error: goaccess install requires apt-get (Ubuntu/Debian)\"; exit 1; ",
                "fi; ",
                "$SUDO apt-get update; $SUDO apt-get install -y goaccess; fi; ",
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
                "$SUDO apt-get install -y caddy goaccess; ",
                "fi"
            ),
            sudo_prefix("init")
        ),
        format!(
            concat!(
                "set -eu; ",
                "{}",
                "$SUDO systemctl enable --now caddy; ",
                "$SUDO systemctl status caddy --no-pager --lines=0"
            ),
            sudo_prefix("init")
        ),
    ]
}

fn systemd_deploy_commands(internal_port: u16) -> Vec<String> {
    vec![
        format!(
            concat!(
                "set -eu; ",
                "{}",
                "$SUDO install -d -m 755 /opt/ptto/bin; ",
                "$SUDO install -m 755 /tmp/ptto-app /opt/ptto/bin/ptto-app"
            ),
            sudo_prefix("deploy")
        ),
        format!(
            concat!(
                "set -eu; ",
                "{}",
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
                "Environment=PORT={internal_port}\n\n",
                "[Install]\n",
                "WantedBy=multi-user.target\n",
                "EOF\n",
                "$SUDO mv \"$tmp_service\" /etc/systemd/system/ptto-app.service; ",
                "$SUDO chmod 644 /etc/systemd/system/ptto-app.service; ",
                "$SUDO systemctl daemon-reload; ",
                "$SUDO systemctl enable --now ptto-app; ",
                "$SUDO systemctl restart ptto-app; ",
                "$SUDO systemctl status ptto-app --no-pager --lines=0"
            ),
            sudo_prefix("deploy"),
            internal_port = internal_port
        ),
    ]
}

fn caddy_routing_commands(domain: &str, internal_port: u16) -> Vec<String> {
    let caddyfile = format!(
        "{domain} {{\n    reverse_proxy 127.0.0.1:{internal_port}\n    log {{\n        output file /var/log/caddy/ptto-access.log\n        format console\n    }}\n}}\n"
    );
    vec![format!(
        concat!(
            "set -eu; ",
            "{}",
            "tmp_caddy=\"$(mktemp)\"; ",
            "trap 'rm -f \"$tmp_caddy\"' EXIT; ",
            "printf '%s' {} > \"$tmp_caddy\"; ",
            "$SUDO caddy validate --config \"$tmp_caddy\"; ",
            "backup_dir=\"/etc/caddy/backups\"; ",
            "if [ -f /etc/caddy/Caddyfile ]; then ",
            "$SUDO install -d -m 755 \"$backup_dir\"; ",
            "$SUDO cp /etc/caddy/Caddyfile \"$backup_dir/Caddyfile.$(date +%Y%m%d%H%M%S).bak\"; ",
            "fi; ",
            "$SUDO mv \"$tmp_caddy\" /etc/caddy/Caddyfile; ",
            "$SUDO chmod 644 /etc/caddy/Caddyfile; ",
            "$SUDO systemctl reload caddy; ",
            "$SUDO systemctl status caddy --no-pager --lines=0"
        ),
        sudo_prefix("deploy"),
        shell_quote(&caddyfile)
    )]
}

fn sudo_prefix(phase: &str) -> String {
    format!(
        concat!(
            "if [ \"$(id -u)\" -eq 0 ]; then SUDO=\"\"; ",
            "elif command -v sudo >/dev/null 2>&1; then ",
            "if sudo -n true >/dev/null 2>&1; then SUDO=\"sudo\"; ",
            "else echo \"[ptto] error: passwordless sudo is required for non-interactive {phase}\"; exit 1; fi; ",
            "else echo \"[ptto] error: root or sudo is required\"; exit 1; fi; "
        ),
        phase = phase
    )
}

fn validate_domain(domain: &str) -> Result<()> {
    if domain.is_empty() || domain.len() > 253 {
        bail!("invalid domain: must be 1-253 characters");
    }
    if domain
        .chars()
        .any(|ch| ch.is_ascii_whitespace() || ch.is_control())
    {
        bail!("invalid domain: whitespace/control characters are not allowed");
    }
    if !domain.is_ascii() {
        bail!("invalid domain: only ASCII DNS-style domains are currently supported");
    }

    let labels: Vec<&str> = domain.split('.').collect();
    if labels.len() < 2 {
        bail!("invalid domain: expected a DNS-style host like example.com");
    }

    for (index, label) in labels.iter().enumerate() {
        if *label == "*" && index == 0 {
            continue;
        }
        if label.is_empty() || label.len() > 63 {
            bail!("invalid domain: labels must be 1-63 characters");
        }
        if label.starts_with('-') || label.ends_with('-') {
            bail!("invalid domain: labels cannot start or end with hyphens");
        }
        if !label
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        {
            bail!("invalid domain: labels may only contain ASCII letters, digits, and hyphens");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        build_go_linux_amd64_binary, caddy_init_commands, caddy_routing_commands,
        ensure_artifact_parent_dir, go_build_command_preview, resolve_domain, resolve_target,
        resolve_target_for_db, resolve_target_for_telemetry, systemd_deploy_commands,
        validate_domain, validate_systemd_unit_name, PttoConfig,
    };

    #[test]
    fn caddy_init_contains_install_and_service_steps() {
        let commands = caddy_init_commands();
        assert_eq!(commands.len(), 2);
        assert!(commands[0].contains("apt-get install -y caddy goaccess"));
        assert!(commands[0].contains("command -v goaccess"));
        assert!(commands[0].contains("goaccess install requires apt-get"));
        assert!(commands[1].contains("systemctl enable --now caddy"));
        assert!(commands[0].contains("sudo -n true"));
        assert!(commands[1].contains("sudo -n true"));
        assert!(commands[0]
            .contains("curl -1sLf https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt -o"));
    }

    #[test]
    fn systemd_deploy_contains_install_and_reload_steps() {
        let commands = systemd_deploy_commands(8080);
        assert_eq!(commands.len(), 2);
        assert!(commands[0].contains("install -m 755 /tmp/ptto-app /opt/ptto/bin/ptto-app"));
        assert!(commands[1].contains("cat > \"$tmp_service\" <<'EOF'"));
        assert!(commands[1].contains("ExecStart=/opt/ptto/bin/ptto-app"));
        assert!(commands[1].contains("systemctl daemon-reload"));
        assert!(commands[1].contains("systemctl enable --now ptto-app"));
        assert!(commands[1].contains("systemctl restart ptto-app"));
        assert!(commands[1].contains("Environment=PORT=8080"));
        assert!(commands[0].contains("sudo -n true"));
        assert!(commands[1].contains("sudo -n true"));
    }

    #[test]
    fn caddy_routing_contains_reverse_proxy_and_reload_steps() {
        let commands = caddy_routing_commands("example.com", 8080);
        assert_eq!(commands.len(), 1);
        assert!(commands[0].contains("printf '%s'"));
        assert!(commands[0].contains("reverse_proxy 127.0.0.1:8080"));
        assert!(commands[0].contains("output file /var/log/caddy/ptto-access.log"));
        assert!(commands[0].contains("caddy validate --config \"$tmp_caddy\""));
        assert!(commands[0].contains("cp /etc/caddy/Caddyfile"));
        assert!(commands[0].contains("systemctl reload caddy"));
        assert!(commands[0].contains("sudo -n true"));
    }

    #[test]
    fn domain_validation_rejects_newline_characters() {
        let err = validate_domain("example.com\nrm -rf /").expect_err("domain should be rejected");
        assert!(err
            .to_string()
            .contains("whitespace/control characters are not allowed"));
    }

    #[test]
    fn domain_validation_accepts_wildcard_only_in_leftmost_label() {
        validate_domain("*.example.com").expect("leftmost wildcard should be valid");

        let err = validate_domain("api.*.example.com")
            .expect_err("wildcard outside leftmost label should be rejected");
        assert!(err
            .to_string()
            .contains("labels may only contain ASCII letters, digits, and hyphens"));
    }

    #[test]
    fn domain_validation_rejects_non_ascii_domains() {
        let err = validate_domain("tést.example.com").expect_err("unicode should be rejected");
        assert!(err
            .to_string()
            .contains("only ASCII DNS-style domains are currently supported"));
    }

    #[test]
    fn domain_validation_enforces_length_constraints() {
        let empty = validate_domain("").expect_err("empty should be rejected");
        assert!(empty.to_string().contains("must be 1-253 characters"));

        let too_long_domain = format!("{}.com", "a".repeat(250));
        let too_long = validate_domain(&too_long_domain).expect_err("too long should be rejected");
        assert!(too_long.to_string().contains("must be 1-253 characters"));

        let label_too_long = format!("{}.com", "a".repeat(64));
        let label_error =
            validate_domain(&label_too_long).expect_err("label longer than 63 should fail");
        assert!(label_error
            .to_string()
            .contains("labels must be 1-63 characters"));
    }

    #[test]
    fn domain_validation_rejects_labels_with_edge_hyphens() {
        let leading = validate_domain("-api.example.com")
            .expect_err("leading hyphen in label should be rejected");
        assert!(leading
            .to_string()
            .contains("labels cannot start or end with hyphens"));

        let trailing = validate_domain("api-.example.com")
            .expect_err("trailing hyphen in label should be rejected");
        assert!(trailing
            .to_string()
            .contains("labels cannot start or end with hyphens"));
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
            "GOOS=linux GOARCH=amd64 go build -o './dist/my app' './cmd/my server'"
        );
    }

    #[test]
    fn systemd_service_name_validation_rejects_shell_metacharacters() {
        let err = validate_systemd_unit_name("ptto-app$(touch /tmp/pwn)")
            .expect_err("service name should be rejected");
        assert!(err.to_string().contains("invalid service name"));
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

    #[test]
    fn resolve_helpers_prefer_cli_values_over_config() {
        let config = PttoConfig {
            host: Some("root@from-config".to_string()),
            domain: Some("from-config.example.com".to_string()),
            ssh_key: None,
        };

        let target = resolve_target(Some("root@from-cli".to_string()), &config)
            .expect("target should resolve from cli");
        let db_target = resolve_target_for_db(Some("root@db-cli".to_string()), &config)
            .expect("db target should resolve from cli");
        let telemetry_target =
            resolve_target_for_telemetry(Some("root@telemetry-cli".to_string()), &config)
                .expect("telemetry target should resolve from cli");
        let domain =
            resolve_domain(Some("from-cli.example.com".to_string()), &config).expect("domain");

        assert_eq!(target, "root@from-cli");
        assert_eq!(db_target, "root@db-cli");
        assert_eq!(telemetry_target, "root@telemetry-cli");
        assert_eq!(domain, "from-cli.example.com");
    }

    #[test]
    fn resolve_helpers_fall_back_to_config_values() {
        let config = PttoConfig {
            host: Some("root@config-host".to_string()),
            domain: Some("config.example.com".to_string()),
            ssh_key: None,
        };

        let target = resolve_target(None, &config).expect("target should come from config");
        let db_target =
            resolve_target_for_db(None, &config).expect("db target should come from config");
        let telemetry_target = resolve_target_for_telemetry(None, &config)
            .expect("telemetry target should come from config");
        let domain = resolve_domain(None, &config).expect("domain should come from config");

        assert_eq!(target, "root@config-host");
        assert_eq!(db_target, "root@config-host");
        assert_eq!(telemetry_target, "root@config-host");
        assert_eq!(domain, "config.example.com");
    }

    #[test]
    fn resolve_helpers_return_actionable_errors_when_missing() {
        let config = PttoConfig::default();

        let target_error = resolve_target(None, &config).expect_err("target should be required");
        let db_error =
            resolve_target_for_db(None, &config).expect_err("db target should be required");
        let telemetry_error = resolve_target_for_telemetry(None, &config)
            .expect_err("telemetry target should be required");
        let domain_error = resolve_domain(None, &config).expect_err("domain should be required");

        let target_error_text = target_error.to_string();
        let domain_error_text = domain_error.to_string();

        assert!(target_error_text.contains("missing SSH target"));
        assert!(target_error_text.contains("--target"));
        assert!(target_error_text.contains(".ptto.toml"));
        assert!(db_error.to_string().contains("pass --target to ptto db"));
        assert!(telemetry_error.to_string().contains("pass --target"));
        assert!(domain_error_text.contains("missing domain"));
        assert!(domain_error_text.contains("--domain"));
        assert!(domain_error_text.contains(".ptto.toml"));
    }
}
