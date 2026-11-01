//! `hoppy` without arguments prints the short help on stdout and exits 0,
//! exactly as `hoppy -h` does. winget's validator runs the installed binary
//! with no arguments and fails the package on any non-zero exit.

use super::support;

fn run(args: &[&str]) -> std::process::Output {
    support::hoppy_cmd()
        .args(args)
        .output()
        .expect("failed to run hoppy")
}

#[test]
fn bare_hoppy_prints_short_help_and_exits_zero() {
    let bare = run(&[]);
    let stdout = String::from_utf8_lossy(&bare.stdout);
    assert_eq!(
        bare.status.code(),
        Some(0),
        "bare `hoppy` must exit 0\nstdout: {stdout}\nstderr: {}",
        String::from_utf8_lossy(&bare.stderr),
    );
    assert!(
        stdout.contains("Usage: hoppy"),
        "stdout lacks the usage line:\n{stdout}"
    );
    assert!(bare.stderr.is_empty(), "stderr must stay empty");

    let short_help = run(&["-h"]);
    assert_eq!(
        stdout,
        String::from_utf8_lossy(&short_help.stdout),
        "bare `hoppy` must print exactly what `hoppy -h` prints"
    );
}

#[test]
fn invalid_invocations_still_exit_non_zero() {
    for args in [
        &["no-such-command"][..],
        &["--no-such-flag"],
        &["--debug"],
        &["dns"],
    ] {
        let out = run(args);
        assert!(
            !out.status.success(),
            "`hoppy {}` must exit non-zero",
            args.join(" ")
        );
    }
}
