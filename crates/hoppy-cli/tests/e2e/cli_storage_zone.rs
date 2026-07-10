use super::support;

use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn storage_zone_list_json() {
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

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "storage-zone", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn storage_zone_list_forwards_include_deleted() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/storagezone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("includeDeleted", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/storagezone_list_paginated.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "storage-zone",
            "list",
            "--include-deleted",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn storage_zone_get_redacts_passwords_by_default() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/storagezone/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/storagezone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "storage-zone", "get", "--id", "9001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("redacted-storage-password"),
        "raw password leaked into default output"
    );
    assert!(
        stdout.contains("\"Password\": \"<set, length=25>\""),
        "expected Password to be replaced with the <set, length=N> placeholder, got: {stdout}"
    );
    assert!(
        stdout.contains("\"ReadOnlyPassword\": \"<set, length=26>\""),
        "expected ReadOnlyPassword to be replaced with the <set, length=N> placeholder, got: {stdout}"
    );
}

#[tokio::test]
async fn storage_zone_get_reveal_returns_passwords() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/storagezone/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/storagezone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--reveal",
            "--format",
            "json",
            "storage-zone",
            "get",
            "--id",
            "9001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("redacted-storage-password"),
        "--reveal should bypass redaction"
    );
}

#[tokio::test]
async fn storage_zone_list_table() {
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

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "storage-zone", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn storage_zone_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/storagezone/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/storagezone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "storage-zone", "get", "--id", "9001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn storage_zone_get_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/storagezone/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/storagezone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "storage-zone", "get", "--id", "9001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn storage_zone_create_json() {
    let server = MockServer::start().await;
    // The real API returns the literal string "string" as a placeholder for
    // Password / ReadOnlyPassword on the POST response. Mock that here so the
    // test fails if the CLI surfaces the POST credentials instead of the GET.
    let post_body = r#"{
        "Id": 9099,
        "UserId": "00000000-0000-0000-0000-000000000001",
        "Name": "hoppy-test-zone",
        "Password": "string",
        "DateModified": "2026-03-18T02:12:58.4797282Z",
        "Deleted": false,
        "StorageUsed": 0,
        "FilesStored": 0,
        "Region": "DE",
        "ReplicationRegions": [],
        "PullZones": null,
        "ReadOnlyPassword": "string",
        "Rewrite404To200": false,
        "Custom404FilePath": null,
        "StorageHostname": "storage.bunnycdn.com",
        "ZoneTier": 0,
        "ReplicationChangeInProgress": false,
        "PriceOverride": 0.0,
        "Discount": 0,
        "StorageZoneType": 0
    }"#;
    Mock::given(method("POST"))
        .and(path("/storagezone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(201).set_body_raw(post_body.as_bytes(), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    // After create, the CLI immediately fetches the zone (GET /storagezone/{id})
    // to obtain the real Password (the create response has a placeholder).
    // The create fixture has Id=9099, so the GET will be for /storagezone/9099.
    Mock::given(method("GET"))
        .and(path("/storagezone/9099"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/storagezone_create.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "storage-zone",
            "create",
            "--name",
            "hoppy-test-zone",
            "--region",
            "DE",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn storage_zone_update() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/storagezone/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/storagezone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "storage-zone",
            "update",
            "--id",
            "9001",
            "--rewrite-404-to-200",
            "true",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn storage_zone_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/storagezone/9001"))
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
            "storage-zone",
            "delete",
            "--id",
            "9001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn storage_zone_delete_defaults_to_removing_linked_pull_zones() {
    // Without --keep-linked-pull-zones, hoppy must send the explicit
    // deleteLinkedPullZones=true so the destructive default is on the record.
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/storagezone/9001"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("deleteLinkedPullZones", "true"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--yes", "storage-zone", "delete", "--id", "9001"])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn storage_zone_delete_keep_linked_pull_zones_opts_out() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/storagezone/9001"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("deleteLinkedPullZones", "false"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "storage-zone",
            "delete",
            "--id",
            "9001",
            "--keep-linked-pull-zones",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("linked pull zones kept"),
        "expected keep-linked note in prose, got: {stderr}"
    );
}

#[tokio::test]
async fn storage_zone_reset_password_refetches_and_redacts() {
    let server = MockServer::start().await;
    // The reset POST returns 204 with no body.
    Mock::given(method("POST"))
        .and(path("/storagezone/9001/resetPassword"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    // hoppy then re-fetches the zone to surface the new credential.
    Mock::given(method("GET"))
        .and(path("/storagezone/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/storagezone_get.json"),
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
            "storage-zone",
            "reset-password",
            "--id",
            "9001",
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
        !stdout.contains("redacted-storage-password"),
        "raw password leaked into default output: {stdout}"
    );
    assert!(
        stdout.contains("<set, length="),
        "expected redacted placeholder, got: {stdout}"
    );
}

#[tokio::test]
async fn storage_zone_reset_read_only_password_uses_query_id_and_reveals() {
    let server = MockServer::start().await;
    // Read-only reset takes id as a QUERY param, not a path segment.
    Mock::given(method("POST"))
        .and(path("/storagezone/resetReadOnlyPassword"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("id", "9001"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/storagezone/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/storagezone_get.json"),
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
            "storage-zone",
            "reset-read-only-password",
            "--id",
            "9001",
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
        stdout.contains("redacted-readonly-password"),
        "--reveal should surface the raw read-only password, got: {stdout}"
    );
}

#[tokio::test]
async fn storage_zone_get_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/storagezone/999999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(404).set_body_raw(
            support::fixture("core/error_not_found_storagezone.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "storage-zone", "get", "--id", "999999"])
        .output()
        .unwrap();

    assert!(!output.status.success());
}

#[tokio::test]
async fn storage_zone_statistics_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/storagezone/42/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/storagezone_statistics.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "storage-zone",
            "statistics",
            "--id",
            "42",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
    assert!(json["StorageUsedChart"].is_object());
}

#[cfg(feature = "live-api")]
#[test]
fn live_storage_zone_lifecycle() {
    support::run_lifecycle(|cleanup| {
        let raw_name = support::unique_name("hpst");
        let name: String = raw_name.chars().take(20).collect();

        // 1. Create
        let create = support::hoppy_live_json(&[
            "storage-zone",
            "create",
            "--name",
            &name,
            "--region",
            "DE",
        ]);
        assert!(create.success, "create failed: {}", create.stderr);
        let id = create.json.as_ref().unwrap()["Id"]
            .as_u64()
            .expect("Id missing from create response");
        let id_str = id.to_string();

        // Register cleanup early
        cleanup.push(&["storage-zone", "delete", "--id", &id_str]);

        // 2. Get by id
        let get = support::hoppy_live_json(&["storage-zone", "get", "--id", &id_str]);
        assert!(get.success, "get failed: {}", get.stderr);

        // 3. List — verify zone appears
        let list = support::hoppy_live_json(&["storage-zone", "list"]);
        assert!(list.success, "list failed: {}", list.stderr);
        let items = list.json.as_ref().unwrap()["Items"]
            .as_array()
            .expect("Items missing from list response");
        let found = items.iter().any(|z| z["Id"].as_u64() == Some(id));
        assert!(found, "created zone {id} not found in list");

        // 4. Update
        let update = support::hoppy_live_json(&[
            "storage-zone",
            "update",
            "--id",
            &id_str,
            "--rewrite-404-to-200",
            "true",
        ]);
        assert!(update.success, "update failed: {}", update.stderr);

        // 5. Get and verify Rewrite404To200
        let get2 = support::hoppy_live_json(&["storage-zone", "get", "--id", &id_str]);
        assert!(get2.success, "second get failed: {}", get2.stderr);
        let rewrite = get2.json.as_ref().unwrap()["Rewrite404To200"]
            .as_bool()
            .unwrap_or(false);
        assert!(rewrite, "Rewrite404To200 should be true after update");

        // 6. Get statistics
        let stats = support::hoppy_live_json(&["storage-zone", "statistics", "--id", &id_str]);
        assert!(stats.success, "statistics failed: {}", stats.stderr);

        // 7. Delete is handled by cleanup
    });
}

// ---------------------------------------------------------------------------
// iter-51: --format json on mutations emits success envelope on stdout
// ---------------------------------------------------------------------------

#[tokio::test]
async fn storage_zone_delete_format_json_emits_envelope() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/storagezone/9001"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "--format",
            "json",
            "storage-zone",
            "delete",
            "--id",
            "9001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout is not JSON: {e}\nstdout: {stdout}"));
    assert_eq!(v["status"], "ok");
    assert_eq!(v["action"], "delete");
    assert_eq!(v["resource"], "storage-zone");
    assert_eq!(v["Id"], 9001);
}

#[tokio::test]
async fn storage_zone_delete_default_format_keeps_prose() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/storagezone/9001"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--yes", "storage-zone", "delete", "--id", "9001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    // Default format = table; prose goes to stderr, stdout stays clean.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Deleted storage zone 9001"),
        "stderr: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// iter-71 — storage-zone check / regions / egress
// ---------------------------------------------------------------------------

#[tokio::test]
async fn storage_zone_check_json() {
    use wiremock::matchers::body_string_contains;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/storagezone/checkavailability"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_string_contains("\"Name\":\"my-assets\""))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/zone_availability.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "storage-zone",
            "check",
            "--name",
            "my-assets",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid json");
    assert_eq!(json["Available"], true);
}

#[tokio::test]
async fn storage_zone_regions_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/storagezone/regions"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/storagezone_regions.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "storage-zone", "regions"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid json");
    let regions = json.as_array().expect("array of regions");
    assert_eq!(regions.len(), 3);
    assert_eq!(regions[0]["Id"], "DE");
}

#[tokio::test]
async fn storage_zone_egress_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/storagezone/9001/statistics/egress"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/storagezone_egress_statistics.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "storage-zone", "egress", "--id", "9001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid json");
    assert_eq!(json["TotalEgress"], 3600);
    assert_eq!(json["HttpEgressTotal"], 3000);
}

#[tokio::test]
async fn storage_zone_egress_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/storagezone/9001/statistics/egress"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/storagezone_egress_statistics.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "table",
            "storage-zone",
            "egress",
            "--id",
            "9001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Protocol"), "expected Protocol column");
    assert!(stdout.contains("Total"), "expected Total row");
}
