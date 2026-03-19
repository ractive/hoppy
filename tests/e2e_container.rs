mod e2e_support;

use e2e_support::{cmd, server, skip_in_live_mode};
use predicates::prelude::*;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, ResponseTemplate};

const FIXTURE_APPS_LIST: &str = include_str!("fixtures/containers/apps_list.json");
const FIXTURE_APP_GET: &str = include_str!("fixtures/containers/app_get.json");
const FIXTURE_APP_ADD: &str = include_str!("fixtures/containers/app_add.json");
const FIXTURE_ENDPOINTS_LIST: &str = include_str!("fixtures/containers/endpoints_list.json");
const FIXTURE_VOLUMES_LIST: &str = include_str!("fixtures/containers/volumes_list.json");
const FIXTURE_REGISTRIES_LIST: &str = include_str!("fixtures/containers/registries_list.json");
const FIXTURE_REGISTRY_GET: &str = include_str!("fixtures/containers/registry_get.json");
const FIXTURE_REGIONS_LIST: &str = include_str!("fixtures/containers/regions_list.json");
const FIXTURE_NODES_LIST: &str = include_str!("fixtures/containers/nodes_list.json");

// ---------------------------------------------------------------------------
// App — List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_app_list() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/apps"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_APPS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["container", "app", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("stova-api"))
        .stdout(predicate::str::contains("stova-web"));
}

#[tokio::test]
async fn container_app_list_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/apps"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_APPS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--format", "json", "container", "app", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"items\""))
        .stdout(predicate::str::contains("stova-api"));
}

// ---------------------------------------------------------------------------
// App — Get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_app_get() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_APP_GET, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["container", "app", "get", "--id", "l3faCv1fWRHAYNU"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hoppy-test-fixture-tmp"));
}

#[tokio::test]
async fn container_app_get_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_APP_GET, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format",
            "json",
            "container",
            "app",
            "get",
            "--id",
            "l3faCv1fWRHAYNU",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\""))
        .stdout(predicate::str::contains("l3faCv1fWRHAYNU"));
}

#[tokio::test]
async fn container_app_get_not_found() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/apps/does-not-exist"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_string("{\"message\":\"Object with the requested ID does not exist.\"}"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["container", "app", "get", "--id", "does-not-exist"])
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// App — Create
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_app_create() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("POST"))
        .and(path("/apps"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_APP_ADD, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "container",
            "app",
            "create",
            "--name",
            "my-app",
            "--runtime-type",
            "shared",
            "--min",
            "1",
            "--max",
            "2",
            "--region",
            "DE",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("Created application:"));
}

// ---------------------------------------------------------------------------
// Endpoint — List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_endpoint_list() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU/endpoints"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ENDPOINTS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "container",
            "endpoint",
            "list",
            "--app-id",
            "l3faCv1fWRHAYNU",
        ])
        .assert()
        .success();
}

#[tokio::test]
async fn container_endpoint_list_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU/endpoints"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ENDPOINTS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format",
            "json",
            "container",
            "endpoint",
            "list",
            "--app-id",
            "l3faCv1fWRHAYNU",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"items\""));
}

// ---------------------------------------------------------------------------
// Volume — List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_volume_list() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU/volumes"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VOLUMES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["container", "volume", "list", "--app-id", "l3faCv1fWRHAYNU"])
        .assert()
        .success();
}

#[tokio::test]
async fn container_volume_list_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU/volumes"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VOLUMES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format",
            "json",
            "container",
            "volume",
            "list",
            "--app-id",
            "l3faCv1fWRHAYNU",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"items\""));
}

// ---------------------------------------------------------------------------
// Registry — List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_registry_list() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/registries"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_REGISTRIES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["container", "registry", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("GitHub Packages testuser"))
        .stdout(predicate::str::contains("DockerHub Public"));
}

#[tokio::test]
async fn container_registry_list_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/registries"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_REGISTRIES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--format", "json", "container", "registry", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"items\""))
        .stdout(predicate::str::contains("ghcr.io"));
}

// ---------------------------------------------------------------------------
// Registry — Get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_registry_get() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/registries/9999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_REGISTRY_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["container", "registry", "get", "--id", "9999"])
        .assert()
        .success()
        .stdout(predicate::str::contains("GitHub Packages testuser"))
        .stdout(predicate::str::contains("ghcr.io"));
}

#[tokio::test]
async fn container_registry_get_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/registries/9999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_REGISTRY_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format",
            "json",
            "container",
            "registry",
            "get",
            "--id",
            "9999",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\": 9999"));
}

// ---------------------------------------------------------------------------
// Region — List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_region_list() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/regions"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_REGIONS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["container", "region", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("DE"))
        .stdout(predicate::str::contains("AMS"));
}

#[tokio::test]
async fn container_region_list_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/regions"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_REGIONS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--format", "json", "container", "region", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"items\""))
        .stdout(predicate::str::contains("\"id\": \"DE\""));
}

// ---------------------------------------------------------------------------
// Node — List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_node_list() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/nodes"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_NODES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["container", "node", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("104.166.147.46"))
        .stdout(predicate::str::contains("109.61.83.105"));
}

#[tokio::test]
async fn container_node_list_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/nodes"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_NODES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--format", "json", "container", "node", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"items\""))
        .stdout(predicate::str::contains("104.166.147.46"));
}
