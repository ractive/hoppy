//! `pull-zone list` / `count` include the pull zones owned by Magic
//! Containers apps, which `GET /pullzone` and `GET /pullzone/count` omit.

use super::support;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const MC_ZONE_ID: i64 = 5719318;

fn json_response(body: String) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(body, "application/json")
}

async fn get(server: &MockServer, url_path: &str, body: String) {
    Mock::given(method("GET"))
        .and(path(url_path))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(json_response(body))
        .mount(server)
        .await;
}

/// One container app with a single CDN endpoint whose pull zone is the
/// Magic Containers fixture zone.
async fn mount_container_side(server: &MockServer) {
    get(
        server,
        "/apps",
        r#"{"items":[{"id":"app1","name":"my-app","status":"active"}],"meta":{"totalItems":1},"cursor":null}"#
            .to_owned(),
    )
    .await;
    get(
        server,
        "/apps/app1/endpoints",
        format!(
            r#"{{"items":[{{"id":"app1-ep","displayName":"cdn","publicHost":"x.b-cdn.net","type":"cdn","isSslEnabled":true,"pullZoneId":"{MC_ZONE_ID}","portMappings":[]}}],"meta":{{"totalItems":1}},"cursor":null}}"#
        ),
    )
    .await;
    get(
        server,
        &format!("/pullzone/{MC_ZONE_ID}"),
        support::fixture("core/pullzone_get_magic_container.json"),
    )
    .await;
}

async fn mount_ordinary_list(server: &MockServer) {
    get(
        server,
        "/pullzone",
        support::fixture("core/pullzone_list_paginated.json"),
    )
    .await;
}

fn cmd(server: &MockServer) -> assert_cmd::Command {
    support::hoppy_mock_cmd("test-api-key", &server.uri())
}

#[tokio::test]
async fn list_json_appends_container_zone_with_app() {
    let server = MockServer::start().await;
    mount_ordinary_list(&server).await;
    mount_container_side(&server).await;

    let output = cmd(&server)
        .args(["--format", "json", "pull-zone", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let items = json["Items"].as_array().unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[2]["Id"], MC_ZONE_ID);
    assert_eq!(items[2]["ContainerApp"]["Id"], "app1");
    assert_eq!(items[2]["ContainerApp"]["Name"], "my-app");
    assert!(items[0].get("ContainerApp").is_none());
}

#[tokio::test]
async fn list_table_adds_app_column_only_with_container_zones() {
    let server = MockServer::start().await;
    mount_ordinary_list(&server).await;
    mount_container_side(&server).await;
    let output = cmd(&server)
        .args(["--format", "table", "pull-zone", "list"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("App"), "{stdout}");
    assert!(stdout.contains("my-app"), "{stdout}");
    assert!(stdout.contains("magic-container-auto-pz"), "{stdout}");

    // No container apps: the table is unchanged (no App column).
    let plain = MockServer::start().await;
    mount_ordinary_list(&plain).await;
    get(
        &plain,
        "/apps",
        r#"{"items":[],"meta":{"totalItems":0},"cursor":null}"#.to_owned(),
    )
    .await;
    let output = cmd(&plain)
        .args(["--format", "table", "pull-zone", "list"])
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&output.stdout).contains("App"));
}

#[tokio::test]
async fn list_all_includes_container_zone_once() {
    let server = MockServer::start().await;
    // The API unexpectedly returns the MC zone too: it must not be duplicated.
    let mut page: serde_json::Value =
        serde_json::from_str(&support::fixture("core/pullzone_list_paginated.json")).unwrap();
    let mc: serde_json::Value =
        serde_json::from_str(&support::fixture("core/pullzone_get_magic_container.json")).unwrap();
    page["Items"].as_array_mut().unwrap().push(mc);
    get(&server, "/pullzone", page.to_string()).await;
    mount_container_side(&server).await;

    let output = cmd(&server)
        .args(["--format", "json", "pull-zone", "list", "--all"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let items = json["Items"].as_array().unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[2]["ContainerApp"]["Name"], "my-app");
    assert_eq!(json["TotalItems"], 3);
}

#[tokio::test]
async fn list_no_container_zones_skips_lookup() {
    let server = MockServer::start().await;
    mount_ordinary_list(&server).await;
    // No container mocks: a lookup would 404 and warn.
    let output = cmd(&server)
        .args([
            "--format",
            "json",
            "pull-zone",
            "list",
            "--no-container-zones",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["Items"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn list_survives_container_api_failure_with_warning() {
    let server = MockServer::start().await;
    mount_ordinary_list(&server).await;
    Mock::given(method("GET"))
        .and(path("/apps"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let output = cmd(&server)
        .args(["--format", "json", "pull-zone", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Magic Containers"));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["Items"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn count_includes_container_zones() {
    let server = MockServer::start().await;
    get(&server, "/pullzone/count", r#"{"Count":2}"#.to_owned()).await;
    mount_ordinary_list(&server).await;
    mount_container_side(&server).await;
    let output = cmd(&server)
        .args(["--format", "json", "pull-zone", "count"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["Count"], 3);
}

#[tokio::test]
async fn count_without_container_zones_keeps_api_count() {
    let server = MockServer::start().await;
    get(&server, "/pullzone/count", r#"{"Count":7}"#.to_owned()).await;
    get(
        &server,
        "/apps",
        r#"{"items":[],"meta":{"totalItems":0},"cursor":null}"#.to_owned(),
    )
    .await;
    let output = cmd(&server)
        .args(["--format", "json", "pull-zone", "count"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["Count"], 7);

    let output = cmd(&server)
        .args([
            "--format",
            "json",
            "pull-zone",
            "count",
            "--no-container-zones",
        ])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["Count"], 7);
}
