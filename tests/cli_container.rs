mod support;

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

    // Overview response contains HashMap-backed charts whose key ordering is
    // non-deterministic after deserialization, so we only verify success.
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

    // Statistics response contains HashMap-backed charts whose key ordering is
    // non-deterministic after deserialization, so we only verify success.
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
            "--enabled",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
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

        // Register cleanup early
        cleanup.push(&["container", "app", "delete", "--id", &id]);

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
