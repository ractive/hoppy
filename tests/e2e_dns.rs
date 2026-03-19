mod e2e_support;

use e2e_support::{cmd, server, skip_in_live_mode};
use predicates::prelude::*;
use wiremock::matchers::{body_partial_json, header, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

const FIXTURE_LIST: &str =
    include_str!("../crates/bunny-api-core/tests/fixtures/dnszone_list_paginated.json");
const FIXTURE_GET: &str = include_str!("../crates/bunny-api-core/tests/fixtures/dnszone_get.json");
const FIXTURE_CREATE: &str =
    include_str!("../crates/bunny-api-core/tests/fixtures/dnszone_create.json");
const FIXTURE_RECORD_ADD: &str =
    include_str!("../crates/bunny-api-core/tests/fixtures/dnsrecord_add.json");

// ---------------------------------------------------------------------------
// DNS Zone — List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dns_zone_list_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["dns", "zone", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("example.com"))
        .stdout(predicate::str::contains("test-site.org"));
}

#[tokio::test]
async fn dns_zone_list_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--format", "json", "dns", "zone", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"CurrentPage\""))
        .stdout(predicate::str::contains("\"TotalItems\""));
}

// ---------------------------------------------------------------------------
// DNS Zone — Get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dns_zone_get_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["dns", "zone", "get", "--id", "50001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("example.com"))
        .stdout(predicate::str::contains("50001"));
}

#[tokio::test]
async fn dns_zone_get_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--format", "json", "dns", "zone", "get", "--id", "50001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"Id\": 50001"))
        .stdout(predicate::str::contains("\"Domain\": \"example.com\""));
}

// ---------------------------------------------------------------------------
// DNS Zone — Create
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dns_zone_create() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("POST"))
        .and(path("/dnszone"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_partial_json(serde_json::json!({
            "Domain": "hoppy-test.example"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_raw(FIXTURE_CREATE, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["dns", "zone", "create", "--domain", "hoppy-test.example"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hoppy-test.example"));
}

// ---------------------------------------------------------------------------
// DNS Zone — Delete
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dns_zone_delete_with_yes_flag() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("DELETE"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--yes", "dns", "zone", "delete", "--id", "50001"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Deleted DNS zone 50001"));
}

// ---------------------------------------------------------------------------
// DNS Record — List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dns_record_list_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    // Records are embedded in the zone response; the CLI calls GET /dnszone/{id}
    Mock::given(method("GET"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["dns", "record", "list", "--zone-id", "50001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("93.184.216.34"))
        .stdout(predicate::str::contains("www"));
}

#[tokio::test]
async fn dns_record_list_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format",
            "json",
            "dns",
            "record",
            "list",
            "--zone-id",
            "50001",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"Id\": 100001"));
}

// ---------------------------------------------------------------------------
// DNS Record — Add
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dns_record_add() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("PUT"))
        .and(path("/dnszone/50001/records"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_partial_json(serde_json::json!({
            "Type": 0,
            "Value": "192.0.2.1",
            "Name": "test",
            "Ttl": 300
        })))
        .respond_with(
            ResponseTemplate::new(201).set_body_raw(FIXTURE_RECORD_ADD, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
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
            "--name",
            "test",
            "--ttl",
            "300",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("192.0.2.1"));
}

// ---------------------------------------------------------------------------
// DNS Record — Delete
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dns_record_delete_with_yes_flag() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("DELETE"))
        .and(path("/dnszone/50001/records/100001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--yes",
            "dns",
            "record",
            "delete",
            "--zone-id",
            "50001",
            "--record-id",
            "100001",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Deleted DNS record 100001 from zone 50001",
        ));
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dns_zone_get_not_found() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone/99999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_string("{\"Message\":\"Object with the requested ID does not exist.\"}"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["dns", "zone", "get", "--id", "99999"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error:"));
}
