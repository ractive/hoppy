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

/// Storage zone list whose first zone carries the id of the first pull zone's
/// storage origin (3002); the pull zone with storage id 3001 is left unknown.
fn storage_list_for_origin_labels() -> String {
    let mut list: serde_json::Value =
        serde_json::from_str(&support::fixture("core/storagezone_list_paginated.json")).unwrap();
    list["Items"][0]["Id"] = serde_json::json!(3002);
    list["Items"][0]["Name"] = serde_json::json!("origin-store");
    list.to_string()
}

#[tokio::test]
async fn list_table_labels_origins_by_type() {
    let server = MockServer::start().await;
    mount_ordinary_list(&server).await;
    mount_container_side(&server).await;
    get(&server, "/storagezone", storage_list_for_origin_labels()).await;

    let output = cmd(&server)
        .args(["--format", "table", "pull-zone", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("storage: origin-store"), "{stdout}");
    assert!(stdout.contains("storage zone 3001"), "{stdout}");
    assert!(stdout.contains("container app my-app"), "{stdout}");
    assert!(!stdout.contains("https://bunny.net"), "{stdout}");
}

#[tokio::test]
async fn list_table_origin_falls_back_when_storage_lookup_fails() {
    let server = MockServer::start().await;
    mount_ordinary_list(&server).await;
    Mock::given(method("GET"))
        .and(path("/storagezone"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    get(
        &server,
        "/apps",
        r#"{"items":[],"meta":{"totalItems":0},"cursor":null}"#.to_owned(),
    )
    .await;

    let output = cmd(&server)
        .args(["--format", "table", "pull-zone", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("storage zone 3002"), "{stdout}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("storage zone names"));
}

#[tokio::test]
async fn list_json_keeps_raw_origin_fields() {
    let server = MockServer::start().await;
    mount_ordinary_list(&server).await;
    mount_container_side(&server).await;
    let output = cmd(&server)
        .args(["--format", "json", "pull-zone", "list"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["Items"][0]["OriginUrl"], "https://origin.example.com");
    assert_eq!(json["Items"][0]["OriginType"], 2);
    assert_eq!(json["Items"][2]["OriginType"], 5);
}

async fn mount_storage_page(server: &MockServer, page: &str, id: i64, name: &str, more: bool) {
    let mut list: serde_json::Value =
        serde_json::from_str(&support::fixture("core/storagezone_list_paginated.json")).unwrap();
    list["Items"] = serde_json::json!([{ "Id": id, "Name": name, "UserId": "u" }]);
    list["HasMoreItems"] = serde_json::json!(more);
    Mock::given(method("GET"))
        .and(path("/storagezone"))
        .and(wiremock::matchers::query_param("page", page))
        .respond_with(json_response(list.to_string()))
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn list_table_reads_storage_names_across_pages() {
    let server = MockServer::start().await;
    mount_ordinary_list(&server).await;
    get(
        &server,
        "/apps",
        r#"{"items":[],"meta":{"totalItems":0},"cursor":null}"#.to_owned(),
    )
    .await;
    mount_storage_page(&server, "1", 3002, "first-page-store", true).await;
    mount_storage_page(&server, "2", 3001, "second-page-store", false).await;

    let output = cmd(&server)
        .args(["--format", "table", "pull-zone", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("storage: first-page-store"), "{stdout}");
    assert!(stdout.contains("storage: second-page-store"), "{stdout}");
}

#[tokio::test]
async fn list_json_does_not_look_up_storage_zones() {
    let server = MockServer::start().await;
    mount_ordinary_list(&server).await;
    mount_container_side(&server).await;
    Mock::given(method("GET"))
        .and(path("/storagezone"))
        .respond_with(json_response(storage_list_for_origin_labels()))
        .expect(0)
        .mount(&server)
        .await;
    let output = cmd(&server)
        .args(["--format", "json", "pull-zone", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[tokio::test]
async fn list_without_storage_origins_does_not_look_up_storage_zones() {
    let server = MockServer::start().await;
    let mut page: serde_json::Value =
        serde_json::from_str(&support::fixture("core/pullzone_list_paginated.json")).unwrap();
    for item in page["Items"].as_array_mut().unwrap() {
        item["OriginType"] = serde_json::json!(0);
        item["StorageZoneId"] = serde_json::json!(-1);
    }
    get(&server, "/pullzone", page.to_string()).await;
    get(
        &server,
        "/apps",
        r#"{"items":[],"meta":{"totalItems":0},"cursor":null}"#.to_owned(),
    )
    .await;
    Mock::given(method("GET"))
        .and(path("/storagezone"))
        .respond_with(json_response(storage_list_for_origin_labels()))
        .expect(0)
        .mount(&server)
        .await;
    let output = cmd(&server)
        .args(["--format", "table", "pull-zone", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("https://origin.example.com"));
}
