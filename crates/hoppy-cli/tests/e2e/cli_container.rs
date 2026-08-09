use super::support;

use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Helper: build a command pointed at the mock server for containers
// ---------------------------------------------------------------------------

fn mock_cmd(api_key: &str, server_uri: &str) -> assert_cmd::Command {
    support::hoppy_mock_cmd_full(api_key, server_uri, None, None, Some(server_uri))
}

// ---------------------------------------------------------------------------
// Application — JSON snapshot tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_app_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/apps"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/apps_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "container", "app", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_app_list_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/apps"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/apps_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "container", "app", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_app_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/app_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "app",
            "get",
            "--id",
            "l3faCv1fWRHAYNU",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_app_get_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/app_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "table",
            "container",
            "app",
            "get",
            "--id",
            "l3faCv1fWRHAYNU",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_app_create_json() {
    // --minimal preserves the legacy `{"id": "..."}` shape; this test pins
    // that opt-in path so downstream tooling that relies on the thin output
    // keeps working.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/apps"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/app_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "app",
            "create",
            "--minimal",
            "--name",
            "hoppy-test-app",
            "--runtime-type",
            "shared",
            "--min",
            "1",
            "--max",
            "1",
            "--region",
            "DE",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_app_create_json_default_full_object() {
    // Default (no --minimal): create returns the full app document so callers
    // don't need a follow-up `app get` to chain template / endpoint ids.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/apps"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/app_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/apps/l3faCv1fWRHAYNU"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/app_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "app",
            "create",
            "--name",
            "hoppy-test-app",
            "--runtime-type",
            "shared",
            "--min",
            "1",
            "--max",
            "1",
            "--region",
            "DE",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("containerTemplates"),
        "default create should return full app document, got: {stdout}"
    );
    assert!(
        stdout.contains("\"id\": \"l3faCv1fWRHAYNU\""),
        "expected full app id in output, got: {stdout}"
    );
}

#[tokio::test]
async fn container_app_overview_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/apps/test-app-id/overview"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/app_overview.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "app",
            "overview",
            "--id",
            "test-app-id",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn container_app_statistics_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/apps/test-app-id/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("fromDate", "2026-03-01"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/app_statistics.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "app",
            "statistics",
            "--id",
            "test-app-id",
            "--from",
            "2026-03-01",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn container_app_autoscaling_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/apps/test-app-id/autoscaling"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/autoscaling_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "app",
            "autoscaling-get",
            "--app-id",
            "test-app-id",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_app_region_settings_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/apps/test-app-id/region-settings"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/region_settings_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "app",
            "region-settings-get",
            "--app-id",
            "test-app-id",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

// ---------------------------------------------------------------------------
// Template — JSON snapshot tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_template_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/apps/test-app-id/containers/test-container-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/container_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "template",
            "get",
            "--app-id",
            "test-app-id",
            "--container-id",
            "test-container-id",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_template_add_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/apps/test-app-id/containers"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/container_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "template",
            "add",
            "--app-id",
            "test-app-id",
            "--name",
            "nginx",
            "--image-name",
            "nginx",
            "--image-namespace",
            "library",
            "--image-tag",
            "alpine",
            "--registry-id",
            "1155",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

// ---------------------------------------------------------------------------
// Endpoint — JSON snapshot tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_endpoint_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/apps/test-app-id/endpoints"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/endpoints_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "endpoint",
            "list",
            "--app-id",
            "test-app-id",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_endpoint_add_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/apps/test-app-id/containers/test-container-id/endpoints",
        ))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/endpoint_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "endpoint",
            "add",
            "--app-id",
            "test-app-id",
            "--container-id",
            "test-container-id",
            "--name",
            "web",
            "--container-port",
            "80",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

// ---------------------------------------------------------------------------
// Volume — JSON snapshot tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_volume_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/apps/test-app-id/volumes"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/volumes_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "volume",
            "list",
            "--app-id",
            "test-app-id",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_volume_update_json() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/apps/test-app-id/volumes/test-vol-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/volume_update.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "volume",
            "update",
            "--app-id",
            "test-app-id",
            "--volume-id",
            "test-vol-id",
            "--name",
            "updated-vol",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

// ---------------------------------------------------------------------------
// Registry — JSON snapshot tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_registry_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/registries"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/registries_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "container", "registry", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_registry_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/registries/9999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/registry_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "registry",
            "get",
            "--id",
            "9999",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_registry_create_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/registries"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/registry_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "registry",
            "create",
            "--name",
            "hoppy-test-registry",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_registry_image_tags_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/registries/tags"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/container_image_tags.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "registry",
            "image-tags",
            "--registry-id",
            "1155",
            "--image-name",
            "nginx",
            "--image-namespace",
            "library",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_registry_image_digest_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/registries/digest"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/image_digest.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "registry",
            "image-digest",
            "--registry-id",
            "1155",
            "--image-name",
            "nginx",
            "--image-namespace",
            "library",
            "--tag",
            "alpine",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_registry_config_suggestions_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/registries/config-suggestions"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/config_suggestions.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "registry",
            "config-suggestions",
            "--registry-id",
            "1155",
            "--image-name",
            "nginx",
            "--image-namespace",
            "library",
            "--tag",
            "alpine",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_registry_search_public_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/registries/public-images/search"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/public_images_search.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "registry",
            "search-public",
            "--registry-id",
            "1155",
            "--query",
            "nginx",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

// ---------------------------------------------------------------------------
// Region — JSON snapshot tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_region_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/regions"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/regions_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "container", "region", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_region_optimal_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/regions/optimal"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/optimal_region.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "container", "region", "optimal"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

// ---------------------------------------------------------------------------
// Node — JSON snapshot test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_node_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/nodes"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/nodes_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "container", "node", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

// ---------------------------------------------------------------------------
// Limits — JSON snapshot test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_limits_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/limits"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/user_limits.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "container", "limits"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

// ---------------------------------------------------------------------------
// Log Forwarding — JSON snapshot tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_log_forwarding_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/log/forwarding"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/log_forwarding_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "container", "log-forwarding", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_log_forwarding_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/log/forwarding/test-app-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/log_forwarding_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "log-forwarding",
            "get",
            "--app-id",
            "test-app-id",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn container_log_forwarding_create_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/log/forwarding"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/log_forwarding_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "log-forwarding",
            "create",
            "--app-id",
            "test-app-id",
            "--forwarding-type",
            "SyslogTcp",
            "--endpoint",
            "logs.example.com",
            "--port",
            "514",
            "--syslog-format",
            "SyslogRfc5424",
            "--token",
            "tok-e2e",
            "--enabled",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

/// iter-81: `--token` is required on create — the API rejects tokenless
/// configurations with an empty 400 (undocumented upstream requirement, see
/// backlog/log-forwarding-create-empty-400.md). Must be a clap usage error,
/// no HTTP call.
#[tokio::test]
async fn container_log_forwarding_create_requires_token() {
    let output = mock_cmd("test-api-key", "http://127.0.0.1:1")
        .args([
            "container",
            "log-forwarding",
            "create",
            "--app-id",
            "test-app-id",
            "--forwarding-type",
            "SyslogTcp",
            "--endpoint",
            "logs.example.com",
            "--port",
            "514",
            "--syslog-format",
            "SyslogRfc5424",
            "--enabled",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--token"), "got: {stderr}");
}

// ---------------------------------------------------------------------------
// --debug request-body printing (iter-81: mc-debug-omits-request-body)
// ---------------------------------------------------------------------------

/// `--debug` on a mutating containers command must print the serialized
/// request body (`>>>` line), same as the core client, and redact
/// secret-shaped fields (e.g. the log-forwarding `token`) unless `--reveal`
/// is also passed.
#[tokio::test]
async fn container_log_forwarding_create_debug_prints_and_redacts_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/log/forwarding"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/log_forwarding_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--debug",
            "--format",
            "json",
            "container",
            "log-forwarding",
            "create",
            "--app-id",
            "test-app-id",
            "--forwarding-type",
            "SyslogTcp",
            "--endpoint",
            "logs.example.com",
            "--port",
            "514",
            "--syslog-format",
            "SyslogRfc5424",
            "--token",
            "supersecrettoken123",
            "--enabled",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(">>>"),
        "expected a >>> request-body line in --debug output, got: {stderr}"
    );
    assert!(
        stderr.contains("\"token\""),
        "expected the token field name in the printed body, got: {stderr}"
    );
    assert!(
        !stderr.contains("supersecrettoken123"),
        "expected the token value to be redacted by default, got: {stderr}"
    );
    assert!(
        stderr.contains("<set, length=19>"),
        "expected a redacted-length placeholder for the token, got: {stderr}"
    );
}

/// `--debug --reveal` shows the raw (unredacted) request body.
#[tokio::test]
async fn container_log_forwarding_create_debug_reveal_shows_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/log/forwarding"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/log_forwarding_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--debug",
            "--reveal",
            "--format",
            "json",
            "container",
            "log-forwarding",
            "create",
            "--app-id",
            "test-app-id",
            "--forwarding-type",
            "SyslogTcp",
            "--endpoint",
            "logs.example.com",
            "--port",
            "514",
            "--syslog-format",
            "SyslogRfc5424",
            "--token",
            "supersecrettoken123",
            "--enabled",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(">>>"),
        "expected a >>> request-body line in --debug output, got: {stderr}"
    );
    assert!(
        stderr.contains("supersecrettoken123"),
        "expected the raw token value with --reveal, got: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// Application — success-only mutation tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_app_update() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/apps/test-app-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/app_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "app",
            "update",
            "--id",
            "test-app-id",
            "--name",
            "updated-name",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn container_app_deploy() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/apps/test-app-id/deploy"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args(["container", "app", "deploy", "--id", "test-app-id"])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn container_app_undeploy() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/apps/test-app-id/undeploy"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "container",
            "app",
            "undeploy",
            "--id",
            "test-app-id",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn container_app_restart() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/apps/test-app-id/restart"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args(["container", "app", "restart", "--id", "test-app-id"])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn container_app_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/apps/test-app-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args(["--yes", "container", "app", "delete", "--id", "test-app-id"])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn container_app_autoscaling_update() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/apps/test-app-id/autoscaling"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "container",
            "app",
            "autoscaling-update",
            "--app-id",
            "test-app-id",
            "--min",
            "1",
            "--max",
            "3",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn container_app_region_settings_update() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/apps/test-app-id/region-settings"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "container",
            "app",
            "region-settings-update",
            "--app-id",
            "test-app-id",
            "--allowed-region",
            "DE",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

// ---------------------------------------------------------------------------
// Template — success-only mutation tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_template_update() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/apps/test-app-id/containers/test-container-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/container_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "container",
            "template",
            "update",
            "--app-id",
            "test-app-id",
            "--container-id",
            "test-container-id",
            "--name",
            "nginx-updated",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn container_template_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/apps/test-app-id/containers/test-container-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "container",
            "template",
            "delete",
            "--app-id",
            "test-app-id",
            "--container-id",
            "test-container-id",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn container_template_env_replace_all() {
    // --replace-all is the explicit, named version of the historical
    // "replace whole array" behaviour. `--env` is now only valid in this mode.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/apps/test-app-id/containers/test-container-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/container_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/apps/test-app-id/containers/test-container-id/env"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/container_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "container",
            "template",
            "env",
            "--app-id",
            "test-app-id",
            "--container-id",
            "test-container-id",
            "--replace-all",
            "--env",
            "FOO=bar",
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
async fn container_template_env_zero_args_is_refused() {
    // MC.1: bare `template env` (no flags) used to silently wipe all vars.
    // It must now error out with a friendly recipe and never call the API.
    let server = MockServer::start().await;
    // No mocks mounted — any HTTP call would fail the test.

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "container",
            "template",
            "env",
            "--app-id",
            "test-app-id",
            "--container-id",
            "test-container-id",
        ])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "expected non-zero exit for bare env command"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no operation specified"),
        "expected friendly error, got: {stderr}"
    );
    assert!(
        stderr.contains("--add") && stderr.contains("--clear"),
        "expected recipe to mention --add and --clear, got: {stderr}"
    );
}

#[tokio::test]
async fn container_template_env_add_merges_with_existing() {
    // MC.5: `--add KEY=VAL` reads the current set, merges, and writes back —
    // never destructive. We assert against the PUT body to prove the merge.
    use wiremock::matchers::body_json;

    let server = MockServer::start().await;
    let existing = serde_json::json!({
        "id": "test-container-id",
        "name": "nginx",
        "packageId": "p",
        "image": "library/nginx:alpine",
        "imageName": "nginx",
        "imageNamespace": "library",
        "imageTag": "alpine",
        "imageRegistryId": "1155",
        "imageDigest": "",
        "imagePullPolicy": "ifNotPresent",
        "entryPoint": {
            "command": "", "commandArray": [], "arguments": "",
            "argumentsArray": [], "workingDirectory": ""
        },
        "probes": {"startup": null, "readiness": null, "liveness": null},
        "environmentVariables": [
            {"name": "EXISTING", "value": "keep-me"}
        ],
        "endpoints": [],
        "volumeMounts": []
    });

    Mock::given(method("GET"))
        .and(path("/apps/test-app-id/containers/test-container-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(existing.to_string(), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path("/apps/test-app-id/containers/test-container-id/env"))
        .and(body_json(serde_json::json!({
            "EXISTING": "keep-me",
            "NEW_KEY": "new-val"
        })))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(existing.to_string(), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "container",
            "template",
            "env",
            "--app-id",
            "test-app-id",
            "--container-id",
            "test-container-id",
            "--add",
            "NEW_KEY=new-val",
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
async fn container_template_env_remove_drops_key() {
    // MC.5: `--remove KEY` reads, drops, writes back. Removing a missing key
    // is a no-op (still writes the unchanged set, which is acceptable).
    use wiremock::matchers::body_json;

    let server = MockServer::start().await;
    let existing = serde_json::json!({
        "id": "c", "name": "n", "packageId": "p",
        "image": "library/nginx:alpine", "imageName": "nginx",
        "imageNamespace": "library", "imageTag": "alpine",
        "imageRegistryId": "1155", "imageDigest": "",
        "imagePullPolicy": "ifNotPresent",
        "entryPoint": {"command": "", "commandArray": [], "arguments": "",
            "argumentsArray": [], "workingDirectory": ""},
        "probes": {"startup": null, "readiness": null, "liveness": null},
        "environmentVariables": [
            {"name": "KEEP", "value": "1"},
            {"name": "DROP", "value": "2"},
        ],
        "endpoints": [], "volumeMounts": []
    });

    Mock::given(method("GET"))
        .and(path("/apps/test-app-id/containers/test-container-id"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(existing.to_string(), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path("/apps/test-app-id/containers/test-container-id/env"))
        .and(body_json(serde_json::json!({"KEEP": "1"})))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(existing.to_string(), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "container",
            "template",
            "env",
            "--app-id",
            "test-app-id",
            "--container-id",
            "test-container-id",
            "--remove",
            "DROP",
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
async fn container_template_env_list_redacts_by_default() {
    // MC.6: env values are redacted by default. `--list` prints names + masked
    // values; the user must opt in with --reveal to see the raw value.
    let server = MockServer::start().await;
    let body = serde_json::json!({
        "id": "c", "name": "n", "packageId": "p",
        "image": "library/nginx:alpine", "imageName": "nginx",
        "imageNamespace": "library", "imageTag": "alpine",
        "imageRegistryId": "1155", "imageDigest": "",
        "imagePullPolicy": "ifNotPresent",
        "entryPoint": {"command": "", "commandArray": [], "arguments": "",
            "argumentsArray": [], "workingDirectory": ""},
        "probes": {"startup": null, "readiness": null, "liveness": null},
        "environmentVariables": [
            {"name": "API_KEY", "value": "super-secret-value"},
        ],
        "endpoints": [], "volumeMounts": []
    });

    Mock::given(method("GET"))
        .and(path("/apps/a/containers/c"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body.to_string(), "application/json"))
        .expect(2)
        .mount(&server)
        .await;

    // Default: redacted
    let out = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "template",
            "env",
            "--app-id",
            "a",
            "--container-id",
            "c",
            "--list",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("<set, length=18>"),
        "expected redacted placeholder, got: {stdout}"
    );
    assert!(
        !stdout.contains("super-secret-value"),
        "raw value leaked: {stdout}"
    );

    // --reveal: raw value shown
    let out = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "--reveal",
            "container",
            "template",
            "env",
            "--app-id",
            "a",
            "--container-id",
            "c",
            "--list",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("super-secret-value"),
        "expected raw value with --reveal, got: {stdout}"
    );
}

#[tokio::test]
async fn container_app_delete_refuses_with_auto_pull_zone() {
    // MC.3: `app delete` discovers auto-managed PZs and refuses without
    // --cascade or --no-cascade. The orphan PZ ids appear in the error output.
    let server = MockServer::start().await;
    let endpoints = serde_json::json!({
        "items": [{
            "id": "ep-1",
            "displayName": "default",
            "publicHost": "example.b-cdn.net",
            "type": "cdn",
            "isSslEnabled": true,
            "pullZoneId": "999",
            "portMappings": [],
            "containerName": "nginx",
            "containerId": "c-1"
        }]
    });
    Mock::given(method("GET"))
        .and(path("/apps/app-1/endpoints"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(endpoints.to_string(), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    // No DELETE mock — if the code calls it, the test fails with an unexpected request.

    let out = mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes", // even with --yes, missing --cascade/--no-cascade refuses
            "container",
            "app",
            "delete",
            "--id",
            "app-1",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success(), "expected refusal");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("999"),
        "expected orphan PZ id in error, got: {stderr}"
    );
    assert!(
        stderr.contains("--cascade") && stderr.contains("--no-cascade"),
        "expected guidance to flags, got: {stderr}"
    );
}

#[tokio::test]
async fn container_app_delete_no_cascade_lists_orphans() {
    // MC.3: --no-cascade deletes the app but prints orphan IDs + cleanup recipe.
    let server = MockServer::start().await;
    let endpoints = serde_json::json!({
        "items": [{
            "id": "ep-1",
            "displayName": "default",
            "publicHost": "example.b-cdn.net",
            "type": "cdn",
            "isSslEnabled": true,
            "pullZoneId": "12345",
            "portMappings": [],
            "containerName": "nginx",
            "containerId": "c-1"
        }]
    });
    Mock::given(method("GET"))
        .and(path("/apps/app-2/endpoints"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(endpoints.to_string(), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/apps/app-2"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let out = mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "container",
            "app",
            "delete",
            "--id",
            "app-2",
            "--no-cascade",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("12345"),
        "expected orphan id printed, got: {stderr}"
    );
    assert!(
        stderr.contains("hoppy pull-zone delete"),
        "expected cleanup recipe, got: {stderr}"
    );
}

#[tokio::test]
async fn container_app_delete_cascade_deletes_pull_zone() {
    // MC.3: --cascade also deletes the auto-PZ via the core API.
    let server = MockServer::start().await;
    let endpoints = serde_json::json!({
        "items": [{
            "id": "ep-1",
            "displayName": "default",
            "publicHost": "example.b-cdn.net",
            "type": "cdn",
            "isSslEnabled": true,
            "pullZoneId": "777",
            "portMappings": [],
            "containerName": "nginx",
            "containerId": "c-1"
        }]
    });
    Mock::given(method("GET"))
        .and(path("/apps/app-3/endpoints"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(endpoints.to_string(), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/apps/app-3"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/pullzone/777"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    // Note: same mock server hosts both the core and containers APIs in test.
    let mut cmd = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        None,
        Some(&server.uri()),
    );
    cmd.args([
        "--yes",
        "container",
        "app",
        "delete",
        "--id",
        "app-3",
        "--cascade",
    ]);
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// ---------------------------------------------------------------------------
// Endpoint — success-only mutation tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_endpoint_update() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/apps/test-app-id/endpoints/test-endpoint-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "container",
            "endpoint",
            "update",
            "--app-id",
            "test-app-id",
            "--endpoint-id",
            "test-endpoint-id",
            "--name",
            "web-updated",
            "--container-port",
            "80",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn container_endpoint_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/apps/test-app-id/endpoints/test-endpoint-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "container",
            "endpoint",
            "delete",
            "--app-id",
            "test-app-id",
            "--endpoint-id",
            "test-endpoint-id",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

// ---------------------------------------------------------------------------
// Volume — success-only mutation tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_volume_detach() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/apps/test-app-id/volumes/test-vol-id/detach"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/volume_detach.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "container",
            "volume",
            "detach",
            "--app-id",
            "test-app-id",
            "--volume-id",
            "test-vol-id",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn container_volume_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/apps/test-app-id/volumes/test-vol-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/volume_delete_all.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "container",
            "volume",
            "delete",
            "--app-id",
            "test-app-id",
            "--volume-id",
            "test-vol-id",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn container_volume_delete_instance() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(
            "/apps/test-app-id/volumes/test-vol-id/instances/inst-1",
        ))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/volume_delete_instance.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "container",
            "volume",
            "delete-instance",
            "--app-id",
            "test-app-id",
            "--volume-id",
            "test-vol-id",
            "--instance-id",
            "inst-1",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

// ---------------------------------------------------------------------------
// Registry — success-only mutation tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_registry_update() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/registries/9999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/registry_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "container",
            "registry",
            "update",
            "--id",
            "9999",
            "--name",
            "updated-registry",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn container_registry_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/registries/9999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/registry_delete.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args(["--yes", "container", "registry", "delete", "--id", "9999"])
        .output()
        .unwrap();

    assert!(output.status.success());
}

// ---------------------------------------------------------------------------
// Pod — success-only mutation test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_pod_recreate() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/apps/test-app-id/pods/test-pod-id/recreate"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "container",
            "pod",
            "recreate",
            "--app-id",
            "test-app-id",
            "--pod-id",
            "test-pod-id",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

// ---------------------------------------------------------------------------
// Log Forwarding — success-only mutation tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_log_forwarding_update() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/log/forwarding/test-app-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/log_forwarding_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "container",
            "log-forwarding",
            "update",
            "--app-id",
            "test-app-id",
            "--forwarding-type",
            "SyslogTcp",
            "--endpoint",
            "logs.example.com",
            "--port",
            "514",
            "--syslog-format",
            "SyslogRfc5424",
            "--token",
            "tok-e2e",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn container_log_forwarding_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/log/forwarding/test-app-id"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "container",
            "log-forwarding",
            "delete",
            "--app-id",
            "test-app-id",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

// ---------------------------------------------------------------------------
// Live API lifecycle test
// ---------------------------------------------------------------------------

#[cfg(feature = "live-api")]
#[test]
fn live_container_app_lifecycle() {
    support::run_lifecycle(|cleanup| {
        let raw_name = support::unique_name("hpmc");
        let name: String = raw_name.chars().take(30).collect();

        // 1. Create (API requires at least one container template)
        let create = support::hoppy_live_json(&[
            "container",
            "app",
            "create",
            "--name",
            &name,
            "--runtime-type",
            "shared",
            "--min",
            "1",
            "--max",
            "1",
            "--region",
            "DE",
            "--image-name",
            "nginx",
            "--image-namespace",
            "library",
            "--image-tag",
            "alpine",
            "--registry-id",
            "1155",
        ]);
        assert!(create.success, "create failed: {}", create.stderr);
        let id = create.json.as_ref().unwrap()["id"]
            .as_str()
            .expect("id missing from create response")
            .to_owned();

        // Register cleanup early. `--cascade` is required here because step 9
        // below adds a CDN endpoint that provisions an auto-managed pull
        // zone; without `--cascade` (or `--no-cascade`), `handle_app_delete`
        // refuses to delete an app that owns auto-managed pull zones
        // (crates/hoppy-cli/src/commands/container.rs handle_app_delete),
        // CleanupStack::run swallows that failure, and both the app and the
        // pull zone leak. `--cascade` is a no-op when there are zero
        // auto-managed pull zones, so it's safe even before step 9 runs.
        cleanup.push(&["container", "app", "delete", "--id", &id, "--cascade"]);

        // 2. Get by id
        let get = support::hoppy_live_json(&["container", "app", "get", "--id", &id]);
        assert!(get.success, "get failed: {}", get.stderr);

        // 3. List — verify app appears
        let list = support::hoppy_live_json(&["container", "app", "list"]);
        assert!(list.success, "list failed: {}", list.stderr);
        let items = list.json.as_ref().unwrap()["items"]
            .as_array()
            .expect("items missing from list response");
        let found = items.iter().any(|a| a["id"].as_str() == Some(&id));
        assert!(found, "created app {id} not found in list");

        // 4. Overview (may 500 while app is still progressing — non-fatal)
        let overview = support::hoppy_live_json(&["container", "app", "overview", "--id", &id]);
        if !overview.success {
            eprintln!(
                "overview skipped (app still progressing): {}",
                overview.stderr
            );
        }

        // 5. Autoscaling get
        let autoscaling =
            support::hoppy_live_json(&["container", "app", "autoscaling-get", "--app-id", &id]);
        assert!(
            autoscaling.success,
            "autoscaling-get failed: {}",
            autoscaling.stderr
        );

        // 6. Autoscaling update
        let autoscaling_update = support::hoppy_live_raw(&[
            "container",
            "app",
            "autoscaling-update",
            "--app-id",
            &id,
            "--min",
            "1",
            "--max",
            "2",
        ]);
        assert!(
            autoscaling_update.success,
            "autoscaling-update failed: {}",
            autoscaling_update.stderr
        );

        // 7. Region settings get
        let region_settings =
            support::hoppy_live_json(&["container", "app", "region-settings-get", "--app-id", &id]);
        assert!(
            region_settings.success,
            "region-settings-get failed: {}",
            region_settings.stderr
        );

        // 8. Update app name
        let updated_name = format!("{name}-upd");
        let update = support::hoppy_live_json(&[
            "container",
            "app",
            "update",
            "--id",
            &id,
            "--name",
            &updated_name,
        ]);
        assert!(update.success, "update failed: {}", update.stderr);

        // 9. Add a CDN endpoint to exercise port-mapping deserialization
        let container_id = get.json.as_ref().unwrap()["containerTemplates"][0]["id"]
            .as_str()
            .expect("container template id missing")
            .to_owned();
        let ep_add = support::hoppy_live_json(&[
            "container",
            "endpoint",
            "add",
            "--app-id",
            &id,
            "--container-id",
            &container_id,
            "--name",
            "test-ep",
            "--container-port",
            "80",
        ]);
        assert!(ep_add.success, "endpoint add failed: {}", ep_add.stderr);

        // 10. Get app again — response now includes endpoints with exposedPort (may be null)
        let get3 = support::hoppy_live_json(&["container", "app", "get", "--id", &id]);
        assert!(
            get3.success,
            "get after endpoint add failed: {}",
            get3.stderr
        );

        // 11. List endpoints
        let ep_list = support::hoppy_live_json(&["container", "endpoint", "list", "--app-id", &id]);
        assert!(ep_list.success, "endpoint list failed: {}", ep_list.stderr);

        // 12. Delete is handled by cleanup
    });
}

// ---------------------------------------------------------------------------
// container logs — help snapshot
// ---------------------------------------------------------------------------

#[test]
fn container_logs_help() {
    let output = support::hoppy_cmd()
        .args(["container", "logs", "--help"])
        .output()
        .unwrap();
    // --help exits with 0
    assert!(output.status.success(), "status: {}", output.status);
    crate::assert_cli_snapshot!(String::from_utf8_lossy(&output.stdout));
}

// ---------------------------------------------------------------------------
// iter-62: negative-int flag parsing hint
// ---------------------------------------------------------------------------

/// `--min -1` previously surfaced clap's confusing "unexpected argument '-1'"
/// error. Iter-62 rewrites that into a friendly hint that points at the
/// preceding `--min` and the `--min=-1` workaround.
#[test]
fn container_app_create_negative_min_emits_hint() {
    let output = support::hoppy_cmd()
        .args([
            "container",
            "app",
            "create",
            "--name",
            "probe",
            "--runtime-type",
            "Shared",
            "--min",
            "-1",
            "--max",
            "1",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success(), "expected non-zero exit");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("looks like a negative number"),
        "stderr missing hint: {stderr}"
    );
    assert!(stderr.contains("--min"), "stderr missing flag: {stderr}");
    assert!(
        stderr.contains("--min=-1"),
        "stderr missing workaround: {stderr}"
    );
    assert!(
        !stderr.contains("unexpected argument"),
        "stderr still surfaces clap's confusing default error: {stderr}"
    );
}

/// The `=` form sidesteps clap's short-flag parsing. The user can keep using
/// it as a workaround; this test pins that escape hatch.
#[test]
fn container_app_create_eq_form_passes_negative_value() {
    // We don't run the full create flow (that would need a mock server); we
    // just confirm clap accepts the value and the failure now happens later,
    // at the API-credentials or domain-validation layer — never at parse time.
    let output = support::hoppy_cmd()
        .args([
            "container",
            "app",
            "create",
            "--name",
            "probe",
            "--runtime-type",
            "Shared",
            "--min=-1",
            "--max",
            "1",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "--min=-1 should bypass clap's short-flag parsing: {stderr}"
    );
    assert!(
        !stderr.contains("looks like a negative number"),
        "should not trigger negative-value hint for --min=-1: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// iter-76: containers polish — volumes, runtime config, endpoint options,
// registry images, and the schema-less summary / node-ips / image-config
// passthrough commands.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn container_app_create_with_volumes_sends_array() {
    // --volume NAME:SIZE_GB must land in the POST /apps `volumes` array so the
    // volume is actually created (previously hard-coded to None).
    use wiremock::matchers::body_partial_json;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/apps"))
        .and(body_partial_json(serde_json::json!({
            "volumes": [
                {"name": "data", "size": 10},
                {"name": "cache", "size": 5}
            ]
        })))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/app_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "app",
            "create",
            "--minimal",
            "--name",
            "vol-app",
            "--runtime-type",
            "shared",
            "--min",
            "1",
            "--max",
            "1",
            "--region",
            "DE",
            "--volume",
            "data:10",
            "--volume",
            "cache:5",
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
async fn container_app_update_with_volumes_sends_array() {
    use wiremock::matchers::body_partial_json;

    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/apps/test-app-id"))
        .and(body_partial_json(serde_json::json!({
            "volumes": [{"name": "logs", "size": 3}]
        })))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/app_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "app",
            "update",
            "--id",
            "test-app-id",
            "--volume",
            "logs:3",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn container_app_create_bad_volume_spec_errors() {
    let output = support::hoppy_cmd()
        .env("BUNNY_API_KEY", "dummy")
        .args([
            "container",
            "app",
            "create",
            "--name",
            "x",
            "--runtime-type",
            "shared",
            "--min",
            "1",
            "--max",
            "1",
            "--region",
            "DE",
            "--volume",
            "data-no-colon",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("NAME:SIZE_GB"),
        "expected volume-format hint, got: {stderr}"
    );
}

#[tokio::test]
async fn container_app_summary_passthrough() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"usage": {"cpu": 12, "memory": 256}});
    Mock::given(method("GET"))
        .and(path("/apps/test-app-id/summary"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body.to_string(), "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args(["container", "app", "summary", "--id", "test-app-id"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"cpu\": 12"), "got: {stdout}");
}

#[tokio::test]
async fn container_template_add_with_runtime_config() {
    // --pull-policy, --image-digest, entrypoint flags, and --volume-mount must
    // all land in the POST /apps/{id}/containers body.
    use wiremock::matchers::body_partial_json;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/apps/test-app-id/containers"))
        .and(body_partial_json(serde_json::json!({
            "imageDigest": "sha256:abc",
            "imagePullPolicy": "always",
            "entryPoint": {
                "commandArray": ["/bin/sh", "-c"],
                "arguments": "echo hi",
                "workingDirectory": "/app"
            },
            "volumeMounts": [{"name": "data", "mountPath": "/var/lib/data"}]
        })))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/container_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "template",
            "add",
            "--app-id",
            "test-app-id",
            "--name",
            "nginx",
            "--image-name",
            "nginx",
            "--image-namespace",
            "library",
            "--image-tag",
            "alpine",
            "--registry-id",
            "1155",
            "--image-digest",
            "sha256:abc",
            "--pull-policy",
            "Always",
            "--command-array",
            "/bin/sh",
            "--command-array=-c",
            "--arguments",
            "echo hi",
            "--working-directory",
            "/app",
            "--volume-mount",
            "data:/var/lib/data",
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
async fn container_template_add_with_probes_json_file() {
    use wiremock::matchers::body_partial_json;

    let dir = tempfile::tempdir().unwrap();
    let probes_path = dir.path().join("probes.json");
    std::fs::write(
        &probes_path,
        r#"{"liveness":{"periodSeconds":10,"httpGet":{"request":{"path":"/health","portNumber":80}}}}"#,
    )
    .unwrap();

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/apps/test-app-id/containers"))
        .and(body_partial_json(serde_json::json!({
            "probes": {
                "liveness": {
                    "periodSeconds": 10,
                    "httpGet": {"request": {"path": "/health", "portNumber": 80}}
                }
            }
        })))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/container_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "template",
            "add",
            "--app-id",
            "test-app-id",
            "--name",
            "nginx",
            "--image-name",
            "nginx",
            "--image-namespace",
            "library",
            "--image-tag",
            "alpine",
            "--registry-id",
            "1155",
            "--probes-json",
            probes_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn container_template_add_bad_pull_policy_errors() {
    let output = support::hoppy_cmd()
        .env("BUNNY_API_KEY", "dummy")
        .args([
            "container",
            "template",
            "add",
            "--app-id",
            "a",
            "--name",
            "n",
            "--image-name",
            "nginx",
            "--image-namespace",
            "library",
            "--image-tag",
            "alpine",
            "--registry-id",
            "1155",
            "--pull-policy",
            "Sometimes",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Always or IfNotPresent"), "got: {stderr}");
}

#[tokio::test]
async fn container_template_update_with_runtime_config() {
    use wiremock::matchers::body_partial_json;

    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/apps/test-app-id/containers/test-container-id"))
        .and(body_partial_json(serde_json::json!({
            "imagePullPolicy": "ifNotPresent",
            "entryPoint": {"command": "/start.sh"}
        })))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/container_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "template",
            "update",
            "--app-id",
            "test-app-id",
            "--container-id",
            "test-container-id",
            "--pull-policy",
            "IfNotPresent",
            "--command",
            "/start.sh",
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
async fn container_endpoint_add_cdn_with_ssl_and_sticky() {
    use wiremock::matchers::body_partial_json;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/apps/test-app-id/containers/test-container-id/endpoints",
        ))
        .and(body_partial_json(serde_json::json!({
            "displayName": "web",
            "cdn": {
                "isSslEnabled": true,
                "pullZoneId": 42,
                "stickySessions": {
                    "enabled": true,
                    "sessionHeaders": ["X-User"],
                    "cookieName": "sid"
                },
                "portMappings": [{"containerPort": 8080, "exposedPort": 443}]
            }
        })))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/endpoint_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "endpoint",
            "add",
            "--app-id",
            "test-app-id",
            "--container-id",
            "test-container-id",
            "--name",
            "web",
            "--cdn",
            "--container-port",
            "8080",
            "--exposed-port",
            "443",
            "--ssl",
            "--pull-zone-id",
            "42",
            "--sticky",
            "--sticky-header",
            "X-User",
            "--sticky-cookie",
            "sid",
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
async fn container_endpoint_add_anycast_multiple_port_mappings() {
    use wiremock::matchers::body_partial_json;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/apps/test-app-id/containers/test-container-id/endpoints",
        ))
        .and(body_partial_json(serde_json::json!({
            "anycast": {
                "type": "IPv4",
                "portMappings": [
                    {"containerPort": 80, "exposedPort": 80, "protocols": ["Tcp"]},
                    {"containerPort": 53, "protocols": ["Udp"]}
                ]
            }
        })))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/endpoint_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "endpoint",
            "add",
            "--app-id",
            "test-app-id",
            "--container-id",
            "test-container-id",
            "--name",
            "any",
            "--anycast",
            "--port-mapping",
            "80:80:Tcp",
            "--port-mapping",
            "53::Udp",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn container_endpoint_add_cdn_rejects_multiple_mappings() {
    let output = support::hoppy_cmd()
        .env("BUNNY_API_KEY", "dummy")
        .args([
            "container",
            "endpoint",
            "add",
            "--app-id",
            "a",
            "--container-id",
            "c",
            "--name",
            "web",
            "--cdn",
            "--port-mapping",
            "80",
            "--port-mapping",
            "443",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("exactly one port mapping"), "got: {stderr}");
}

#[test]
fn container_endpoint_add_anycast_rejects_cdn_options() {
    let output = support::hoppy_cmd()
        .env("BUNNY_API_KEY", "dummy")
        .args([
            "container",
            "endpoint",
            "add",
            "--app-id",
            "a",
            "--container-id",
            "c",
            "--name",
            "any",
            "--anycast",
            "--container-port",
            "80",
            "--ssl",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("CDN-only"), "got: {stderr}");
}

#[test]
fn container_endpoint_add_sticky_without_header_errors() {
    let output = support::hoppy_cmd()
        .env("BUNNY_API_KEY", "dummy")
        .args([
            "container",
            "endpoint",
            "add",
            "--app-id",
            "a",
            "--container-id",
            "c",
            "--name",
            "web",
            "--container-port",
            "80",
            "--sticky",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--sticky-header"), "got: {stderr}");
}

#[tokio::test]
async fn container_registry_images_json() {
    use wiremock::matchers::body_partial_json;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/registries/images"))
        .and(body_partial_json(serde_json::json!({"registryId": "1155"})))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("containers/container_images_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "container",
            "registry",
            "images",
            "--registry-id",
            "1155",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("nginx"), "got: {stdout}");
}

#[tokio::test]
async fn container_registry_image_config_passthrough() {
    let server = MockServer::start().await;
    let body =
        serde_json::json!({"exposedPorts": ["80/tcp"], "entrypoint": ["/docker-entrypoint.sh"]});
    Mock::given(method("POST"))
        .and(path("/registries/image-config"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body.to_string(), "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args([
            "container",
            "registry",
            "image-config",
            "--registry-id",
            "1155",
            "--image-name",
            "nginx",
            "--image-namespace",
            "library",
            "--tag",
            "alpine",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("exposedPorts"), "got: {stdout}");
}

#[tokio::test]
async fn container_node_ips_passthrough() {
    let server = MockServer::start().await;
    let body = serde_json::json!(["203.0.113.1", "203.0.113.2"]);
    Mock::given(method("GET"))
        .and(path("/nodes/plain"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body.to_string(), "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let output = mock_cmd("test-api-key", &server.uri())
        .args(["container", "node", "ips"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("203.0.113.1"), "got: {stdout}");
}
