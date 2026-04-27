use assert_cmd::Command;
use predicates::str::contains;

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
