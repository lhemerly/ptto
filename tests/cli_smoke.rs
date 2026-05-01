use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;

#[test]
fn help_shows_manifesto_language() {
    let mut cmd = Command::cargo_bin("ptto").expect("binary should build");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("Deploy single-binary web apps"));
}

#[test]
fn init_command_accepts_target() {
    let mut cmd = Command::cargo_bin("ptto").expect("binary should build");
    cmd.args(["init", "root@127.0.0.1", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("bootstrap starting for root@127.0.0.1"));
}

#[test]
fn deploy_command_supports_ssh_transfer_dry_run() {
    let mut cmd = Command::cargo_bin("ptto").expect("binary should build");
    cmd.args([
        "deploy",
        "--domain",
        "example.com",
        "--target",
        "root@127.0.0.1",
        "--dry-run",
    ])
    .assert()
    .success()
    .stdout(contains("artifact staged over ssh"));
}

#[test]
fn deploy_uses_ptto_toml_defaults() {
    let dir = tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join(".ptto.toml"),
        r#"host = "root@127.0.0.1"
domain = "example.com"
ssh_key = "~/.ssh/custom_key"
"#,
    )
    .expect("config should write");

    let mut cmd = Command::cargo_bin("ptto").expect("binary should build");
    cmd.current_dir(dir.path())
        .args(["deploy", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("deploy pipeline planned for domain example.com"))
        .stdout(contains(
            "scp -o BatchMode=yes -o StrictHostKeyChecking=accept-new -i ~/.ssh/custom_key",
        ));
}

#[test]
fn logs_does_not_require_or_parse_ptto_toml() {
    let dir = tempdir().expect("tempdir");
    std::fs::write(dir.path().join(".ptto.toml"), "this is not valid toml")
        .expect("config should write");

    let mut cmd = Command::cargo_bin("ptto").expect("binary should build");
    cmd.current_dir(dir.path())
        .args(["logs"])
        .assert()
        .success()
        .stdout(contains("log streaming planned for service ptto-app"));
}

#[test]
fn generate_key_does_not_require_or_parse_ptto_toml() {
    let dir = tempdir().expect("tempdir");
    std::fs::write(dir.path().join(".ptto.toml"), "not valid = { toml")
        .expect("config should write");

    let mut cmd = Command::cargo_bin("ptto").expect("binary should build");
    cmd.current_dir(dir.path())
        .args(["generate-key"])
        .assert()
        .success()
        .stdout(contains("key generation hook planned for CI/CD"));
}

#[test]
fn db_help_lists_management_commands() {
    let mut cmd = Command::cargo_bin("ptto").expect("binary should build");
    cmd.args(["db", "--help"])
        .assert()
        .success()
        .stdout(contains("shell"))
        .stdout(contains("pull"))
        .stdout(contains("push"));
}

#[test]
fn deploy_rejects_invalid_domain_input() {
    let mut cmd = Command::cargo_bin("ptto").expect("binary should build");
    cmd.args([
        "deploy",
        "--domain",
        "example.com\nbad",
        "--target",
        "root@127.0.0.1",
        "--dry-run",
    ])
    .assert()
    .failure()
    .stderr(contains("invalid domain"))
    .stderr(contains("whitespace/control characters"));
}

#[test]
fn db_pull_requires_target_when_not_in_config() {
    let dir = tempdir().expect("tempdir");

    let mut cmd = Command::cargo_bin("ptto").expect("binary should build");
    cmd.current_dir(dir.path())
        .args(["db", "pull"])
        .assert()
        .failure()
        .stderr(contains("missing SSH target"))
        .stderr(contains("pass --target to ptto db"));
}
