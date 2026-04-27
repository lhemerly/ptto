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
    cmd.args(["init", "root@127.0.0.1"])
        .assert()
        .success()
        .stdout(contains("bootstrap planned for root@127.0.0.1"));
}
