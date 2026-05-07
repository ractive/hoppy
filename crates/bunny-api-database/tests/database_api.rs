use bunny_api_database::DatabaseClient;
use bunny_api_database::types::{
    Authorization, CreateDatabaseGroupPayload, CreateDatabasePayload, GenerateTokenDatabasePayload,
    LiveStatus,
};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_DB_GET: &str = include_str!("../../../fixtures/database/database_get.json");
const FIXTURE_DB_LIST: &str = include_str!("../../../fixtures/database/database_list.json");
const FIXTURE_DB_CREATE: &str = include_str!("../../../fixtures/database/database_create.json");
const FIXTURE_DB_DELETE: &str = include_str!("../../../fixtures/database/database_delete.json");
const FIXTURE_GROUP_GET: &str = include_str!("../../../fixtures/database/group_get.json");
const FIXTURE_GROUP_LIST: &str = include_str!("../../../fixtures/database/group_list.json");
const FIXTURE_GROUP_CREATE: &str = include_str!("../../../fixtures/database/group_create.json");
const FIXTURE_TOKEN_MINT: &str = include_str!("../../../fixtures/database/token_mint.json");
const FIXTURE_CONFIG: &str = include_str!("../../../fixtures/database/config.json");
const FIXTURE_CONFIG_LIMITS: &str = include_str!("../../../fixtures/database/config_limits.json");
const FIXTURE_ACTIVE_USAGE: &str = include_str!("../../../fixtures/database/active_usage.json");
const FIXTURE_LIST_VERSIONS: &str = include_str!("../../../fixtures/database/list_versions.json");
const FIXTURE_USAGE: &str = include_str!("../../../fixtures/database/usage.json");
const FIXTURE_LIVE_DB: &str = include_str!("../../../fixtures/database/live_db.json");
const FIXTURE_PING_OK: &str = include_str!("../../../fixtures/database/ping_ok.json");

fn test_client(uri: &str) -> DatabaseClient {
    DatabaseClient::new("db-test-key").with_base_url(uri)
}

// ---------------------------------------------------------------------------
// Database v1
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_databases_returns_envelope() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/databases"))
        .and(header("AccessKey", "db-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_DB_LIST, "application/json"))
        .expect(1)
        .mount(&server)
        .await;
    let resp = test_client(&server.uri())
        .list_databases(None)
        .await
        .unwrap();
    assert_eq!(resp.databases.len(), 2);
    assert_eq!(resp.databases[0].id, "db_01HX0000000000000000000001");
}

#[tokio::test]
async fn list_databases_forwards_group_filter() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/databases"))
        .and(query_param("group_id", "group_01"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_DB_LIST, "application/json"))
        .expect(1)
        .mount(&server)
        .await;
    test_client(&server.uri())
        .list_databases(Some("group_01"))
        .await
        .unwrap();
}

#[tokio::test]
async fn get_database_returns_envelope() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/databases/db_01HX0000000000000000000001"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_DB_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;
    let resp = test_client(&server.uri())
        .get_database("db_01HX0000000000000000000001")
        .await
        .unwrap();
    assert_eq!(resp.database.name, "my-app");
    assert!(resp.database.url.ends_with('/'));
}

#[tokio::test]
async fn create_database_posts_payload() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/databases"))
        .and(body_json(
            serde_json::json!({"slug": "my-app", "group": "group_x"}),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_DB_CREATE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let resp = test_client(&server.uri())
        .create_database(&CreateDatabasePayload::new("my-app", "group_x"))
        .await
        .unwrap();
    assert_eq!(resp.database.name, "my-app");
}

#[tokio::test]
async fn delete_database_returns_name() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/v1/databases/db_01"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_DB_DELETE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let resp = test_client(&server.uri())
        .delete_database("db_01")
        .await
        .unwrap();
    assert_eq!(resp.database, "my-app");
}

#[tokio::test]
async fn list_versions_posts_filters() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/databases/db_01/list_versions"))
        .and(body_json(serde_json::json!({"limit": 10})))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_VERSIONS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let body = bunny_api_database::types::ListVersionsDatabaseGroupPayload {
        limit: Some(10),
        older_than: None,
        newer_than: None,
    };
    let resp = test_client(&server.uri())
        .list_database_versions("db_01", &body)
        .await
        .unwrap();
    assert_eq!(resp.count, 2);
    assert_eq!(resp.generations.len(), 2);
}

// ---------------------------------------------------------------------------
// Token
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mint_token_serialises_authorization_kebab() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/databases/db_01/auth/tokens"))
        .and(body_json(
            serde_json::json!({"authorization": "full-access"}),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_TOKEN_MINT, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let resp = test_client(&server.uri())
        .mint_database_token(
            "db_01",
            &GenerateTokenDatabasePayload::new(Authorization::FullAccess),
        )
        .await
        .unwrap();
    assert!(!resp.token.is_empty());
}

#[tokio::test]
async fn invalidate_keys_204() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/databases/db_01/auth/invalidate"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    test_client(&server.uri())
        .invalidate_database_keys("db_01")
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Group
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_groups_filters_search() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/groups"))
        .and(query_param("search", "EU"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_GROUP_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let resp = test_client(&server.uri())
        .list_groups(Some("EU"))
        .await
        .unwrap();
    assert_eq!(resp.groups.len(), 2);
}

#[tokio::test]
async fn get_group_returns_envelope() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/groups/group_01"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_GROUP_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let resp = test_client(&server.uri())
        .get_group("group_01")
        .await
        .unwrap();
    assert_eq!(resp.group.name, "EU");
    assert_eq!(resp.group.primary_regions, vec!["DE", "FR"]);
}

#[tokio::test]
async fn create_group_posts_payload() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/groups"))
        .and(body_json(serde_json::json!({
            "display_name": "EU",
            "storage_region": "eu-west-1",
            "primary_regions": ["DE"],
            "replicas_regions": []
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_GROUP_CREATE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let body = CreateDatabaseGroupPayload::new("EU", "eu-west-1", vec!["DE".into()], vec![]);
    let resp = test_client(&server.uri())
        .create_group(&body)
        .await
        .unwrap();
    assert_eq!(resp.group.name, "EU");
}

#[tokio::test]
async fn invalidate_group_keys_204() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/groups/group_01/auth/invalidate"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    test_client(&server.uri())
        .invalidate_group_keys("group_01")
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_config_lists_regions() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/config"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_CONFIG, "application/json"))
        .expect(1)
        .mount(&server)
        .await;
    let resp = test_client(&server.uri()).get_config().await.unwrap();
    assert_eq!(resp.storage_region_available.len(), 2);
    assert_eq!(resp.primary_regions[0].id, "DE");
}

#[tokio::test]
async fn get_config_limits_returns_counts() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/config/limits"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_CONFIG_LIMITS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let resp = test_client(&server.uri())
        .get_config_limits()
        .await
        .unwrap();
    assert_eq!(resp.current_databases, 3);
    assert_eq!(resp.max_databases, 50);
}

// ---------------------------------------------------------------------------
// v2
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_active_usage_v2() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v2/databases/active_usage"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ACTIVE_USAGE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let resp = test_client(&server.uri())
        .get_active_usage_v2()
        .await
        .unwrap();
    assert_eq!(resp.active_db, 2);
    assert_eq!(resp.total_db, 3);
}

#[tokio::test]
async fn get_database_usage_v2_query_params() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v2/databases/db_01/usage"))
        .and(query_param("from", "2026-05-01T00:00:00Z"))
        .and(query_param("to", "2026-05-07T00:00:00Z"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_USAGE, "application/json"))
        .expect(1)
        .mount(&server)
        .await;
    let resp = test_client(&server.uri())
        .get_database_usage_v2("db_01", "2026-05-01T00:00:00Z", "2026-05-07T00:00:00Z")
        .await
        .unwrap();
    assert_eq!(resp.rows_read, 12345);
}

// ---------------------------------------------------------------------------
// Live metrics — header propagation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn live_metrics_db_sets_header() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/live/live_db"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_LIVE_DB, "application/json"))
        .expect(1)
        .mount(&server)
        .await;
    let resp = test_client(&server.uri())
        .live_metrics_db(&["db_a".into(), "db_b".into()])
        .await
        .unwrap();
    let received = &server.received_requests().await.unwrap()[0];
    let header_value = received
        .headers
        .get("db-ids")
        .map(|v| v.to_str().unwrap().to_owned());
    assert_eq!(header_value, Some("db_a,db_b".to_owned()));
    assert_eq!(resp.live_metrics.len(), 2);
    let live = resp
        .live_metrics
        .get("db_01HX0000000000000000000001")
        .unwrap();
    match live {
        LiveStatus::Live { metadata } => assert_eq!(metadata.main, "DE"),
        _ => panic!("expected Live"),
    }
}

#[tokio::test]
async fn live_metrics_group_sets_header() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/live/live_group"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_LIVE_DB, "application/json"))
        .expect(1)
        .mount(&server)
        .await;
    test_client(&server.uri())
        .live_metrics_group(&["group_a".into()])
        .await
        .unwrap();
    let received = &server.received_requests().await.unwrap()[0];
    let value = received
        .headers
        .get("group-ids")
        .map(|v| v.to_str().unwrap().to_owned());
    assert_eq!(value, Some("group_a".to_owned()));
}

// ---------------------------------------------------------------------------
// Auth/error paths
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unauthorized_surfaces_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/databases/db_01"))
        .respond_with(ResponseTemplate::new(401).set_body_string(r#"{"error":"unauthorized"}"#))
        .mount(&server)
        .await;
    let err = test_client(&server.uri())
        .get_database("db_01")
        .await
        .unwrap_err();
    let msg = format!("{err:#}");
    assert!(msg.contains("401"));
    assert!(msg.contains("unauthorized"));
}

// ---------------------------------------------------------------------------
// Ping (data plane) — pointed at the same wiremock with http://
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ping_returns_ok_on_2xx() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/pipeline"))
        .and(header("Authorization", "Bearer jwt-here"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_PING_OK, "application/json"))
        .expect(1)
        .mount(&server)
        .await;
    let url = format!("{}/", server.uri());
    let result = test_client(&server.uri()).ping(&url, "jwt-here").await;
    assert!(result.ok, "expected ok=true, got {result:?}");
    assert!(result.error.is_none());
}

#[tokio::test]
async fn ping_returns_error_on_401() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/pipeline"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;
    let url = format!("{}/", server.uri());
    let result = test_client(&server.uri()).ping(&url, "bad-token").await;
    assert!(!result.ok);
    assert!(result.error.unwrap().contains("401"));
}
