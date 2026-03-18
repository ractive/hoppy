use bunny_api_core::{ApiError, CoreClient, CreateStorageZone, UpdateStorageZone};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_LIST_PAGINATED: &str = include_str!("fixtures/storagezone_list_paginated.json");
const FIXTURE_GET: &str = include_str!("fixtures/storagezone_get.json");
const FIXTURE_CREATE: &str = include_str!("fixtures/storagezone_create.json");
const FIXTURE_NOT_FOUND: &str = include_str!("fixtures/error_not_found_storagezone.json");
const FIXTURE_UNAUTHORIZED: &str = include_str!("fixtures/error_unauthorized.json");

fn test_client(uri: &str) -> CoreClient {
    CoreClient::with_base_url("test-api-key", uri)
}

#[tokio::test]
async fn list_storage_zones_returns_paginated_items() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/storagezone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_PAGINATED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_storage_zones(None, None, None, None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 4);
    assert_eq!(result.total_items, 4);
    assert!(!result.has_more_items);
    assert_eq!(result.current_page, 1);

    let first = &result.items[0];
    assert_eq!(first.id, 9001);
    assert_eq!(first.name, "test-storage-zone-1");
    assert_eq!(first.region, "DE");
    // Password is deserialized from the API response, but skip_serializing
    // prevents it from appearing in serialized JSON output — verify that here.
    let json_output = serde_json::to_string(&first).unwrap();
    assert!(
        !json_output.contains("redacted-storage-password"),
        "password should not appear in serialized output"
    );
    assert_eq!(first.files_stored, 46);
    assert_eq!(first.storage_used, 5635181);
}

#[tokio::test]
async fn list_storage_zones_forwards_page_and_per_page() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/storagezone"))
        .and(query_param("page", "2"))
        .and(query_param("perPage", "10"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_PAGINATED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_storage_zones(Some(2), Some(10), None, None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 4);
}

#[tokio::test]
async fn list_storage_zones_with_search() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/storagezone"))
        .and(query_param("search", "test-storage-zone-1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_PAGINATED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_storage_zones(None, None, Some("test-storage-zone-1"), None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 4);
}

#[tokio::test]
async fn get_storage_zone_by_id() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/storagezone/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let zone = test_client(&server.uri())
        .get_storage_zone(9001)
        .await
        .unwrap();

    assert_eq!(zone.id, 9001);
    assert_eq!(zone.name, "test-storage-zone-1");
    assert_eq!(zone.storage_hostname, "storage.bunnycdn.com");
    assert_eq!(zone.files_stored, 46);
    assert!(!zone.deleted);
}

#[tokio::test]
async fn create_storage_zone_sends_correct_body() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "Name": "hoppy-test-zone",
        "Region": "DE"
    });

    Mock::given(method("POST"))
        .and(path("/storagezone"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(201).set_body_raw(FIXTURE_CREATE, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = CreateStorageZone::new("hoppy-test-zone", "DE");
    let zone = test_client(&server.uri())
        .create_storage_zone(&body)
        .await
        .unwrap();

    assert_eq!(zone.id, 9099);
    assert_eq!(zone.name, "hoppy-test-zone");
    assert_eq!(zone.region, "DE");
}

#[tokio::test]
async fn get_nonexistent_storage_zone_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/storagezone/99999"))
        .respond_with(
            ResponseTemplate::new(404).set_body_raw(FIXTURE_NOT_FOUND, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .get_storage_zone(99999)
        .await
        .unwrap_err();

    let api_err = err
        .downcast_ref::<ApiError>()
        .expect("should be an ApiError");
    assert_eq!(api_err.status_code, 404);
    assert!(
        api_err.error_key.contains("not_found") || api_err.message.contains("not found"),
        "unexpected error: {api_err}"
    );
}

#[tokio::test]
async fn invalid_api_key_returns_unauthorized() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/storagezone"))
        .respond_with(
            ResponseTemplate::new(401).set_body_raw(FIXTURE_UNAUTHORIZED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .list_storage_zones(None, None, None, None)
        .await
        .unwrap_err();

    let api_err = err
        .downcast_ref::<ApiError>()
        .expect("should be an ApiError");
    assert_eq!(api_err.status_code, 401);
    assert!(api_err.message.contains("Authorization has been denied"));
}

#[tokio::test]
async fn delete_storage_zone_success() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/storagezone/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_storage_zone(9001)
        .await
        .unwrap();
}

#[tokio::test]
async fn update_storage_zone_sends_correct_body() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "Rewrite404To200": true,
        "Custom404FilePath": "/errors/404.html"
    });

    Mock::given(method("POST"))
        .and(path("/storagezone/9001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateStorageZone::new()
        .rewrite_404_to_200(true)
        .custom_404_file_path("/errors/404.html");

    test_client(&server.uri())
        .update_storage_zone(9001, &body)
        .await
        .unwrap();
}
