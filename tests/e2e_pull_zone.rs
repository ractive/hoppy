mod e2e_support;

use e2e_support::{cmd, server, skip_in_live_mode};
use predicates::prelude::*;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

const FIXTURE_LIST: &str = include_str!("fixtures/core/pullzone_list_paginated.json");
const FIXTURE_GET: &str = include_str!("fixtures/core/pullzone_get.json");

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pull_zone_list_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["pull-zone", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-zone-10"))
        .stdout(predicate::str::contains("test-zone-15"));
}

#[tokio::test]
async fn pull_zone_list_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--format", "json", "pull-zone", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"Items\""))
        .stdout(predicate::str::contains("\"TotalItems\""));
}

#[tokio::test]
async fn pull_zone_list_with_search() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("search", "test"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["pull-zone", "list", "--search", "test"])
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// Get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pull_zone_get_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["pull-zone", "get", "--id", "1001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-zone-19"))
        .stdout(predicate::str::contains("origin.example.com"));
}

#[tokio::test]
async fn pull_zone_get_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--format", "json", "pull-zone", "get", "--id", "1001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"Id\""));
}

// ---------------------------------------------------------------------------
// Create
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pull_zone_create() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("POST"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(201).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "pull-zone",
            "create",
            "--name",
            "my-zone",
            "--origin-url",
            "https://example.com",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-zone-19"));
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pull_zone_delete_with_yes_flag() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("DELETE"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--yes", "pull-zone", "delete", "--id", "1001"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Deleted pull zone 1001"));
}

// ---------------------------------------------------------------------------
// Purge
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pull_zone_purge_all() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("POST"))
        .and(path("/pullzone/1001/purgeCache"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["pull-zone", "purge", "--id", "1001"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Purged cache for pull zone 1001"));
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pull_zone_get_not_found() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone/99999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_string("{\"Message\":\"Object with the requested ID does not exist.\"}"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["pull-zone", "get", "--id", "99999"])
        .assert()
        .failure();
}
