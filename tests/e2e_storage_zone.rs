mod e2e_support;

use e2e_support::{cmd, server, skip_in_live_mode};
use predicates::prelude::*;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

const FIXTURE_LIST: &str = include_str!("fixtures/core/storagezone_list_paginated.json");
const FIXTURE_GET: &str = include_str!("fixtures/core/storagezone_get.json");
const FIXTURE_CREATE: &str = include_str!("fixtures/core/storagezone_create.json");

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn storage_zone_list_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/storagezone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["storage-zone", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-storage-zone-1"))
        .stdout(predicate::str::contains("test-storage-zone-4"));
}

#[tokio::test]
async fn storage_zone_list_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/storagezone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--format", "json", "storage-zone", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"CurrentPage\""))
        .stdout(predicate::str::contains("\"TotalItems\""));
}

#[tokio::test]
async fn storage_zone_list_with_search() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/storagezone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("search", "test-storage-zone-1"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["storage-zone", "list", "--search", "test-storage-zone-1"])
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// Get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn storage_zone_get_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/storagezone/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["storage-zone", "get", "--id", "9001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-storage-zone-1"))
        .stdout(predicate::str::contains("storage.bunnycdn.com"));
}

#[tokio::test]
async fn storage_zone_get_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/storagezone/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--format", "json", "storage-zone", "get", "--id", "9001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"Id\": 9001"));
}

// ---------------------------------------------------------------------------
// Create
// ---------------------------------------------------------------------------

#[tokio::test]
async fn storage_zone_create() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("POST"))
        .and(path("/storagezone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(201).set_body_raw(FIXTURE_CREATE, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "storage-zone",
            "create",
            "--name",
            "hoppy-test-zone",
            "--region",
            "DE",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("hoppy-test-zone"));
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

#[tokio::test]
async fn storage_zone_delete_with_yes_flag() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("DELETE"))
        .and(path("/storagezone/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--yes", "storage-zone", "delete", "--id", "9001"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Deleted storage zone 9001"));
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn storage_zone_get_not_found() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/storagezone/99999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_string("{\"Message\":\"Object with the requested ID does not exist.\"}"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["storage-zone", "get", "--id", "99999"])
        .assert()
        .failure();
}
