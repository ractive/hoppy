mod support;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// DNS export/import E2E tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dns_zone_export_prints_zone_file() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001/export"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(support::fixture("core/dnszone_export.txt"), "text/plain"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["dns", "zone", "export", "--id", "50001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("$ORIGIN"));
    assert!(stdout.contains("example.com"));
}

#[tokio::test]
async fn dns_zone_import_from_file_table_output() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/dnszone/50001/import"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_import.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    // Write a temp zone file
    let zone_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(
        zone_file.path(),
        support::fixture("core/dnszone_export.txt"),
    )
    .unwrap();

    let mut cmd = support::hoppy_mock_cmd("test-api-key", &server.uri());
    cmd.args([
        "dns",
        "zone",
        "import",
        "--id",
        "50001",
        "--file",
        zone_file.path().to_str().unwrap(),
    ]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Import complete"));
    assert!(stderr.contains("5 successful"));
}

#[tokio::test]
async fn dns_zone_import_from_file_json_output() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/dnszone/50001/import"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_import.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let zone_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(
        zone_file.path(),
        support::fixture("core/dnszone_export.txt"),
    )
    .unwrap();

    let mut cmd = support::hoppy_mock_cmd("test-api-key", &server.uri());
    cmd.args([
        "--format",
        "json",
        "dns",
        "zone",
        "import",
        "--id",
        "50001",
        "--file",
        zone_file.path().to_str().unwrap(),
    ]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert_eq!(json["RecordsSuccessful"], 5);
    assert_eq!(json["RecordsFailed"], 0);
}

#[tokio::test]
async fn dns_zone_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_list_paginated.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "dns", "zone", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn dns_zone_list_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_list_paginated.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "dns", "zone", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn dns_zone_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "dns", "zone", "get", "--id", "50001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn dns_zone_get_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "dns", "zone", "get", "--id", "50001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn dns_zone_create_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/dnszone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
            support::fixture("core/dnszone_create.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "dns",
            "zone",
            "create",
            "--domain",
            "example.com",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn dns_zone_update() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "dns",
            "zone",
            "update",
            "--id",
            "50001",
            "--logging-enabled",
            "true",
            "--soa-email",
            "admin@test.com",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn dns_zone_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes", "--format", "json", "dns", "zone", "delete", "--id", "50001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn dns_record_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "dns",
            "record",
            "list",
            "--zone-id",
            "50001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn dns_record_add_json() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/dnszone/50001/records"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
            support::fixture("core/dnsrecord_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "dns",
            "record",
            "add",
            "--zone-id",
            "50001",
            "--type",
            "A",
            "--name",
            "test",
            "--value",
            "192.0.2.1",
            "--ttl",
            "300",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn dns_record_update() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/dnszone/50001/records/100001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnsrecord_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "dns",
            "record",
            "update",
            "--zone-id",
            "50001",
            "--record-id",
            "100001",
            "--type",
            "A",
            "--value",
            "5.6.7.8",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

// ---------------------------------------------------------------------------
// Live API tests
// ---------------------------------------------------------------------------

#[cfg(feature = "live-api")]
#[test]
fn live_dns_zone_lifecycle() {
    support::run_lifecycle(|cleanup| {
        let domain = format!("{}.test", support::unique_name("hoppytest"));

        // 1. Create zone
        let create = support::hoppy_live_json(&["dns", "zone", "create", "--domain", &domain]);
        assert!(create.success, "create zone failed: {}", create.stderr);
        let zone_id = create.json.as_ref().unwrap()["Id"].as_i64().unwrap();
        let zone_id_str = zone_id.to_string();

        // Register cleanup early so it runs even if later assertions fail
        cleanup.push(&["dns", "zone", "delete", "--id", &zone_id_str]);

        // 2. Get zone by id
        let get = support::hoppy_live_json(&["dns", "zone", "get", "--id", &zone_id_str]);
        assert!(get.success, "get zone failed: {}", get.stderr);
        assert_eq!(get.json.as_ref().unwrap()["Id"].as_i64().unwrap(), zone_id);

        // 3. List zones and verify the new zone appears
        let list = support::hoppy_live_json(&["dns", "zone", "list"]);
        assert!(list.success, "list zones failed: {}", list.stderr);
        let zones = list.json.as_ref().unwrap()["Items"].as_array().unwrap();
        assert!(
            zones.iter().any(|z| z["Id"].as_i64() == Some(zone_id)),
            "created zone {zone_id} not found in list"
        );

        // 4. Update zone: enable logging
        let update = support::hoppy_live_json(&[
            "dns",
            "zone",
            "update",
            "--id",
            &zone_id_str,
            "--logging-enabled",
            "true",
        ]);
        assert!(update.success, "update zone failed: {}", update.stderr);

        // 5. Get again and verify LoggingEnabled is true
        let get2 = support::hoppy_live_json(&["dns", "zone", "get", "--id", &zone_id_str]);
        assert!(get2.success, "second get zone failed: {}", get2.stderr);
        assert!(
            get2.json.as_ref().unwrap()["LoggingEnabled"]
                .as_bool()
                .unwrap(),
            "expected LoggingEnabled to be true after update"
        );

        // 6. Get statistics
        let stats = support::hoppy_live_json(&["dns", "zone", "statistics", "--id", &zone_id_str]);
        assert!(stats.success, "zone statistics failed: {}", stats.stderr);

        // 7. Delete is handled by cleanup stack
    });
}

#[cfg(feature = "live-api")]
#[test]
fn live_dns_record_lifecycle() {
    support::run_lifecycle(|cleanup| {
        let domain = format!("{}.test", support::unique_name("hoppytest"));

        // 1. Create zone
        let create = support::hoppy_live_json(&["dns", "zone", "create", "--domain", &domain]);
        assert!(create.success, "create zone failed: {}", create.stderr);
        let zone_id = create.json.as_ref().unwrap()["Id"].as_i64().unwrap();
        let zone_id_str = zone_id.to_string();

        cleanup.push(&["dns", "zone", "delete", "--id", &zone_id_str]);

        // 2. Add A record
        let add = support::hoppy_live_json(&[
            "dns",
            "record",
            "add",
            "--zone-id",
            &zone_id_str,
            "--type",
            "A",
            "--name",
            "test",
            "--value",
            "1.2.3.4",
        ]);
        assert!(add.success, "add record failed: {}", add.stderr);
        let record_id = add.json.as_ref().unwrap()["Id"].as_i64().unwrap();
        let record_id_str = record_id.to_string();

        // 3. List records and verify the A record appears
        let list = support::hoppy_live_json(&["dns", "record", "list", "--zone-id", &zone_id_str]);
        assert!(list.success, "list records failed: {}", list.stderr);
        let records = list.json.as_ref().unwrap().as_array().unwrap();
        assert!(
            records.iter().any(|r| r["Id"].as_i64() == Some(record_id)),
            "added record {record_id} not found in list"
        );

        // 4. Update record value
        let update = support::hoppy_live_json(&[
            "dns",
            "record",
            "update",
            "--zone-id",
            &zone_id_str,
            "--record-id",
            &record_id_str,
            "--type",
            "A",
            "--value",
            "5.6.7.8",
        ]);
        assert!(update.success, "update record failed: {}", update.stderr);

        // 5. Delete record
        let delete = support::hoppy_live_json_yes(&[
            "dns",
            "record",
            "delete",
            "--zone-id",
            &zone_id_str,
            "--record-id",
            &record_id_str,
        ]);
        assert!(delete.success, "delete record failed: {}", delete.stderr);

        // 6. Zone cleanup handled by cleanup stack
    });
}

#[cfg(feature = "live-api")]
#[test]
fn live_dns_record_mx_priority() {
    support::run_lifecycle(|cleanup| {
        let domain = format!("{}.test", support::unique_name("hoppytest"));

        // 1. Create zone
        let create = support::hoppy_live_json(&["dns", "zone", "create", "--domain", &domain]);
        assert!(create.success, "create zone failed: {}", create.stderr);
        let zone_id = create.json.as_ref().unwrap()["Id"].as_i64().unwrap();
        let zone_id_str = zone_id.to_string();

        cleanup.push(&["dns", "zone", "delete", "--id", &zone_id_str]);

        // 2. Add MX record with priority 10
        let add = support::hoppy_live_json(&[
            "dns",
            "record",
            "add",
            "--zone-id",
            &zone_id_str,
            "--type",
            "MX",
            "--value",
            "mail.example.com",
            "--priority",
            "10",
        ]);
        assert!(add.success, "add MX record failed: {}", add.stderr);
        let record_id = add.json.as_ref().unwrap()["Id"].as_i64().unwrap();

        // 3. List records, find the MX entry, verify priority is 10
        let list = support::hoppy_live_json(&["dns", "record", "list", "--zone-id", &zone_id_str]);
        assert!(list.success, "list records failed: {}", list.stderr);
        let records = list.json.as_ref().unwrap().as_array().unwrap();
        let mx = records
            .iter()
            .find(|r| r["Id"].as_i64() == Some(record_id))
            .expect("MX record not found in list");
        assert_eq!(
            mx["Priority"].as_i64().unwrap_or(0),
            10,
            "expected MX priority 10, got {:?}",
            mx["Priority"]
        );

        // Zone cleanup handled by cleanup stack
    });
}

#[tokio::test]
async fn dns_record_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/dnszone/50001/records/100001"))
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
}

#[tokio::test]
async fn dns_record_add_mx_with_priority() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/dnszone/50001/records"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
            support::fixture("core/dnsrecord_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "dns",
            "record",
            "add",
            "--zone-id",
            "50001",
            "--type",
            "MX",
            "--name",
            "mail",
            "--value",
            "mail.example.com",
            "--ttl",
            "300",
            "--priority",
            "10",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn dns_zone_statistics_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_statistics.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "dns",
            "zone",
            "statistics",
            "--id",
            "50001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
    assert!(json["TotalQueriesServed"].is_number());
}
