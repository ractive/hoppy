//! E2E coverage for the `fixture-refresh` drift-radar binary.
//!
//! These tests spawn the actual `fixture-refresh` binary (via `assert_cmd`)
//! against a synthetic `--recorded` tree pointed at the real, checked-in
//! `fixtures/` directory (via `--fixtures`), and assert:
//! - `--shape-report` output contains the documented sections
//! - exit codes follow the documented contract (0 clean, 1 drift, 2 leaks)
//! - `fixtures/` is never modified — its bytes and mtime are identical
//!   before and after every invocation, including the removed `--apply`
//!   flag path (which no longer exists at all)
//! - `--apply` is not a recognized flag anymore

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use assert_cmd::Command;
use tempfile::tempdir;

fn fixture_refresh_cmd() -> Command {
    Command::cargo_bin("fixture-refresh").expect("fixture-refresh binary not found")
}

/// Absolute path to the workspace's real `fixtures/` directory.
fn real_fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures")
}

/// Snapshot of (relative path -> (bytes, mtime)) for every file under `dir`,
/// used to assert a directory tree was not touched by a command invocation.
fn snapshot_tree(dir: &Path) -> Vec<(PathBuf, u64, SystemTime)> {
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry.expect("walk fixtures dir");
        if !entry.file_type().is_file() {
            continue;
        }
        let meta = entry.metadata().expect("read metadata");
        out.push((
            entry.path().to_path_buf(),
            meta.len(),
            meta.modified().expect("mtime"),
        ));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Build a synthetic recorded tree with a single `core/GET_billing.json`
/// recording. `body` is the raw JSON bytes to record.
fn write_recording(recorded_dir: &Path, domain: &str, filename: &str, body: &str) {
    let domain_dir = recorded_dir.join(domain);
    fs::create_dir_all(&domain_dir).expect("create domain dir");
    fs::write(domain_dir.join(filename), body).expect("write recording");
}

#[test]
fn apply_flag_no_longer_exists() {
    let dir = tempdir().unwrap();
    let recorded_dir = dir.path().join("recorded");
    fs::create_dir_all(&recorded_dir).unwrap();

    fixture_refresh_cmd()
        .args(["--recorded", recorded_dir.to_str().unwrap()])
        .arg("--apply")
        .assert()
        .failure();
}

#[test]
fn default_mode_never_writes_to_fixtures() {
    let dir = tempdir().unwrap();
    let recorded_dir = dir.path().join("recorded");
    // Recording body differs from the real fixture -> would have been "drift".
    write_recording(
        &recorded_dir,
        "core",
        "GET_billing.json",
        r#"{"Balance": 999}"#,
    );

    let fixtures_dir = real_fixtures_dir();
    let before = snapshot_tree(&fixtures_dir);

    fixture_refresh_cmd()
        .args(["--recorded", recorded_dir.to_str().unwrap()])
        .args(["--fixtures", fixtures_dir.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("drift:"));

    let after = snapshot_tree(&fixtures_dir);
    assert_eq!(before, after, "fixtures/ must be byte- and mtime-identical");
}

#[test]
fn shape_report_never_writes_to_fixtures() {
    let dir = tempdir().unwrap();
    let recorded_dir = dir.path().join("recorded");
    write_recording(
        &recorded_dir,
        "core",
        "GET_billing.json",
        r#"{"Balance": 0, "BrandNewField": "x"}"#,
    );

    let fixtures_dir = real_fixtures_dir();
    let before = snapshot_tree(&fixtures_dir);

    fixture_refresh_cmd()
        .args(["--recorded", recorded_dir.to_str().unwrap()])
        .args(["--fixtures", fixtures_dir.to_str().unwrap()])
        .arg("--shape-report")
        .assert()
        .code(1); // drift found: BrandNewField added, plus real fixture's other fields removed

    let after = snapshot_tree(&fixtures_dir);
    assert_eq!(before, after, "fixtures/ must be byte- and mtime-identical");
}

#[test]
fn shape_report_contains_documented_sections() {
    let dir = tempdir().unwrap();
    let recorded_dir = dir.path().join("recorded");
    write_recording(
        &recorded_dir,
        "core",
        "GET_billing.json",
        r#"{"Balance": 0}"#,
    );

    let fixtures_dir = real_fixtures_dir();

    let output = fixture_refresh_cmd()
        .args(["--recorded", recorded_dir.to_str().unwrap()])
        .args(["--fixtures", fixtures_dir.to_str().unwrap()])
        .arg("--shape-report")
        .output()
        .expect("run fixture-refresh");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("## Leak audit"), "stdout: {stdout}");
    assert!(stdout.contains("## Shape drift"), "stdout: {stdout}");
    assert!(
        stdout.contains("## Unmapped recordings"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("## Collisions"), "stdout: {stdout}");
}

#[test]
fn shape_report_exit_code_zero_when_recording_matches_fixture_exactly() {
    let dir = tempdir().unwrap();
    let recorded_dir = dir.path().join("recorded");
    let fixtures_dir = real_fixtures_dir();

    // Use the exact bytes of the real fixture as the recording so the diff
    // (and leak audit, since the fixture already uses <redacted>) is empty.
    let real_billing = fs::read_to_string(fixtures_dir.join("core/billing_get.json")).unwrap();
    write_recording(&recorded_dir, "core", "GET_billing.json", &real_billing);

    fixture_refresh_cmd()
        .args(["--recorded", recorded_dir.to_str().unwrap()])
        .args(["--fixtures", fixtures_dir.to_str().unwrap()])
        .arg("--shape-report")
        .assert()
        .code(0);
}

#[test]
fn shape_report_exit_code_two_when_recording_has_leak() {
    let dir = tempdir().unwrap();
    let recorded_dir = dir.path().join("recorded");
    let fixtures_dir = real_fixtures_dir();

    // Plant an unredacted email in the recording — must flip exit code to 2
    // even though this recording also has plenty of shape drift.
    write_recording(
        &recorded_dir,
        "core",
        "GET_billing.json",
        r#"{"Balance": 0, "AuthorEmail": "leaked-person@real-company.com"}"#,
    );

    let output = fixture_refresh_cmd()
        .args(["--recorded", recorded_dir.to_str().unwrap()])
        .args(["--fixtures", fixtures_dir.to_str().unwrap()])
        .arg("--shape-report")
        .output()
        .expect("run fixture-refresh");

    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("leaked-person@real-company.com") || stdout.contains("AuthorEmail"),
        "leak should be listed in the report: {stdout}"
    );
}

#[test]
fn shape_report_exit_code_two_when_recording_has_double_uuid() {
    let dir = tempdir().unwrap();
    let recorded_dir = dir.path().join("recorded");
    let fixtures_dir = real_fixtures_dir();

    let double_uuid = "eda66cfe-8fd7-4040-997f-77a6c66fe488ea41a773-201d-4cbf-81df-1735d605b486";
    write_recording(
        &recorded_dir,
        "core",
        "GET_billing.json",
        &format!(r#"{{"Balance": 0, "SomeHarmlessField": "{double_uuid}"}}"#),
    );

    fixture_refresh_cmd()
        .args(["--recorded", recorded_dir.to_str().unwrap()])
        .args(["--fixtures", fixtures_dir.to_str().unwrap()])
        .arg("--shape-report")
        .assert()
        .code(2);
}

#[test]
fn shape_report_out_flag_writes_file_instead_of_stdout() {
    let dir = tempdir().unwrap();
    let recorded_dir = dir.path().join("recorded");
    let fixtures_dir = real_fixtures_dir();
    let real_billing = fs::read_to_string(fixtures_dir.join("core/billing_get.json")).unwrap();
    write_recording(&recorded_dir, "core", "GET_billing.json", &real_billing);

    let out_path = dir.path().join("report.md");

    let output = fixture_refresh_cmd()
        .args(["--recorded", recorded_dir.to_str().unwrap()])
        .args(["--fixtures", fixtures_dir.to_str().unwrap()])
        .arg("--shape-report")
        .args(["--out", out_path.to_str().unwrap()])
        .output()
        .expect("run fixture-refresh");

    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout).is_empty(),
        "report should not also print to stdout when --out is set"
    );
    let written = fs::read_to_string(&out_path).expect("report file written");
    assert!(written.contains("## Leak audit"));
}

#[test]
fn shape_report_unmapped_recording_flips_exit_code() {
    let dir = tempdir().unwrap();
    let recorded_dir = dir.path().join("recorded");
    let fixtures_dir = real_fixtures_dir();

    // No test source maps this path to any descriptive fixture.
    write_recording(
        &recorded_dir,
        "core",
        "GET_totally_unknown_endpoint.json",
        r#"{"x": 1}"#,
    );

    let output = fixture_refresh_cmd()
        .args(["--recorded", recorded_dir.to_str().unwrap()])
        .args(["--fixtures", fixtures_dir.to_str().unwrap()])
        .arg("--shape-report")
        .output()
        .expect("run fixture-refresh");

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("core/GET_totally_unknown_endpoint.json"),
        "stdout: {stdout}"
    );
}
