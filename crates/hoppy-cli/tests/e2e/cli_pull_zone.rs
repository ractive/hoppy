use super::support;

use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// --all flag: auto-pagination tests
// ---------------------------------------------------------------------------

/// `pull-zone list --all --format json` fetches 3 pages and merges them into one
/// JSON envelope with `hasMoreItems: false`.
#[tokio::test]
async fn pull_zone_list_all_json_three_pages() {
    let server = MockServer::start().await;

    // Page 1 — hasMoreItems: true
    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_list_page1.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    // Page 2 — hasMoreItems: true
    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "2"))
        .and(query_param("perPage", "1000"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_list_page2.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    // Page 3 — hasMoreItems: false
    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "3"))
        .and(query_param("perPage", "1000"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_list_page3.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "pull-zone", "list", "--all"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");

    // Three items collected across 3 pages (PaginatedListJson uses PascalCase)
    let items = json["Items"].as_array().expect("Items must be array");
    assert_eq!(items.len(), 3, "expected 3 items from 3 pages");
    assert_eq!(items[0]["Id"], 1001);
    assert_eq!(items[1]["Id"], 1002);
    assert_eq!(items[2]["Id"], 1003);
    assert_eq!(json["HasMoreItems"], false, "all pages consumed — no more");
}

/// `pull-zone list --all --format table` fetches multiple pages and appends rows.
#[tokio::test]
async fn pull_zone_list_all_table_three_pages() {
    let server = MockServer::start().await;

    for (page, fixture) in [
        ("1", "core/pullzone_list_page1.json"),
        ("2", "core/pullzone_list_page2.json"),
        ("3", "core/pullzone_list_page3.json"),
    ] {
        Mock::given(method("GET"))
            .and(path("/pullzone"))
            .and(header("AccessKey", "test-api-key"))
            .and(query_param("page", page))
            .and(query_param("perPage", "1000"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(support::fixture(fixture), "application/json"),
            )
            .expect(1)
            .mount(&server)
            .await;
    }

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "pull-zone", "list", "--all"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Expect rows for all 3 zones
    assert!(stdout.contains("pz-page-one"), "missing page 1 row");
    assert!(stdout.contains("pz-page-two"), "missing page 2 row");
    assert!(stdout.contains("pz-page-three"), "missing page 3 row");
}

/// `pull-zone list --all --format text` works the same as table.
#[tokio::test]
async fn pull_zone_list_all_text_three_pages() {
    let server = MockServer::start().await;

    for (page, fixture) in [
        ("1", "core/pullzone_list_page1.json"),
        ("2", "core/pullzone_list_page2.json"),
        ("3", "core/pullzone_list_page3.json"),
    ] {
        Mock::given(method("GET"))
            .and(path("/pullzone"))
            .and(header("AccessKey", "test-api-key"))
            .and(query_param("page", page))
            .and(query_param("perPage", "1000"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(support::fixture(fixture), "application/json"),
            )
            .expect(1)
            .mount(&server)
            .await;
    }

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "text", "pull-zone", "list", "--all"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pz-page-one"));
    assert!(stdout.contains("pz-page-two"));
    assert!(stdout.contains("pz-page-three"));
}

/// `--all` conflicts with `--page` (clap error, exit code 2).
#[tokio::test]
async fn pull_zone_list_all_conflicts_with_page() {
    let output = support::hoppy_cmd()
        .env("BUNNY_API_KEY", "test-api-key")
        .args(["pull-zone", "list", "--all", "--page", "2"])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(2),
        "expected clap error exit code 2"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cannot be used with"),
        "expected conflict error, got: {stderr}"
    );
}

/// `--all` conflicts with `--per-page` (clap error, exit code 2).
#[tokio::test]
async fn pull_zone_list_all_conflicts_with_per_page() {
    let output = support::hoppy_cmd()
        .env("BUNNY_API_KEY", "test-api-key")
        .args(["pull-zone", "list", "--all", "--per-page", "50"])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(2),
        "expected clap error exit code 2"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cannot be used with"),
        "expected conflict error, got: {stderr}"
    );
}

/// Regression: `--page` and `--per-page` still work without `--all`.
#[tokio::test]
async fn pull_zone_list_explicit_page_regression() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "2"))
        .and(query_param("perPage", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_list_paginated.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "pull-zone",
            "list",
            "--page",
            "2",
            "--per-page",
            "10",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(json["Items"].is_array(), "expected Items array");
}

#[tokio::test]
async fn pull_zone_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_list_paginated.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "pull-zone", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn pull_zone_list_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_list_paginated.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "pull-zone", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn pull_zone_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "pull-zone", "get", "--id", "1001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(json["Id"].is_number(), "expected Id to be a number");
    assert!(json["Name"].is_string(), "expected Name to be a string");
    assert!(
        json["OriginUrl"].is_string(),
        "expected OriginUrl to be a string"
    );
    assert!(
        json["Enabled"].is_boolean(),
        "expected Enabled to be a boolean"
    );
    assert!(
        json["Hostnames"].is_array(),
        "expected Hostnames to be an array"
    );
}

#[tokio::test]
async fn pull_zone_get_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "pull-zone", "get", "--id", "1001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Vertical layout: Field/Value columns with zone properties as rows
    assert!(stdout.contains("Field"), "expected Field column");
    assert!(stdout.contains("Value"), "expected Value column");
    assert!(stdout.contains("Id"), "expected Id row");
    assert!(stdout.contains("Name"), "expected Name row");
    assert!(stdout.contains("OriginUrl"), "expected OriginUrl row");
    assert!(stdout.contains("Enabled"), "expected Enabled row");
}

#[tokio::test]
async fn pull_zone_create_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
            support::fixture("core/pullzone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
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
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(json["Id"].is_number(), "expected Id to be a number");
    assert!(json["Name"].is_string(), "expected Name to be a string");
    assert!(
        json["OriginUrl"].is_string(),
        "expected OriginUrl to be a string"
    );
    assert!(
        json["Enabled"].is_boolean(),
        "expected Enabled to be a boolean"
    );
    assert!(
        json["Hostnames"].is_array(),
        "expected Hostnames to be an array"
    );
}

#[tokio::test]
async fn pull_zone_create_with_storage_zone_id() {
    let server = MockServer::start().await;
    let expected_body = serde_json::json!({
        "Name": "static-files-pz",
        "StorageZoneId": 1234,
        "Type": 0,
    });
    Mock::given(method("POST"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
            support::fixture("core/pullzone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "pull-zone",
            "create",
            "--name",
            "static-files-pz",
            "--storage-zone-id",
            "1234",
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
async fn pull_zone_create_with_volume_tier() {
    let server = MockServer::start().await;
    let expected_body = serde_json::json!({
        "Name": "high-traffic",
        "OriginUrl": "https://example.com",
        "Type": 1,
    });
    Mock::given(method("POST"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
            support::fixture("core/pullzone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "pull-zone",
            "create",
            "--name",
            "high-traffic",
            "--origin-url",
            "https://example.com",
            "--zone-tier",
            "volume",
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
async fn pull_zone_create_requires_origin_or_storage_zone() {
    // No mock — clap should reject before any HTTP call.
    let output = support::hoppy_mock_cmd("test-api-key", "http://127.0.0.1:1")
        .args(["pull-zone", "create", "--name", "missing-origin"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("required")
            && stderr.contains("origin-url")
            && stderr.contains("storage-zone-id"),
        "expected clap to demand --origin-url or --storage-zone-id, got: {stderr}"
    );
}

#[tokio::test]
async fn pull_zone_create_rejects_both_origin_and_storage_zone() {
    let output = support::hoppy_mock_cmd("test-api-key", "http://127.0.0.1:1")
        .args([
            "pull-zone",
            "create",
            "--name",
            "ambiguous",
            "--origin-url",
            "https://example.com",
            "--storage-zone-id",
            "1234",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cannot be used with") || stderr.contains("conflicts with"),
        "expected clap mutual-exclusion error, got: {stderr}"
    );
}

#[tokio::test]
async fn pull_zone_update_storage_zone_id() {
    let server = MockServer::start().await;
    let expected_body = serde_json::json!({
        "StorageZoneId": 9876,
    });
    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "pull-zone",
            "update",
            "--id",
            "1001",
            "--storage-zone-id",
            "9876",
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
async fn pull_zone_update() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "pull-zone",
            "update",
            "--id",
            "1001",
            "--origin-url",
            "https://new.example.com",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn pull_zone_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "--format",
            "json",
            "pull-zone",
            "delete",
            "--id",
            "1001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn pull_zone_reset_security_key_refetches_and_hides_key() {
    let server = MockServer::start().await;
    // The reset POST returns 204 with no body.
    Mock::given(method("POST"))
        .and(path("/pullzone/5857627/resetSecurityKey"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    // hoppy re-fetches the zone afterwards.
    Mock::given(method("GET"))
        .and(path("/pullzone/5857627"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "--format",
            "json",
            "pull-zone",
            "reset-security-key",
            "--id",
            "5857627",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // ZoneSecurityKey is skip_serializing — the raw key must never appear
    // without --reveal.
    assert!(
        !stdout.contains("REDACTED-SECURITY-KEY"),
        "raw security key leaked without --reveal: {stdout}"
    );
}

#[tokio::test]
async fn pull_zone_reset_security_key_reveals_new_key() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/5857627/resetSecurityKey"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/pullzone/5857627"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--reveal",
            "--yes",
            "--format",
            "json",
            "pull-zone",
            "reset-security-key",
            "--id",
            "5857627",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("REDACTED-SECURITY-KEY"),
        "--reveal should surface the new ZoneSecurityKey, got: {stdout}"
    );
}

#[tokio::test]
async fn pull_zone_purge() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/purgeCache"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "pull-zone", "purge", "--id", "1001"])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn pull_zone_get_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/999999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(404).set_body_raw(
            support::fixture("core/error_not_found_storagezone.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "pull-zone", "get", "--id", "999999"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not_found") || stderr.contains("not found") || stderr.contains("404"));
}

#[tokio::test]
async fn pull_zone_update_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/999999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(404).set_body_raw(
            support::fixture("core/error_not_found_storagezone.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "pull-zone",
            "update",
            "--id",
            "999999",
            "--origin-url",
            "https://example.com",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not_found") || stderr.contains("not found") || stderr.contains("404"));
}

// ---------------------------------------------------------------------------
// URL purge E2E test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn purge_url_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/purge"))
        .and(header("AccessKey", "test-key"))
        .and(query_param("url", "https://cdn.example.com/index.html"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args(["purge", "--url", "https://cdn.example.com/index.html"]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Purged"));
}

#[tokio::test]
async fn purge_url_forwards_exact_path_and_async() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/purge"))
        .and(header("AccessKey", "test-key"))
        .and(query_param("url", "https://cdn.example.com/index.html"))
        .and(query_param("exactPath", "true"))
        .and(query_param("async", "true"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "purge",
        "--url",
        "https://cdn.example.com/index.html",
        "--exact-path",
        "--async",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Purged"));
}

// ---------------------------------------------------------------------------
// Pull Zone hostname & SSL E2E tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pull_zone_hostname_add() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/addHostname"))
        .and(header("AccessKey", "test-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "cdn.example.com" }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "hostname",
        "add",
        "--id",
        "1001",
        "--hostname",
        "cdn.example.com",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Added hostname"));
}

#[tokio::test]
async fn pull_zone_hostname_remove() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/pullzone/1001/removeHostname"))
        .and(header("AccessKey", "test-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "cdn.example.com" }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "hostname",
        "remove",
        "--id",
        "1001",
        "--hostname",
        "cdn.example.com",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Removed hostname"));
}

#[tokio::test]
async fn pull_zone_hostname_load_free_cert() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/loadFreeCertificate"))
        .and(header("AccessKey", "test-key"))
        .and(query_param("hostname", "cdn.example.com"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "hostname",
        "load-free-cert",
        "--hostname",
        "cdn.example.com",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Loaded free certificate"));
}

#[tokio::test]
async fn pull_zone_hostname_force_ssl() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/setForceSSL"))
        .and(header("AccessKey", "test-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "cdn.example.com", "ForceSSL": true }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "hostname",
        "force-ssl",
        "--id",
        "1001",
        "--hostname",
        "cdn.example.com",
        "--enabled=true",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Force SSL enabled"));
}

#[tokio::test]
async fn pull_zone_hostname_remove_cert() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/pullzone/1001/removeCertificate"))
        .and(header("AccessKey", "test-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "cdn.example.com" }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "hostname",
        "remove-cert",
        "--id",
        "1001",
        "--hostname",
        "cdn.example.com",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Removed certificate"));
}

// ---------------------------------------------------------------------------
// Pull Zone access-control (referrer / IP) E2E tests
// ---------------------------------------------------------------------------

fn pullzone_with_access_control_json() -> String {
    serde_json::json!({
        "Id": 1001,
        "Name": "test-zone",
        "OriginUrl": "https://example.com",
        "AllowedReferrers": ["allowed.example.com", "*.partner.com"],
        "BlockedReferrers": ["badsite.com"],
        "BlockedIps": ["192.0.2.1", "203.0.113.0/24"]
    })
    .to_string()
}

#[tokio::test]
async fn pull_zone_referrer_list_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(pullzone_with_access_control_json(), "application/json"),
        )
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-key", &server.uri())
        .args([
            "--format",
            "table",
            "pull-zone",
            "referrer",
            "list",
            "--id",
            "1001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn pull_zone_referrer_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(pullzone_with_access_control_json(), "application/json"),
        )
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-key", &server.uri())
        .args([
            "--format",
            "json",
            "pull-zone",
            "referrer",
            "list",
            "--id",
            "1001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn pull_zone_referrer_allow() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/addAllowedReferrer"))
        .and(header("AccessKey", "test-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "*.example.com" }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "referrer",
        "allow",
        "--id",
        "1001",
        "--value",
        "*.example.com",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Allowed referrer"));
}

#[tokio::test]
async fn pull_zone_referrer_remove_allowed() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/removeAllowedReferrer"))
        .and(header("AccessKey", "test-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "*.example.com" }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "referrer",
        "remove-allowed",
        "--id",
        "1001",
        "--value",
        "*.example.com",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Removed allowed referrer"));
}

#[tokio::test]
async fn pull_zone_referrer_block() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/addBlockedReferrer"))
        .and(header("AccessKey", "test-key"))
        .and(body_json(serde_json::json!({ "Hostname": "badsite.com" })))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "referrer",
        "block",
        "--id",
        "1001",
        "--value",
        "badsite.com",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Blocked referrer"));
}

#[tokio::test]
async fn pull_zone_referrer_remove_blocked() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/removeBlockedReferrer"))
        .and(header("AccessKey", "test-key"))
        .and(body_json(serde_json::json!({ "Hostname": "badsite.com" })))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "referrer",
        "remove-blocked",
        "--id",
        "1001",
        "--value",
        "badsite.com",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Removed blocked referrer"));
}

#[tokio::test]
async fn pull_zone_ip_list_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(pullzone_with_access_control_json(), "application/json"),
        )
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-key", &server.uri())
        .args([
            "--format",
            "table",
            "pull-zone",
            "ip",
            "list",
            "--id",
            "1001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn pull_zone_ip_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(pullzone_with_access_control_json(), "application/json"),
        )
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-key", &server.uri())
        .args([
            "--format",
            "json",
            "pull-zone",
            "ip",
            "list",
            "--id",
            "1001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn pull_zone_ip_block() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/addBlockedIp"))
        .and(header("AccessKey", "test-key"))
        .and(body_json(serde_json::json!({ "BlockedIp": "192.0.2.1" })))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "ip",
        "block",
        "--id",
        "1001",
        "--value",
        "192.0.2.1",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Blocked IP"));
}

#[tokio::test]
async fn pull_zone_ip_unblock() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/removeBlockedIp"))
        .and(header("AccessKey", "test-key"))
        .and(body_json(serde_json::json!({ "BlockedIp": "192.0.2.1" })))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "ip",
        "unblock",
        "--id",
        "1001",
        "--value",
        "192.0.2.1",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Unblocked IP"));
}

#[cfg(feature = "live-api")]
#[test]
fn live_pull_zone_lifecycle() {
    support::run_lifecycle(|cleanup| {
        let name = support::unique_name("hoppy-test");

        // 1. Create
        let create = support::hoppy_live_json(&[
            "pull-zone",
            "create",
            "--name",
            &name,
            "--origin-url",
            "https://example.com",
        ]);
        assert!(create.success, "create failed — stderr: {}", create.stderr);
        let id = create.json.as_ref().unwrap()["Id"].as_i64().unwrap();
        let id_str = id.to_string();

        // Register cleanup early so it runs even on panic
        cleanup.push(&["pull-zone", "delete", "--id", &id_str]);

        // 2. Get by id
        let get = support::hoppy_live_json(&["pull-zone", "get", "--id", &id_str]);
        assert!(get.success, "get failed — stderr: {}", get.stderr);

        // 3. List and verify zone appears
        let list = support::hoppy_live_json(&["pull-zone", "list"]);
        assert!(list.success, "list failed — stderr: {}", list.stderr);
        let zones = list.json.as_ref().unwrap()["Items"].as_array().unwrap();
        let found = zones.iter().any(|z| z["Id"].as_i64() == Some(id));
        assert!(found, "zone {id} not found in list output");

        // 4. Update origin URL
        let update = support::hoppy_live_json(&[
            "pull-zone",
            "update",
            "--id",
            &id_str,
            "--origin-url",
            "https://new.example.com",
        ]);
        assert!(update.success, "update failed — stderr: {}", update.stderr);

        // 5. Get again and verify OriginUrl changed
        let get2 = support::hoppy_live_json(&["pull-zone", "get", "--id", &id_str]);
        assert!(get2.success, "second get failed — stderr: {}", get2.stderr);
        let origin = get2.json.as_ref().unwrap()["OriginUrl"]
            .as_str()
            .unwrap_or("");
        assert!(
            origin.contains("new.example.com"),
            "expected updated OriginUrl, got: {origin}"
        );

        // 6. Purge
        let purge = support::hoppy_live_json(&["pull-zone", "purge", "--id", &id_str]);
        assert!(purge.success, "purge failed — stderr: {}", purge.stderr);

        // 7. Optimizer statistics
        let opt_stats = support::hoppy_live_json(&[
            "pull-zone",
            "statistics",
            "--id",
            &id_str,
            "--type",
            "optimizer",
        ]);
        assert!(
            opt_stats.success,
            "optimizer statistics failed — stderr: {}",
            opt_stats.stderr
        );

        // 8. Origin shield statistics
        let os_stats = support::hoppy_live_json(&[
            "pull-zone",
            "statistics",
            "--id",
            &id_str,
            "--type",
            "origin-shield",
        ]);
        assert!(
            os_stats.success,
            "origin-shield statistics failed — stderr: {}",
            os_stats.stderr
        );

        // 9. SafeHop statistics
        let sh_stats = support::hoppy_live_json(&[
            "pull-zone",
            "statistics",
            "--id",
            &id_str,
            "--type",
            "safehop",
        ]);
        assert!(
            sh_stats.success,
            "safehop statistics failed — stderr: {}",
            sh_stats.stderr
        );

        // 10. Cleanup runs via CleanupStack on exit
    });
}

#[tokio::test]
async fn pull_zone_statistics_optimizer_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/1001/optimizer/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_optimizer_statistics.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "pull-zone",
            "statistics",
            "--id",
            "1001",
            "--type",
            "optimizer",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let _json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
}

#[tokio::test]
async fn pull_zone_statistics_origin_shield_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/1001/originshield/queuestatistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_originshield_statistics.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "pull-zone",
            "statistics",
            "--id",
            "1001",
            "--type",
            "origin-shield",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let _json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
}

#[tokio::test]
async fn pull_zone_statistics_safehop_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/1001/safehop/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_safehop_statistics.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "pull-zone",
            "statistics",
            "--id",
            "1001",
            "--type",
            "safehop",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let _json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
}

#[cfg(feature = "live-api")]
#[test]
fn live_pull_zone_edge_rule_lifecycle() {
    support::run_lifecycle(|cleanup| {
        let name = support::unique_name("hoppy-edge-rule");

        // 1. Create pull zone
        let create = support::hoppy_live_json(&[
            "pull-zone",
            "create",
            "--name",
            &name,
            "--origin-url",
            "https://example.com",
        ]);
        assert!(create.success, "create failed — stderr: {}", create.stderr);
        let id = create.json.as_ref().unwrap()["Id"].as_i64().unwrap();
        let id_str = id.to_string();
        cleanup.push(&["pull-zone", "delete", "--id", &id_str]);

        // 2. Add a redirect edge rule
        let add = support::hoppy_live_raw(&[
            "pull-zone",
            "edge-rule",
            "add",
            "--id",
            &id_str,
            "--description",
            "E2E redirect rule",
            "--action-type",
            "redirect",
            "--action-param1",
            "https://example.com/new-path",
            "--action-param2",
            "301",
            "--trigger",
            "url:*/old-path*",
        ]);
        assert!(add.success, "add failed — stderr: {}", add.stderr);

        // 3. List edge rules and find the newly added rule's Guid
        let list = support::hoppy_live_json(&["pull-zone", "edge-rule", "list", "--id", &id_str]);
        assert!(list.success, "list failed — stderr: {}", list.stderr);
        let rules = list.json.as_ref().unwrap().as_array().unwrap();
        let rule = rules
            .iter()
            .find(|r| r["Description"].as_str() == Some("E2E redirect rule"))
            .expect("added edge rule not found in list");
        let guid = rule["Guid"].as_str().unwrap().to_string();

        // 4. Disable the rule
        let disable = support::hoppy_live_raw(&[
            "pull-zone",
            "edge-rule",
            "enable",
            "--id",
            &id_str,
            "--rule-id",
            &guid,
            "--enabled",
            "false",
        ]);
        assert!(
            disable.success,
            "disable failed — stderr: {}",
            disable.stderr
        );

        // 5. Re-enable the rule
        let enable = support::hoppy_live_raw(&[
            "pull-zone",
            "edge-rule",
            "enable",
            "--id",
            &id_str,
            "--rule-id",
            &guid,
            "--enabled",
            "true",
        ]);
        assert!(enable.success, "enable failed — stderr: {}", enable.stderr);

        // 6. Delete the rule
        let delete = support::hoppy_live_json_yes(&[
            "pull-zone",
            "edge-rule",
            "delete",
            "--id",
            &id_str,
            "--rule-id",
            &guid,
        ]);
        assert!(delete.success, "delete failed — stderr: {}", delete.stderr);

        // 7. Verify the rule is gone
        let after = support::hoppy_live_json(&["pull-zone", "edge-rule", "list", "--id", &id_str]);
        assert!(
            after.success,
            "post-delete list failed — stderr: {}",
            after.stderr
        );
        let remaining = after.json.as_ref().unwrap().as_array().unwrap();
        assert!(
            !remaining.iter().any(|r| r["Guid"].as_str() == Some(&guid)),
            "edge rule {guid} still present after delete"
        );

        // 8. Cleanup runs via CleanupStack
    });
}

#[cfg(feature = "live-api")]
#[test]
fn live_pull_zone_get_nonexistent() {
    let result = support::hoppy_live_json(&["pull-zone", "get", "--id", "999999999"]);
    assert!(
        !result.success,
        "expected failure for nonexistent pull zone"
    );
}

#[cfg(feature = "live-api")]
#[test]
fn live_pull_zone_update_nonexistent() {
    let result = support::hoppy_live_json(&[
        "pull-zone",
        "update",
        "--id",
        "999999999",
        "--origin-url",
        "https://example.com",
    ]);
    assert!(
        !result.success,
        "expected failure for nonexistent pull zone update"
    );
}

// ---------------------------------------------------------------------------
// Edge rule E2E tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pull_zone_edge_rule_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_get_with_edgerules.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "pull-zone",
            "edge-rule",
            "list",
            "--id",
            "1001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn pull_zone_edge_rule_list_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_get_with_edgerules.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "table",
            "pull-zone",
            "edge-rule",
            "list",
            "--id",
            "1001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn pull_zone_edge_rule_add() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/edgerules/addOrUpdate"))
        .and(header("AccessKey", "test-key"))
        .respond_with(ResponseTemplate::new(201))
        .expect(1)
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "edge-rule",
        "add",
        "--id",
        "1001",
        "--action-type",
        "block-request",
        "--description",
        "Block bad actors",
        "--trigger",
        "url:*/bad-path*",
    ]);
    cmd.assert().success().stderr(predicates::str::contains(
        "Added edge rule to pull zone 1001",
    ));
}

#[tokio::test]
async fn pull_zone_edge_rule_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(
            "/pullzone/1001/edgerules/a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        ))
        .and(header("AccessKey", "test-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "--yes",
        "pull-zone",
        "edge-rule",
        "delete",
        "--id",
        "1001",
        "--rule-id",
        "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    ]);
    cmd.assert().success().stderr(predicates::str::contains(
        "Deleted edge rule a1b2c3d4-e5f6-7890-abcd-ef1234567890 from pull zone 1001",
    ));
}

#[tokio::test]
async fn pull_zone_edge_rule_enable() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/pullzone/1001/edgerules/a1b2c3d4-e5f6-7890-abcd-ef1234567890/setEdgeRuleEnabled",
        ))
        .and(header("AccessKey", "test-key"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "edge-rule",
        "enable",
        "--id",
        "1001",
        "--rule-id",
        "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "--enabled",
        "true",
    ]);
    cmd.assert().success().stderr(predicates::str::contains(
        "Enabled edge rule a1b2c3d4-e5f6-7890-abcd-ef1234567890 on pull zone 1001",
    ));
}

#[cfg(feature = "live-api")]
#[test]
fn live_pull_zone_access_control_lifecycle() {
    support::run_lifecycle(|cleanup| {
        let name = support::unique_name("hoppy-test-ac");

        // 1. Create pull zone
        let create = support::hoppy_live_json(&[
            "pull-zone",
            "create",
            "--name",
            &name,
            "--origin-url",
            "https://example.com",
        ]);
        assert!(create.success, "create failed — stderr: {}", create.stderr);
        let id = create.json.as_ref().unwrap()["Id"].as_i64().unwrap();
        let id_str = id.to_string();
        cleanup.push(&["pull-zone", "delete", "--id", &id_str]);

        // 2. Add allowed referrer
        let allow = support::hoppy_live_raw(&[
            "pull-zone",
            "referrer",
            "allow",
            "--id",
            &id_str,
            "--value",
            "*.allowed.example.com",
        ]);
        assert!(allow.success, "allow failed — stderr: {}", allow.stderr);

        // 3. Add blocked referrer
        let block = support::hoppy_live_raw(&[
            "pull-zone",
            "referrer",
            "block",
            "--id",
            &id_str,
            "--value",
            "blocked.example.com",
        ]);
        assert!(block.success, "block failed — stderr: {}", block.stderr);

        // 4. Verify both lists via get
        let get1 = support::hoppy_live_json(&["pull-zone", "get", "--id", &id_str]);
        assert!(get1.success, "get failed — stderr: {}", get1.stderr);
        let allowed = get1.json.as_ref().unwrap()["AllowedReferrers"]
            .as_array()
            .map(|a| {
                a.iter()
                    .any(|v| v.as_str() == Some("*.allowed.example.com"))
            })
            .unwrap_or(false);
        let blocked = get1.json.as_ref().unwrap()["BlockedReferrers"]
            .as_array()
            .map(|a| a.iter().any(|v| v.as_str() == Some("blocked.example.com")))
            .unwrap_or(false);
        assert!(allowed, "expected allowed referrer in AllowedReferrers");
        assert!(blocked, "expected blocked referrer in BlockedReferrers");

        // 5. Remove both referrers
        let rm_allow = support::hoppy_live_raw(&[
            "pull-zone",
            "referrer",
            "remove-allowed",
            "--id",
            &id_str,
            "--value",
            "*.allowed.example.com",
        ]);
        assert!(
            rm_allow.success,
            "remove-allowed failed — stderr: {}",
            rm_allow.stderr
        );
        let rm_block = support::hoppy_live_raw(&[
            "pull-zone",
            "referrer",
            "remove-blocked",
            "--id",
            &id_str,
            "--value",
            "blocked.example.com",
        ]);
        assert!(
            rm_block.success,
            "remove-blocked failed — stderr: {}",
            rm_block.stderr
        );

        // 6. Block an IP
        let ip_block = support::hoppy_live_raw(&[
            "pull-zone",
            "ip",
            "block",
            "--id",
            &id_str,
            "--value",
            "192.0.2.1",
        ]);
        assert!(
            ip_block.success,
            "ip block failed — stderr: {}",
            ip_block.stderr
        );

        // 7. Verify IP via get
        let get2 = support::hoppy_live_json(&["pull-zone", "get", "--id", &id_str]);
        assert!(get2.success, "get failed — stderr: {}", get2.stderr);
        let has_ip = get2.json.as_ref().unwrap()["BlockedIps"]
            .as_array()
            .map(|a| a.iter().any(|v| v.as_str() == Some("192.0.2.1")))
            .unwrap_or(false);
        assert!(has_ip, "expected blocked IP in BlockedIps");

        // 8. Unblock IP
        let ip_unblock = support::hoppy_live_raw(&[
            "pull-zone",
            "ip",
            "unblock",
            "--id",
            &id_str,
            "--value",
            "192.0.2.1",
        ]);
        assert!(
            ip_unblock.success,
            "ip unblock failed — stderr: {}",
            ip_unblock.stderr
        );

        // 9. Cleanup runs via CleanupStack on exit
    });
}

// ---------------------------------------------------------------------------
// Optimizer CLI tests
// ---------------------------------------------------------------------------

/// Verifies that all Optimizer flags are wired to the correct wire keys.
/// Notably: `--optimizer-minify-js` → `OptimizerMinifyJavaScript` (long form),
/// `--optimizer-watermark-position center` → `OptimizerWatermarkPosition: 4`.
#[tokio::test]
async fn pull_zone_update_optimizer_settings() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "OptimizerEnabled": true,
        "OptimizerEnableWebP": true,
        "OptimizerMinifyCSS": true,
        "OptimizerMinifyJavaScript": true,
        "OptimizerImageQuality": 80,
        "OptimizerWatermarkPosition": 4
    });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "pull-zone",
            "update",
            "--id",
            "1001",
            "--optimizer-enabled",
            "true",
            "--optimizer-webp",
            "true",
            "--optimizer-minify-css",
            "true",
            "--optimizer-minify-js",
            "true",
            "--optimizer-image-quality",
            "80",
            "--optimizer-watermark-position",
            "center",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------------
// Log-forwarding precheck E2E tests
// ---------------------------------------------------------------------------

/// When LogForwardingEnabled=false and the user passes --log-forwarding-hostname
/// without --log-forwarding-enabled true, the CLI must:
///  1. GET /pullzone/{id} to check current state.
///  2. Exit non-zero with an error and hint on stderr.
///  3. NOT send the POST /pullzone/{id} update request.
#[tokio::test]
async fn pull_zone_update_lf_hostname_disabled_zone_errors() {
    let server = MockServer::start().await;

    // GET /pullzone/1001 — zone has LogForwardingEnabled=false
    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "pull-zone",
            "update",
            "--id",
            "1001",
            "--log-forwarding-hostname",
            "foo.example.com",
        ])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "expected non-zero exit, got success"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("log-forwarding fields cannot be updated while disabled"),
        "expected error message, got: {stderr}"
    );
    assert!(
        stderr.contains("--log-forwarding-enabled true"),
        "expected hint, got: {stderr}"
    );

    // Verify the precheck short-circuited: no POST /pullzone/1001 was sent.
    let posts = server
        .received_requests()
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|r| {
            r.method.as_str().eq_ignore_ascii_case("POST") && r.url.path() == "/pullzone/1001"
        })
        .count();
    assert_eq!(posts, 0, "expected no POST /pullzone/1001, got {posts}");
}

/// When --log-forwarding-enabled true is passed along with hostname, the precheck
/// is skipped and the update proceeds.
#[tokio::test]
async fn pull_zone_update_lf_hostname_with_enable_skips_precheck() {
    let server = MockServer::start().await;

    // No GET mock — precheck should be skipped.
    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "pull-zone",
            "update",
            "--id",
            "1001",
            "--log-forwarding-enabled",
            "true",
            "--log-forwarding-hostname",
            "foo.example.com",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn pull_zone_update_firewall_fields() {
    let server = MockServer::start().await;
    let expected_body = serde_json::json!({
        "BlockedCountries": ["CN", "RU"],
        "RequestLimit": 100,
        "ShieldDDosProtectionType": 2,
        "ConnectionLimitPerIPCount": 50,
    });
    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "pull-zone",
            "update",
            "--id",
            "1001",
            "--blocked-countries",
            "CN,RU",
            "--request-limit",
            "100",
            "--shield-ddos-protection-type",
            "active-aggressive",
            "--connection-limit-per-ip-count",
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
