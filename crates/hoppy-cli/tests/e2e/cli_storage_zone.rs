use super::support;

use wiremock::matchers::{header, method, path};
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
    Mock::given(method("POST"))
        .and(path("/storagezone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
            support::fixture("core/storagezone_create.json"),
            "application/json",
        ))
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
