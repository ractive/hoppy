use bunny_api_compute::ComputeClient;
use bunny_api_compute::{
    AddSecret, AddVariable, CreateEdgeScript, PublishScript, ScriptType, UpdateEdgeScript,
    UpdateSecret, UpdateVariable, UpsertSecret, UpsertVariable,
};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_SCRIPTS_LIST: &str = include_str!("fixtures/scripts_list.json");
const FIXTURE_SCRIPT_GET: &str = include_str!("fixtures/script_get.json");
const FIXTURE_SCRIPT_CREATE: &str = include_str!("fixtures/script_create.json");
const FIXTURE_SCRIPT_UPDATE: &str = include_str!("fixtures/script_update.json");
const FIXTURE_SCRIPT_CODE_GET: &str = include_str!("fixtures/script_code_get.json");
const FIXTURE_RELEASES_LIST: &str = include_str!("fixtures/releases_list.json");
const FIXTURE_RELEASE_ACTIVE: &str = include_str!("fixtures/release_active.json");
const FIXTURE_VARIABLE_ADD: &str = include_str!("fixtures/variable_add.json");
const FIXTURE_VARIABLE_GET: &str = include_str!("fixtures/variable_get.json");
const FIXTURE_VARIABLE_UPDATE: &str = include_str!("fixtures/variable_update.json");
const FIXTURE_SECRETS_LIST: &str = include_str!("fixtures/secrets_list.json");
const FIXTURE_SECRET_ADD: &str = include_str!("fixtures/secret_add.json");
const FIXTURE_STATISTICS: &str = include_str!("fixtures/statistics.json");
const FIXTURE_ERROR_UNAUTHORIZED: &str = include_str!("fixtures/error_unauthorized.json");
const FIXTURE_ERROR_NOT_FOUND: &str = include_str!("fixtures/error_not_found.json");

fn test_client(uri: &str) -> ComputeClient {
    ComputeClient::with_base_url("test-api-key", uri)
}

// ---------------------------------------------------------------------------
// Scripts
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_scripts_returns_paginated() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCRIPTS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_scripts(None, None, None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
    assert_eq!(result.current_page, 1);
    assert_eq!(result.total_items, 2);
    assert!(!result.has_more_items);
    assert_eq!(result.items[0].id, 1001);
    assert_eq!(result.items[0].name.as_deref(), Some("my-cdn-script"));
    assert_eq!(result.items[1].id, 1002);
}

#[tokio::test]
async fn list_scripts_with_search() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .and(query_param("search", "cdn"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCRIPTS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_scripts(None, None, Some("cdn"))
        .await
        .unwrap();

    assert!(!result.items.is_empty());
}

#[tokio::test]
async fn get_script_returns_details() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCRIPT_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let script = test_client(&server.uri()).get_script(1001).await.unwrap();

    assert_eq!(script.id, 1001);
    assert_eq!(script.name.as_deref(), Some("my-cdn-script"));
    assert_eq!(script.script_type, ScriptType::Cdn);
    assert_eq!(script.current_release_id, 501);
    assert!(script.edge_script_variables.is_some());
    let vars = script.edge_script_variables.unwrap();
    assert_eq!(vars.len(), 1);
    assert_eq!(vars[0].name.as_deref(), Some("API_URL"));
}

#[tokio::test]
async fn create_script_sends_correct_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/compute/script"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "Name": "new-script",
            "Code": null,
            "ScriptType": 1,
            "CreateLinkedPullZone": false,
            "LinkedPullZoneName": null,
            "Integration": null
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCRIPT_CREATE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = CreateEdgeScript {
        name: Some("new-script".to_owned()),
        code: None,
        script_type: ScriptType::Cdn,
        create_linked_pull_zone: false,
        linked_pull_zone_name: None,
        integration: None,
    };

    let script = test_client(&server.uri())
        .create_script(&body)
        .await
        .unwrap();

    assert_eq!(script.id, 1003);
    assert_eq!(script.name.as_deref(), Some("new-script"));
    assert_eq!(script.script_type, ScriptType::Cdn);
}

#[tokio::test]
async fn update_script_sends_correct_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/compute/script/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "Name": "updated-cdn-script",
            "ScriptType": null
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCRIPT_UPDATE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateEdgeScript {
        name: Some("updated-cdn-script".to_owned()),
        script_type: None,
    };

    let script = test_client(&server.uri())
        .update_script(1001, &body)
        .await
        .unwrap();

    assert_eq!(script.id, 1001);
    assert_eq!(script.name.as_deref(), Some("updated-cdn-script"));
}

#[tokio::test]
async fn delete_script_sends_delete_request() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/compute/script/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_script(1001, false)
        .await
        .unwrap();
}

#[tokio::test]
async fn delete_script_with_linked_pull_zones() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/compute/script/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("deleteLinkedPullZones", "true"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_script(1001, true)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Code
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_script_code_returns_code() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/1001/code"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCRIPT_CODE_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let code = test_client(&server.uri())
        .get_script_code(1001)
        .await
        .unwrap();

    assert!(code.code.is_some());
    assert!(code.code.unwrap().contains("Hello from edge"));
    assert_eq!(code.last_modified, "2024-01-20T08:00:00Z");
}

#[tokio::test]
async fn update_script_code_sends_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/compute/script/1001/code"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "Code": "export default { async fetch(req) { return new Response('ok'); } }"
        })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .update_script_code(
            1001,
            "export default { async fetch(req) { return new Response('ok'); } }",
        )
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Releases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_releases_paginated() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/1001/releases"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_RELEASES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_releases(1001, None, None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
    assert_eq!(result.items[0].id, 501);
    assert_eq!(result.items[0].note.as_deref(), Some("Initial release"));
    assert_eq!(result.items[1].id, 502);
}

#[tokio::test]
async fn get_active_release() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/1001/releases/active"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_RELEASE_ACTIVE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let release = test_client(&server.uri())
        .get_active_release(1001)
        .await
        .unwrap();

    assert_eq!(release.id, 501);
    assert_eq!(
        release.uuid.as_deref(),
        Some("abc123de-f456-7890-abcd-ef1234567890")
    );
    assert_eq!(release.note.as_deref(), Some("Initial release"));
}

#[tokio::test]
async fn publish_script_sends_note() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/compute/script/1001/publish"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "Note": "v2.0 release"
        })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let body = PublishScript {
        note: Some("v2.0 release".to_owned()),
    };

    test_client(&server.uri())
        .publish_script(1001, &body)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Variables
// ---------------------------------------------------------------------------

#[tokio::test]
async fn add_variable_returns_variable() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/compute/script/1001/variables/add"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VARIABLE_ADD, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = AddVariable {
        name: "NEW_VAR".to_owned(),
        required: true,
        default_value: Some("default-value".to_owned()),
    };

    let var = test_client(&server.uri())
        .add_variable(1001, &body)
        .await
        .unwrap();

    assert_eq!(var.id, 301);
    assert_eq!(var.name.as_deref(), Some("NEW_VAR"));
    assert!(var.required);
    assert_eq!(var.default_value.as_deref(), Some("default-value"));
}

#[tokio::test]
async fn get_variable_returns_variable() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/1001/variables/201"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VARIABLE_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let var = test_client(&server.uri())
        .get_variable(1001, 201)
        .await
        .unwrap();

    assert_eq!(var.id, 201);
    assert_eq!(var.name.as_deref(), Some("API_URL"));
    assert!(var.required);
    assert_eq!(
        var.default_value.as_deref(),
        Some("https://api.example.com")
    );
}

#[tokio::test]
async fn update_variable_sends_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/compute/script/1001/variables/201"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "DefaultValue": "https://updated-api.example.com",
            "Required": false
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VARIABLE_UPDATE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateVariable {
        required: Some(false),
        default_value: Some("https://updated-api.example.com".to_owned()),
    };

    let var = test_client(&server.uri())
        .update_variable(1001, 201, &body)
        .await
        .unwrap();

    assert_eq!(var.id, 201);
    assert!(!var.required);
    assert_eq!(
        var.default_value.as_deref(),
        Some("https://updated-api.example.com")
    );
}

#[tokio::test]
async fn delete_variable_sends_request() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/compute/script/1001/variables/201"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_variable(1001, 201)
        .await
        .unwrap();
}

#[tokio::test]
async fn upsert_variable_creates_new() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/compute/script/1001/variables"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VARIABLE_ADD, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = UpsertVariable {
        name: "NEW_VAR".to_owned(),
        required: Some(true),
        default_value: Some("default-value".to_owned()),
    };

    let var = test_client(&server.uri())
        .upsert_variable(1001, &body)
        .await
        .unwrap();

    assert_eq!(var.name.as_deref(), Some("NEW_VAR"));
}

// ---------------------------------------------------------------------------
// Secrets
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_secrets_returns_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/1001/secrets"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SECRETS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri()).list_secrets(1001).await.unwrap();

    let secrets = result.secrets.unwrap();
    assert_eq!(secrets.len(), 2);
    assert_eq!(secrets[0].id, 401);
    assert_eq!(secrets[0].name.as_deref(), Some("API_SECRET_KEY"));
    assert_eq!(secrets[1].id, 402);
    assert_eq!(secrets[1].name.as_deref(), Some("DATABASE_PASSWORD"));
}

#[tokio::test]
async fn add_secret_returns_secret() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/compute/script/1001/secrets"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SECRET_ADD, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = AddSecret {
        name: "NEW_SECRET".to_owned(),
        secret: Some("super-secret-value".to_owned()),
    };

    let secret = test_client(&server.uri())
        .add_secret(1001, &body)
        .await
        .unwrap();

    assert_eq!(secret.id, 403);
    assert_eq!(secret.name.as_deref(), Some("NEW_SECRET"));
}

#[tokio::test]
async fn upsert_secret_with_204_returns_placeholder() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/compute/script/1001/secrets"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpsertSecret {
        name: Some("MY_SECRET".to_owned()),
        secret: Some("secret-val".to_owned()),
    };
    let secret = test_client(&server.uri())
        .upsert_secret(1001, &body)
        .await
        .unwrap();
    assert_eq!(secret.id, 0); // placeholder
    assert_eq!(secret.name.as_deref(), Some("MY_SECRET"));
}

#[tokio::test]
async fn upsert_secret_with_200_returns_body() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/compute/script/1001/secrets"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SECRET_ADD, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = UpsertSecret {
        name: Some("NEW_SECRET".to_owned()),
        secret: Some("secret-val".to_owned()),
    };
    let secret = test_client(&server.uri())
        .upsert_secret(1001, &body)
        .await
        .unwrap();
    assert_eq!(secret.id, 403);
    assert_eq!(secret.name.as_deref(), Some("NEW_SECRET"));
}

#[tokio::test]
async fn update_secret_sends_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/compute/script/1001/secrets/401"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "Secret": "new-secret-value"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SECRET_ADD, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateSecret {
        secret: Some("new-secret-value".to_owned()),
    };

    let secret = test_client(&server.uri())
        .update_secret(1001, 401, &body)
        .await
        .unwrap();

    assert_eq!(secret.name.as_deref(), Some("NEW_SECRET"));
}

#[tokio::test]
async fn delete_secret_sends_request() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/compute/script/1001/secrets/401"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_secret(1001, 401)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Statistics
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_statistics_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/1001/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_STATISTICS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stats = test_client(&server.uri())
        .get_script_statistics(1001, None, None, false)
        .await
        .unwrap();

    assert_eq!(stats.total_requests_served, 125000);
    assert!((stats.total_cpu_used - 45.75).abs() < 0.001);
    assert!((stats.total_monthly_cost - 0.0125).abs() < 0.0001);
    assert!(stats.requests_served_chart.is_some());
}

#[tokio::test]
async fn get_statistics_with_date_range() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/1001/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("dateFrom", "2024-01-01"))
        .and(query_param("dateTo", "2024-01-31"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_STATISTICS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stats = test_client(&server.uri())
        .get_script_statistics(1001, Some("2024-01-01"), Some("2024-01-31"), false)
        .await
        .unwrap();

    assert_eq!(stats.total_requests_served, 125000);
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unauthorized_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script"))
        .respond_with(
            ResponseTemplate::new(401).set_body_raw(FIXTURE_ERROR_UNAUTHORIZED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .list_scripts(None, None, None)
        .await
        .unwrap_err();

    assert!(err.to_string().contains("Invalid API key"));
}

#[tokio::test]
async fn not_found_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/9999"))
        .respond_with(
            ResponseTemplate::new(404).set_body_raw(FIXTURE_ERROR_NOT_FOUND, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .get_script(9999)
        .await
        .unwrap_err();

    assert!(err.to_string().contains("Script not found"));
}

// ---------------------------------------------------------------------------
// Debug mode
// ---------------------------------------------------------------------------

#[tokio::test]
async fn debug_mode_makes_request() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCRIPTS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    // With debug enabled the request should still succeed
    let result = ComputeClient::with_base_url("test-api-key", server.uri())
        .with_debug(true)
        .list_scripts(None, None, None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
}
