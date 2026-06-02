//! Asserts that every `--flag` in the entire Cli command tree has non-empty
//! help text. This catches regressions where a new arg is added without a
//! `///` doc comment (the convention used throughout cli.rs).
//!
//! The walk skips clap-internal flags (`help`, `version`) whose help text is
//! auto-generated and never empty, so false positives are not a concern —
//! but they would pass anyway.

use clap::CommandFactory;
use hoppy_cli::cli::Cli;

use super::support;

fn walk_command(cmd: &clap::Command, path: &str, failures: &mut Vec<String>) {
    for arg in cmd.get_arguments() {
        // Skip args with no long name (positionals and short-only flags).
        let Some(long) = arg.get_long() else { continue };
        if long == "help" || long == "version" {
            continue;
        }
        let help_str = arg.get_help().map(|h| h.to_string()).unwrap_or_default();
        if help_str.trim().is_empty() {
            failures.push(format!("{}:--{}", path, long));
        }
    }
    for sub in cmd.get_subcommands() {
        let sub_path = format!("{} {}", path, sub.get_name());
        walk_command(sub, &sub_path, failures);
    }
}

#[test]
fn all_args_have_help_text() {
    let cmd = Cli::command();
    let mut failures: Vec<String> = Vec::new();
    walk_command(&cmd, "hoppy", &mut failures);
    if !failures.is_empty() {
        let list = failures.join("\n  ");
        panic!(
            "The following flags have no help text:\n  {list}\n\n\
             Fix: add a `/// doc comment` above the field in cli.rs."
        );
    }
}

/// Snapshot check: `hoppy pull-zone create --help` must show the `--name`
/// description introduced in iter-64.
#[test]
fn pull_zone_create_help_shows_name_description() {
    let output = support::hoppy_cmd()
        .args(["pull-zone", "create", "--help"])
        .output()
        .expect("failed to run hoppy");

    assert!(
        output.status.success(),
        "`hoppy pull-zone create --help` failed: status={:?}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Pull Zone name"),
        "`hoppy pull-zone create --help` must contain 'Pull Zone name', got:\n{stdout}"
    );
}
