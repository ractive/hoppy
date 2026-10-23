use super::support;

use regex::Regex;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn mock_pull_zone_list() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_list_paginated.json"),
            "application/json",
        ))
        .mount(&server)
        .await;
    server
}

#[tokio::test]
async fn hints_emitted_by_default_for_table() {
    let server = mock_pull_zone_list().await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["pull-zone", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("tip: hoppy pull-zone get 1002"),
        "expected drill-down hint in stderr, got:\n{stderr}"
    );
}

#[tokio::test]
async fn hints_suppressed_by_no_hints_flag() {
    let server = mock_pull_zone_list().await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--no-hints", "pull-zone", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("tip:"),
        "--no-hints must suppress all hint output, got:\n{stderr}"
    );
}

#[tokio::test]
async fn quiet_keeps_table_but_drops_hint_on_data_command() {
    let server = mock_pull_zone_list().await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--quiet", "pull-zone", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Primary payload (the table) still prints — `pull-zone list` is a
    // data command, not a predicate command.
    assert!(
        !stdout.is_empty(),
        "expected the pull-zone list table on stdout under --quiet, got empty stdout"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("tip:"),
        "--quiet must suppress hints, got:\n{stderr}"
    );
}

#[tokio::test]
async fn hints_print_on_stderr_in_json_format() {
    // iter-89: hints are no longer suppressed by --format json — they live on
    // stderr, so JSON stdout stays parseable while agents piping through jq
    // still get next-step guidance.
    let server = mock_pull_zone_list().await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "pull-zone", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("tip:"),
        "hints must print on stderr under --format json; got stderr:\n{stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("tip:"), "hint leaked to stdout:\n{stdout}");
    serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .expect("stdout must remain pure JSON with hints enabled");
}

#[tokio::test]
async fn no_hints_still_suppresses_hints_in_json_format() {
    let server = mock_pull_zone_list().await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "--no-hints", "pull-zone", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("tip:"), "got stderr:\n{stderr}");
}

/// Mount `fixture` at `GET <route>`, run `hoppy --format json <args>` and
/// assert stdout parses as JSON while the tip lands on stderr only.
async fn assert_json_stdout_pure_with_stderr_tip(
    route: &str,
    fixture: &str,
    args: &[&str],
    tip_fragment: &str,
) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(route))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(support::fixture(fixture), "application/json"),
        )
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json"])
        .args(args)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("tip:"), "hint leaked to stdout:\n{stdout}");
    serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("stdout must be pure JSON");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("tip:") && stderr.contains(tip_fragment),
        "expected tip containing {tip_fragment:?} on stderr, got:\n{stderr}"
    );
}

#[tokio::test]
async fn country_list_json_tip_on_stderr_only() {
    assert_json_stdout_pure_with_stderr_tip(
        "/country",
        "core/country_list.json",
        &["country", "list"],
        "--blocked-countries",
    )
    .await;
}

#[tokio::test]
async fn billing_payment_requests_json_tip_on_stderr_only() {
    assert_json_stdout_pure_with_stderr_tip(
        "/billing/payment-requests",
        "core/billing_payment_requests.json",
        &["billing", "payment-requests"],
        "payment-request-pdf",
    )
    .await;
}

#[tokio::test]
async fn version_dash_v_matches_pattern() {
    let output = support::hoppy_cmd().arg("-V").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let trimmed = stdout.trim();

    // Bare `hoppy <semver>` (build script could not find git, e.g.
    // CARGO_HOPPY_FORCE_NO_GIT=1), SHA-only (crates.io tarball build via
    // `.cargo_vcs_info.json`, which carries no date), or full SHA + date.
    let full = Regex::new(r"^hoppy \d+\.\d+\.\d+ \([0-9a-f]{12}(?:\+dirty)? \d{4}-\d{2}-\d{2}\)$")
        .unwrap();
    let sha_only = Regex::new(r"^hoppy \d+\.\d+\.\d+ \([0-9a-f]{12}(?:\+dirty)?\)$").unwrap();
    let bare = Regex::new(r"^hoppy \d+\.\d+\.\d+$").unwrap();

    assert!(
        full.is_match(trimmed) || sha_only.is_match(trimmed) || bare.is_match(trimmed),
        "version output {trimmed:?} matched none of the full, sha-only or bare patterns"
    );
}
