use super::support;

use wiremock::matchers::{header, method, path, query_param};
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
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(json["Items"].is_array(), "expected Items array");
    let items = json["Items"].as_array().unwrap();
    assert!(!items.is_empty(), "expected at least one DNS zone");
    assert!(
        items[0]["Id"].is_number(),
        "expected zone Id to be a number"
    );
    assert!(
        items[0]["Domain"].is_string(),
        "expected Domain to be a string"
    );
    assert!(
        json["TotalItems"].is_number(),
        "expected TotalItems to be a number"
    );
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Check column headers
    assert!(stdout.contains("ID"), "expected ID column");
    assert!(stdout.contains("Domain"), "expected Domain column");
    assert!(stdout.contains("Records"), "expected Records column");
    assert!(
        stdout.contains("NS Detected"),
        "expected NS Detected column"
    );
    assert!(stdout.contains("DNSSEC"), "expected DNSSEC column");
    assert!(stdout.contains("Created"), "expected Created column");
    // At least one data row present beneath the header.  A table data row
    // starts with "| " followed by a non-whitespace cell value; matching on
    // that pattern is more discriminating than counting total lines.
    let data_rows = stdout
        .lines()
        .filter(|l| support::DATA_ROW_RE.is_match(l))
        .count();
    assert!(
        data_rows >= 1,
        "expected at least one data row, got {data_rows} matching lines"
    );
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
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(json["Id"].is_number(), "expected Id to be a number");
    assert!(json["Domain"].is_string(), "expected Domain to be a string");
    assert!(
        json["Records"].is_array(),
        "expected Records to be an array"
    );
    assert!(
        json["DnsSecEnabled"].is_boolean(),
        "expected DnsSecEnabled to be a boolean"
    );
    assert!(
        json["NameserversDetected"].is_boolean(),
        "expected NameserversDetected to be a boolean"
    );
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Vertical layout: Field/Value columns with zone properties as rows
    assert!(stdout.contains("Field"), "expected Field column");
    assert!(stdout.contains("Value"), "expected Value column");
    assert!(stdout.contains("Id"), "expected Id row");
    assert!(stdout.contains("Domain"), "expected Domain row");
    assert!(
        stdout.contains("DnsSecEnabled"),
        "expected DnsSecEnabled row"
    );
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
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(json["Id"].is_number(), "expected Id to be a number");
    assert!(json["Domain"].is_string(), "expected Domain to be a string");
    assert!(
        json["DnsSecEnabled"].is_boolean(),
        "expected DnsSecEnabled to be a boolean"
    );
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
    // `dns record list` is backed by the dedicated paginated records endpoint.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001/records"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_records_paginated.json"),
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
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(json.is_array(), "expected top-level JSON array of records");
    let records = json.as_array().unwrap();
    assert!(!records.is_empty(), "expected at least one record");
    assert!(
        records[0]["Id"].is_number(),
        "expected record Id to be a number"
    );
    assert!(
        records[0]["Type"].is_number(),
        "expected record Type to be a number"
    );
    assert!(
        records[0]["Value"].is_string(),
        "expected record Value to be a string"
    );
}

#[tokio::test]
async fn dns_record_list_all_paginates() {
    // `--all` should auto-paginate the dedicated records endpoint.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001/records"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_records_paginated.json"),
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
            "--all",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    let records = json.as_array().expect("array of records");
    assert_eq!(records.len(), 2, "expected both records accumulated");
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
    // The update handler now reads the zone first (read-modify-write).
    Mock::given(method("GET"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_get_with_srv.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;
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

/// Partial update must be non-lossy: with only `--ttl` supplied, the handler
/// reads the current SRV record and re-sends its Type/Value/Port/Weight/Priority
/// so they survive the round-trip (previously `--type`/`--value` were required
/// and SRV port/weight were dropped).
#[tokio::test]
async fn dns_record_update_partial_preserves_srv_fields() {
    use wiremock::matchers::body_string_contains;

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_get_with_srv.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/dnszone/50001/records/100001"))
        .and(header("AccessKey", "test-api-key"))
        // Type=8 (SRV), original value/port/weight/priority preserved,
        // Ttl overridden to the requested 120.
        .and(body_string_contains("\"Value\":\"sip.example.com\""))
        .and(body_string_contains("\"Port\":5060"))
        .and(body_string_contains("\"Weight\":5"))
        .and(body_string_contains("\"Ttl\":120"))
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
            "--ttl",
            "120",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "partial update failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// `dns record add --disabled true` must serialise the `Disabled` flag.
#[tokio::test]
async fn dns_record_add_disabled() {
    use wiremock::matchers::body_string_contains;

    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/dnszone/50001/records"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_string_contains("\"Disabled\":true"))
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
            "add",
            "--zone-id",
            "50001",
            "--type",
            "A",
            "--value",
            "192.0.2.1",
            "--disabled",
            "true",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "add --disabled failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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
fn live_dns_zone_dnssec_lifecycle() {
    support::run_lifecycle(|cleanup| {
        let domain = format!("{}.test", support::unique_name("hoppytest"));

        // 1. Create zone
        let create = support::hoppy_live_json(&["dns", "zone", "create", "--domain", &domain]);
        assert!(create.success, "create zone failed: {}", create.stderr);
        let zone_id = create.json.as_ref().unwrap()["Id"].as_i64().unwrap();
        let zone_id_str = zone_id.to_string();
        cleanup.push(&["dns", "zone", "delete", "--id", &zone_id_str]);

        // 2. Initial status: DNSSEC disabled
        let status_before =
            support::hoppy_live_json(&["dns", "zone", "dnssec", "status", "--id", &zone_id_str]);
        assert!(
            status_before.success,
            "status failed: {}",
            status_before.stderr
        );
        assert!(
            !status_before.json.as_ref().unwrap()["DnsSecEnabled"]
                .as_bool()
                .unwrap_or(true),
            "expected DnsSecEnabled to start as false"
        );

        // 3. Enable DNSSEC
        let enable =
            support::hoppy_live_json(&["dns", "zone", "dnssec", "enable", "--id", &zone_id_str]);
        assert!(enable.success, "enable dnssec failed: {}", enable.stderr);
        assert_eq!(
            enable.json.as_ref().unwrap()["Enabled"].as_bool(),
            Some(true)
        );

        // 4. Verify status now true
        let status_after =
            support::hoppy_live_json(&["dns", "zone", "dnssec", "status", "--id", &zone_id_str]);
        assert!(
            status_after.success,
            "status failed: {}",
            status_after.stderr
        );
        assert!(
            status_after.json.as_ref().unwrap()["DnsSecEnabled"]
                .as_bool()
                .unwrap_or(false),
            "expected DnsSecEnabled to be true after enable"
        );

        // 5. Disable DNSSEC (with --yes)
        let disable = support::hoppy_live_json_yes(&[
            "dns",
            "zone",
            "dnssec",
            "disable",
            "--id",
            &zone_id_str,
        ]);
        assert!(disable.success, "disable dnssec failed: {}", disable.stderr);

        // 6. Zone cleanup handled by cleanup stack
    });
}

#[cfg(feature = "live-api")]
#[test]
fn live_dns_zone_record_scan_lifecycle() {
    support::run_lifecycle(|cleanup| {
        let domain = format!("{}.test", support::unique_name("hoppytest"));

        // 1. Create zone
        let create = support::hoppy_live_json(&["dns", "zone", "create", "--domain", &domain]);
        assert!(create.success, "create zone failed: {}", create.stderr);
        let zone_id = create.json.as_ref().unwrap()["Id"].as_i64().unwrap();
        let zone_id_str = zone_id.to_string();
        cleanup.push(&["dns", "zone", "delete", "--id", &zone_id_str]);

        // 2. Trigger scan
        let trigger =
            support::hoppy_live_json(&["dns", "zone", "scan", "start", "--id", &zone_id_str]);
        assert!(trigger.success, "scan start failed: {}", trigger.stderr);
        assert!(trigger.json.as_ref().unwrap()["JobId"].is_string());

        // 3. Poll for results (up to ~30s), breaking early on a terminal state.
        let mut attempts = 0;
        let mut got_status: Option<i64> = None;
        while attempts < 15 {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let res =
                support::hoppy_live_json(&["dns", "zone", "scan", "results", "--id", &zone_id_str]);
            attempts += 1;
            if !res.success {
                continue;
            }
            if let Some(s) = res.json.as_ref().and_then(|j| j["Status"].as_i64()) {
                got_status = Some(s);
                if s == 2 || s == 3 {
                    break;
                }
            }
        }
        // How long a scan takes to finish is the API's business, not hoppy's:
        // real scans can sit Pending well past any reasonable poll budget
        // (observed 2026-05-14). Only assert that the scan is queryable and
        // reports a well-typed status: Pending (0), InProgress (1),
        // Completed (2), or Failed (3).
        assert!(
            matches!(got_status, Some(0..=3)),
            "scan results returned no well-typed status (last status: {got_status:?})"
        );

        // 4. Zone cleanup handled by cleanup stack
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

// ---------------------------------------------------------------------------
// DNSSEC tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dns_zone_dnssec_enable_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/dnszone/50001/dnssec"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnssec_enable.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format", "json", "dns", "zone", "dnssec", "enable", "--id", "50001",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
    assert_eq!(json["Enabled"], true);
    assert_eq!(json["KeyTag"], 12345);
    assert_eq!(json["Algorithm"], 13);
}

#[tokio::test]
async fn dns_zone_dnssec_enable_table_shows_ds_record() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/dnszone/50001/dnssec"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnssec_enable.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format", "table", "dns", "zone", "dnssec", "enable", "--id", "50001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("DS record"));
    assert!(stderr.contains("12345"));
}

#[tokio::test]
async fn dns_zone_dnssec_disable_with_yes() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/dnszone/50001/dnssec"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnssec_disable.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes", "--format", "json", "dns", "zone", "dnssec", "disable", "--id", "50001",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
    assert_eq!(json["Enabled"], false);
}

#[tokio::test]
async fn dns_zone_dnssec_status_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format", "json", "dns", "zone", "dnssec", "status", "--id", "50001",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
    assert!(json["Id"].is_number(), "expected Id to be a number");
    assert!(
        json["DnsSecEnabled"].is_boolean(),
        "expected DnsSecEnabled to be a boolean"
    );
}

// ---------------------------------------------------------------------------
// Wildcard certificate tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dns_zone_issue_cert_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/dnszone/50001/certificate/issue"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["dns", "zone", "issue-cert", "--id", "50001"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Issued wildcard certificate"));
}

#[tokio::test]
async fn dns_zone_issue_cert_500_appends_delegation_hint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/dnszone/50001/certificate/issue"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_json(serde_json::json!({"Message": "An error has occurred."})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["dns", "zone", "issue-cert", "--id", "50001"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("hint:") && stderr.contains("delegated to bunny.net nameservers"),
        "expected delegation hint in stderr, got: {stderr}"
    );
    assert!(
        stderr.contains("hoppy dns zone get --id 50001"),
        "expected command suggestion in stderr, got: {stderr}"
    );
}

#[tokio::test]
async fn dns_zone_get_500_does_not_get_issue_cert_hint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_json(serde_json::json!({"Message": "An error has occurred."})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["dns", "zone", "get", "--id", "50001"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("delegated to bunny.net nameservers"),
        "delegation hint should not appear on unrelated commands: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// DNS record scan tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dns_zone_scan_start_with_id_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/dnszone/records/scan"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_scan_trigger.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format", "json", "dns", "zone", "scan", "start", "--id", "50001",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
    assert!(json["JobId"].is_string());
}

#[tokio::test]
async fn dns_zone_scan_start_with_domain_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/dnszone/records/scan"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_scan_trigger.json"),
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
            "scan",
            "start",
            "--domain",
            "example.com",
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
async fn dns_zone_scan_start_requires_id_or_domain() {
    let server = MockServer::start().await;
    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["dns", "zone", "scan", "start"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--id") || stderr.contains("--domain"));
}

#[tokio::test]
async fn dns_zone_scan_results_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001/records/scan"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_scan_result.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format", "json", "dns", "zone", "scan", "results", "--id", "50001",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
    assert_eq!(json["ZoneId"], 50001);
    let records = json["Records"].as_array().expect("records array");
    assert_eq!(records.len(), 3);
}

#[tokio::test]
async fn dns_zone_scan_results_by_domain_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone"))
        .and(query_param("search", "example.com"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Items": [{
                "Id": 50001,
                "Domain": "example.com",
                "Records": [],
                "DateModified": "2026-06-01T00:00:00",
                "DateCreated": "2026-06-01T00:00:00",
                "NameserversDetected": true,
                "CustomNameserversEnabled": false,
                "Nameserver1": "kiki.bunny.net",
                "Nameserver2": "coco.bunny.net",
                "SoaEmail": "hostmaster@bunny.net",
                "NameserversNextCheck": "2026-06-01T00:05:00",
                "LoggingEnabled": false,
                "LoggingIPAnonymizationEnabled": true,
                "LogAnonymizationType": 0,
                "DnsSecEnabled": false,
                "CertificateKeyType": 0
            }],
            "CurrentPage": 1,
            "TotalItems": 1,
            "HasMoreItems": false
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001/records/scan"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_scan_result.json"),
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
            "scan",
            "results",
            "--domain",
            "example.com",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
    assert_eq!(json["ZoneId"], 50001);
}

#[tokio::test]
async fn dns_zone_scan_results_by_domain_table_shows_domain() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone"))
        .and(query_param("search", "example.com"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Items": [{
                "Id": 50001,
                "Domain": "example.com",
                "Records": [],
                "DateModified": "2026-06-01T00:00:00",
                "DateCreated": "2026-06-01T00:00:00",
                "NameserversDetected": true,
                "CustomNameserversEnabled": false,
                "Nameserver1": "kiki.bunny.net",
                "Nameserver2": "coco.bunny.net",
                "SoaEmail": "hostmaster@bunny.net",
                "NameserversNextCheck": "2026-06-01T00:05:00",
                "LoggingEnabled": false,
                "LoggingIPAnonymizationEnabled": true,
                "LogAnonymizationType": 0,
                "DnsSecEnabled": false,
                "CertificateKeyType": 0
            }],
            "CurrentPage": 1,
            "TotalItems": 1,
            "HasMoreItems": false
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001/records/scan"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_scan_result_no_domain.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["dns", "zone", "scan", "results", "--domain", "example.com"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("example.com"),
        "expected resolved domain in table output, stdout was: {stdout}"
    );
}

#[tokio::test]
async fn dns_zone_scan_results_by_id_table_shows_domain() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001/records/scan"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_scan_result_no_domain.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Id": 50001,
            "Domain": "resolved.example",
            "Records": [],
            "DateModified": "2026-06-01T00:00:00",
            "DateCreated": "2026-06-01T00:00:00",
            "NameserversDetected": true,
            "CustomNameserversEnabled": false,
            "Nameserver1": "kiki.bunny.net",
            "Nameserver2": "coco.bunny.net",
            "SoaEmail": "hostmaster@bunny.net",
            "NameserversNextCheck": "2026-06-01T00:05:00",
            "LoggingEnabled": false,
            "LoggingIPAnonymizationEnabled": true,
            "LogAnonymizationType": 0,
            "DnsSecEnabled": false,
            "CertificateKeyType": 0
        })))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["dns", "zone", "scan", "results", "--id", "50001"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("resolved.example"),
        "expected resolved domain in table output, stdout was: {stdout}"
    );
}

#[tokio::test]
async fn dns_zone_scan_results_by_domain_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone"))
        .and(query_param("search", "missing.example"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Items": [],
            "CurrentPage": 1,
            "TotalItems": 0,
            "HasMoreItems": false
        })))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "dns",
            "zone",
            "scan",
            "results",
            "--domain",
            "missing.example",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no DNS zone found"), "stderr was: {stderr}");
}

#[tokio::test]
async fn dns_zone_scan_results_requires_id_or_domain() {
    let server = MockServer::start().await;
    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["dns", "zone", "scan", "results"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--id") || stderr.contains("--domain"));
}

#[tokio::test]
async fn dns_zone_scan_start_prints_next_command_hint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/dnszone/records/scan"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_scan_trigger.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["dns", "zone", "scan", "start", "--domain", "example.com"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("hoppy dns zone scan results --domain example.com"),
        "expected next-command hint, stderr was: {stderr}"
    );
}

#[tokio::test]
async fn dns_zone_scan_start_json_suppresses_hint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/dnszone/records/scan"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_scan_trigger.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format", "json", "dns", "zone", "scan", "start", "--id", "50001",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("hoppy dns zone scan results"),
        "JSON output should not emit the next-command hint, stderr was: {stderr}"
    );
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

// ---------------------------------------------------------------------------
// iter-60: dns zone export --format and empty-zone behaviour
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dns_zone_export_format_json_wraps_bind_envelope() {
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
        .args(["--format", "json", "dns", "zone", "export", "--id", "50001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    let bind = json["Bind"].as_str().expect("Bind field must be a string");
    assert!(bind.contains("$ORIGIN"));
    assert!(bind.contains("example.com"));
}

#[tokio::test]
async fn dns_zone_export_format_table_aliases_text() {
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
        .args([
            "--format", "table", "dns", "zone", "export", "--id", "50001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("$ORIGIN"));
    assert!(stdout.contains("example.com"));
}

#[tokio::test]
async fn dns_zone_export_empty_zone_text_emits_comment() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001/export"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("", "text/plain"))
        .expect(1)
        .mount(&server)
        .await;
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
        .args(["dns", "zone", "export", "--id", "50001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "empty-zone export must not be silent"
    );
    assert!(stdout.contains(";; zone"));
    assert!(stdout.contains("0 records"));
}

#[tokio::test]
async fn dns_zone_export_empty_zone_json_emits_envelope() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001/export"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("   \n", "text/plain"))
        .expect(1)
        .mount(&server)
        .await;
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
        .args(["--format", "json", "dns", "zone", "export", "--id", "50001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    let bind = json["Bind"].as_str().expect("Bind field must be a string");
    assert!(bind.contains(";; zone"));
    assert!(bind.contains("0 records"));
}

#[tokio::test]
async fn dns_zone_export_empty_zone_table_emits_comment() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001/export"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("", "text/plain"))
        .expect(1)
        .mount(&server)
        .await;
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
            "--format", "table", "dns", "zone", "export", "--id", "50001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(";; zone"));
    assert!(stdout.contains("0 records"));
}

#[tokio::test]
async fn dns_zone_export_text_snapshot_matches_bind() {
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
        .args(["--format", "text", "dns", "zone", "export", "--id", "50001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let fixture: String = support::fixture("core/dnszone_export.txt");
    // The fixture is the raw BIND text the API returns. The CLI's text output
    // must preserve the BIND payload verbatim (with a trailing newline appended
    // if missing) — no envelope, no extra framing.
    let expected = if fixture.ends_with('\n') {
        fixture
    } else {
        format!("{fixture}\n")
    };
    assert_eq!(stdout, expected);
}

// ---------------------------------------------------------------------------
// iter-71 — smart routing / linked records / zone availability
// ---------------------------------------------------------------------------

/// `dns record add` with smart-routing + geolocation flags must serialise the
/// SmartRoutingType/GeolocationLatitude/GeolocationLongitude fields.
#[tokio::test]
async fn dns_record_add_smart_routing_geo() {
    use wiremock::matchers::body_string_contains;

    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/dnszone/50001/records"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_string_contains("\"SmartRoutingType\":2"))
        .and(body_string_contains("\"GeolocationLatitude\":51.5"))
        .and(body_string_contains("\"GeolocationLongitude\":-0.1"))
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
            "add",
            "--zone-id",
            "50001",
            "--type",
            "A",
            "--value",
            "192.0.2.1",
            "--smart-routing-type",
            "geolocation",
            "--geolocation-latitude",
            "51.5",
            // Negative coordinate passed in the natural spaced form — works
            // because the geolocation args set `allow_hyphen_values`.
            "--geolocation-longitude",
            "-0.1",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// `dns record add --type PullZone --pull-zone-id` must serialise PullZoneId,
/// which previously had no flag at all.
#[tokio::test]
async fn dns_record_add_linked_pull_zone() {
    use wiremock::matchers::body_string_contains;

    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/dnszone/50001/records"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_string_contains("\"PullZoneId\":1234"))
        .and(body_string_contains("\"MonitorType\":1"))
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
            "add",
            "--zone-id",
            "50001",
            "--type",
            "PullZone",
            "--value",
            "@",
            "--pull-zone-id",
            "1234",
            "--monitor-type",
            "ping",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// An invalid `--smart-routing-type` value must be rejected with a helpful
/// error before any HTTP request is made.
#[tokio::test]
async fn dns_record_add_invalid_smart_routing_type_errors() {
    let server = MockServer::start().await;
    // No mock mounted — the command must fail before hitting the network.
    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "dns",
            "record",
            "add",
            "--zone-id",
            "50001",
            "--type",
            "A",
            "--value",
            "192.0.2.1",
            "--smart-routing-type",
            "bogus",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown smart routing type"),
        "expected smart-routing parse error, got: {stderr}"
    );
}

/// `dns zone check --domain` posts to /dnszone/checkavailability and prints the
/// availability result.
#[tokio::test]
async fn dns_zone_check_available() {
    use wiremock::matchers::body_string_contains;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/dnszone/checkavailability"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_string_contains("\"Name\":\"example.com\""))
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
            "dns",
            "zone",
            "check",
            "--domain",
            "example.com",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid json");
    assert_eq!(json["Available"], true);
}

/// `dns zone update --log-anonymization-type drop` must serialise the enum as
/// its integer discriminant (Drop = 1).
#[tokio::test]
async fn dns_zone_update_log_anonymization_type() {
    use wiremock::matchers::body_string_contains;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_string_contains("\"LogAnonymizationType\":1"))
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
            "--log-anonymization-type",
            "drop",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "update failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
