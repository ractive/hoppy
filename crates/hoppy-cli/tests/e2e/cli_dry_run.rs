//! e2e coverage for the global `--dry-run` flag (iter-82).
//!
//! Mutating (POST/PUT/PATCH/DELETE) requests are blocked at the client
//! layer and never reach the mock server; read-only (GET/HEAD) requests
//! still execute normally. See `hoppy-knowledgebase/iterations/
//! iteration-82-dry-run.md` for the full design.

use super::support;

use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn container_mock_cmd(api_key: &str, server_uri: &str) -> assert_cmd::Command {
    support::hoppy_mock_cmd_full(api_key, server_uri, None, None, Some(server_uri))
}

fn db_mock_cmd(api_key: &str, db_url: &str) -> assert_cmd::Command {
    let mut cmd = support::hoppy_cmd();
    cmd.env("BUNNY_API_KEY", api_key);
    cmd.env("BUNNY_DATABASE_URL", db_url);
    cmd
}

/// Count mock-server requests matching a method + path, for "nothing was
/// sent" assertions (mirrors the pattern used in cli_pull_zone.rs).
async fn count_requests(server: &MockServer, method: &str, path: &str) -> usize {
    server
        .received_requests()
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|r| r.method.as_str().eq_ignore_ascii_case(method) && r.url.path() == path)
        .count()
}

// ---------------------------------------------------------------------------
// Preview output: table/text format
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pull_zone_create_dry_run_blocks_and_previews() {
    let server = MockServer::start().await;
    // No mock mounted for POST /pullzone — any request that reaches the
    // server would 404, but the assertion below on received_requests() is
    // what actually proves nothing was sent.

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--dry-run",
            "pull-zone",
            "create",
            "--name",
            "test-zone",
            "--origin-url",
            "https://example.com",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "expected exit 0, got {:?}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(count_requests(&server, "POST", "/pullzone").await, 0);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[dry-run] Would send: POST"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("/pullzone"), "stderr: {stderr}");
    assert!(stderr.contains("test-zone"), "stderr: {stderr}");
}

/// Deleting without `--yes` and without piping stdin must not hang waiting
/// on a confirmation prompt — `--dry-run` implies `--yes`.
#[tokio::test]
async fn pull_zone_delete_dry_run_skips_prompt_without_stdin() {
    let server = MockServer::start().await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--dry-run", "pull-zone", "delete", "--id", "1001"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "expected exit 0, got {:?}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(count_requests(&server, "DELETE", "/pullzone/1001").await, 0);
}

// ---------------------------------------------------------------------------
// Preview output: JSON envelope
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pull_zone_create_dry_run_json_envelope() {
    let server = MockServer::start().await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--dry-run",
            "--format",
            "json",
            "pull-zone",
            "create",
            "--name",
            "test-zone",
            "--origin-url",
            "https://example.com",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(count_requests(&server, "POST", "/pullzone").await, 0);

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout must be valid JSON");
    assert_eq!(json["status"], "dry-run");
    assert_eq!(json["method"], "POST");
    assert!(
        json["url"]
            .as_str()
            .unwrap_or_default()
            .ends_with("/pullzone"),
        "url: {}",
        json["url"]
    );
    assert_eq!(json["body"]["Name"], "test-zone");
}

// ---------------------------------------------------------------------------
// Composite command: preflight GET still runs, mutation blocked
// ---------------------------------------------------------------------------

/// `storage upload --dry-run` must still resolve the zone password via the
/// Core API `GET /storagezone` preflight (composite commands stay truthful
/// under dry-run) while the actual `PUT` upload is blocked.
#[tokio::test]
async fn storage_upload_dry_run_resolves_zone_but_blocks_put() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/storagezone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/storagezone_list_paginated.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let temp_path = std::env::temp_dir().join("hoppy-test-dry-run-upload.txt");
    std::fs::write(&temp_path, b"hello world content").unwrap();

    // Deliberately use `hoppy_mock_cmd` (not `_full`) plus a manual
    // BUNNY_STORAGE_URL so BUNNY_STORAGE_KEY stays unset — this forces the
    // Core API fallback path in `build_storage_client` instead of reading
    // the access key straight from the environment.
    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .env("BUNNY_STORAGE_URL", server.uri())
        .args([
            "--dry-run",
            "storage",
            "upload",
            "--zone",
            "test-storage-zone-1",
            "--remote-path",
            "/test-dir/hello.txt",
            "--file",
            temp_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let _ = std::fs::remove_file(&temp_path);

    assert!(
        output.status.success(),
        "expected exit 0, got {:?}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(count_requests(&server, "GET", "/storagezone").await, 1);
    assert_eq!(
        count_requests(&server, "PUT", "/test-dir/hello.txt").await,
        0
    );
}

// ---------------------------------------------------------------------------
// One delete-path test per service (mechanical clones of the pull-zone case)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dns_record_delete_dry_run_zero_requests() {
    let server = MockServer::start().await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--dry-run",
            "dns",
            "record",
            "delete",
            "--zone-id",
            "50001",
            "--record-id",
            "100001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        count_requests(&server, "DELETE", "/dnszone/50001/records/100001").await,
        0
    );
}

#[tokio::test]
async fn shield_waf_delete_rule_dry_run_zero_requests() {
    let server = MockServer::start().await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--dry-run", "shield", "waf", "delete-rule", "--id", "7001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        count_requests(&server, "DELETE", "/shield/waf/custom-rule/7001").await,
        0
    );
}

#[tokio::test]
async fn container_app_delete_dry_run_zero_requests() {
    let server = MockServer::start().await;

    let output = container_mock_cmd("test-api-key", &server.uri())
        .args([
            "--dry-run",
            "container",
            "app",
            "delete",
            "--id",
            "test-app-id",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "expected exit 0, got {:?}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        count_requests(&server, "DELETE", "/apps/test-app-id").await,
        0
    );
}

#[tokio::test]
async fn db_delete_dry_run_zero_requests() {
    let server = MockServer::start().await;

    let output = db_mock_cmd("test-api-key", &server.uri())
        .args(["--dry-run", "db", "delete", "--id", "db-001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        count_requests(&server, "DELETE", "/v1/databases/db-001").await,
        0
    );
}

// ---------------------------------------------------------------------------
// Redaction
// ---------------------------------------------------------------------------

/// Secrets in the previewed body are redacted by default and shown raw only
/// under `--reveal` — mirrors the existing `--debug` body-redaction contract.
#[tokio::test]
async fn dry_run_reveal_shows_unredacted_body() {
    let server = MockServer::start().await;

    let redacted_output = container_mock_cmd("test-api-key", &server.uri())
        .args([
            "--dry-run",
            "container",
            "registry",
            "create",
            "--name",
            "hoppy-test-registry",
            "--username",
            "hoppy-user",
            "--password",
            "supersecretvalue",
        ])
        .output()
        .unwrap();
    assert!(redacted_output.status.success());
    let redacted_stderr = String::from_utf8_lossy(&redacted_output.stderr);
    assert!(
        !redacted_stderr.contains("supersecretvalue"),
        "password leaked without --reveal: {redacted_stderr}"
    );

    let revealed_output = container_mock_cmd("test-api-key", &server.uri())
        .args([
            "--dry-run",
            "--reveal",
            "container",
            "registry",
            "create",
            "--name",
            "hoppy-test-registry",
            "--username",
            "hoppy-user",
            "--password",
            "supersecretvalue",
        ])
        .output()
        .unwrap();
    assert!(revealed_output.status.success());
    let revealed_stderr = String::from_utf8_lossy(&revealed_output.stderr);
    assert!(
        revealed_stderr.contains("supersecretvalue"),
        "expected raw password with --reveal: {revealed_stderr}"
    );

    assert_eq!(count_requests(&server, "POST", "/registries").await, 0);
}

// ---------------------------------------------------------------------------
// Read-only commands are unaffected
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pull_zone_list_dry_run_still_executes() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_list_page1.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--dry-run", "--format", "json", "pull-zone", "list"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "expected exit 0, got {:?}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(count_requests(&server, "GET", "/pullzone").await, 1);

    // Normal list output, not a dry-run preview envelope.
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout must be valid JSON");
    assert!(json["Items"].is_array(), "expected normal list output");
}
