mod e2e_support;

use assert_cmd::Command;
use predicates::prelude::*;

fn hoppy_no_env() -> Command {
    let mut cmd = Command::cargo_bin("hoppy").expect("hoppy binary not found");
    cmd.env_remove("BUNNY_API_KEY");
    cmd
}

#[test]
fn help_flag_shows_usage() {
    hoppy_no_env()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("bunny.net"))
        .stdout(predicate::str::contains("pull-zone"))
        .stdout(predicate::str::contains("storage-zone"))
        .stdout(predicate::str::contains("dns"))
        .stdout(predicate::str::contains("stream"))
        .stdout(predicate::str::contains("shield"))
        .stdout(predicate::str::contains("script"))
        .stdout(predicate::str::contains("container"));
}

#[test]
fn version_flag_shows_version() {
    hoppy_no_env()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn missing_api_key_shows_error() {
    hoppy_no_env()
        .args(["pull-zone", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("BUNNY_API_KEY"));
}

#[test]
fn unknown_command_shows_error() {
    hoppy_no_env()
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}
