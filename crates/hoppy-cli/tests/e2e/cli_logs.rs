use super::support;

use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn hoppy_logging_cmd(api_key: &str, logging_url: &str) -> assert_cmd::Command {
    let mut cmd = support::hoppy_cmd();
    cmd.env("BUNNY_API_KEY", api_key);
    cmd.env("BUNNY_LOGGING_URL", logging_url);
    cmd
}

fn hoppy_origin_errors_cmd(api_key: &str, origin_errors_url: &str) -> assert_cmd::Command {
    let mut cmd = support::hoppy_cmd();
    cmd.env("BUNNY_API_KEY", api_key);
    cmd.env("BUNNY_ORIGIN_ERRORS_URL", origin_errors_url);
    cmd
}

#[tokio::test]
async fn logs_pull_zone_v2_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v2/pullzones/12345/logs"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("logging/logs_query.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_logging_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "logs", "pull-zone", "--id", "12345"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"statusCode\": 502"), "stdout: {stdout}");
    assert!(
        stdout.contains("\"cacheStatus\": \"HIT\""),
        "stdout: {stdout}"
    );
}

#[tokio::test]
async fn logs_pull_zone_v2_forwards_filters() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v2/pullzones/12345/logs"))
        .and(query_param("status", "5xx"))
        .and(query_param("country", "EE"))
        .and(query_param("limit", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("logging/logs_query.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_logging_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "logs",
            "pull-zone",
            "--id",
            "12345",
            "--status",
            "5xx",
            "--country",
            "EE",
            "--limit",
            "50",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn logs_pull_zone_v2_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v2/pullzones/12345/logs"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("logging/logs_query.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_logging_cmd("test-api-key", &server.uri())
        .args(["logs", "pull-zone", "--id", "12345"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Status"), "stdout: {stdout}");
    assert!(stdout.contains("502"), "stdout: {stdout}");
}

#[tokio::test]
async fn logs_pull_zone_legacy_streams_to_file() {
    let server = MockServer::start().await;
    let raw = "1720440000|200|HIT|/a.js\n1720440001|502|MISS|/api\n";
    Mock::given(method("GET"))
        .and(path("/07-08-26/12345.log"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(raw, "text/plain"))
        .expect(1)
        .mount(&server)
        .await;

    let tmp = tempfile::tempdir().unwrap();
    let out_path = tmp.path().join("legacy.log");

    let output = hoppy_logging_cmd("test-api-key", &server.uri())
        .args([
            "logs",
            "pull-zone",
            "--id",
            "12345",
            "--legacy",
            "--date",
            "07-08-26",
            "--output",
        ])
        .arg(&out_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let written = std::fs::read_to_string(&out_path).unwrap();
    assert_eq!(written, raw);
}

#[tokio::test]
async fn logs_pull_zone_legacy_requires_date() {
    // --legacy without --date must fail (clap requires it) — no HTTP call made.
    let output = hoppy_logging_cmd("test-api-key", "http://127.0.0.1:1")
        .args(["logs", "pull-zone", "--id", "12345", "--legacy"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--date"), "stderr: {stderr}");
}

#[tokio::test]
async fn logs_origin_errors_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/12345/10-29-2025"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("origin-errors/origin_errors.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_origin_errors_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "logs",
            "origin-errors",
            "--id",
            "12345",
            "--date",
            "10-29-2025",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dns_lookup"), "stdout: {stdout}");
    assert!(stdout.contains("connect_timeout"), "stdout: {stdout}");
}

#[tokio::test]
async fn logs_origin_errors_rejects_bad_date() {
    // Local validation rejects a non MM-DD-YYYY date before any HTTP.
    let output = hoppy_origin_errors_cmd("test-api-key", "http://127.0.0.1:1")
        .args([
            "logs",
            "origin-errors",
            "--id",
            "12345",
            "--date",
            "2025-10-29",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("MM-DD-YYYY"), "stderr: {stderr}");
}

#[tokio::test]
async fn logs_origin_errors_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/12345/10-29-2025"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("origin-errors/origin_errors.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_origin_errors_cmd("test-api-key", &server.uri())
        .args([
            "logs",
            "origin-errors",
            "--id",
            "12345",
            "--date",
            "10-29-2025",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Error Code"), "stdout: {stdout}");
    assert!(stdout.contains("dns_lookup"), "stdout: {stdout}");
}
