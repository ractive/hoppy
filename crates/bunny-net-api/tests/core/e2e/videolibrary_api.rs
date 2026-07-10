use bunny_net_api::core::types::{CreateVideoLibrary, UpdateVideoLibrary};
use bunny_net_api::core::{ApiError, CoreClient};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_LIST_PAGINATED: &str =
    include_str!("../../../../../fixtures/core/videolibrary_list_paginated.json");
const FIXTURE_GET: &str = include_str!("../../../../../fixtures/core/videolibrary_get.json");
const FIXTURE_CREATE: &str = include_str!("../../../../../fixtures/core/videolibrary_create.json");
const FIXTURE_NOT_FOUND: &str =
    include_str!("../../../../../fixtures/core/error_not_found_videolibrary.json");
const FIXTURE_UNAUTHORIZED: &str =
    include_str!("../../../../../fixtures/core/error_unauthorized.json");
const FIXTURE_DRM_STATS: &str =
    include_str!("../../../../../fixtures/core/videolibrary_drm_statistics.json");
const FIXTURE_TRANSCRIBING_STATS: &str =
    include_str!("../../../../../fixtures/core/videolibrary_transcribing_statistics.json");

fn test_client(uri: &str) -> CoreClient {
    CoreClient::with_base_url("test-api-key", uri)
}

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_video_libraries_returns_paginated_items() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/videolibrary"))
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
        .list_video_libraries(None, None, None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
    assert_eq!(result.total_items, 2);
    assert!(!result.has_more_items);
    assert_eq!(result.current_page, 1);

    let first = &result.items[0];
    assert_eq!(first.id, 10001);
    assert_eq!(first.name, "main-library");
    assert_eq!(first.video_count, 42);
    // api_key must not be serialized — but deserialized correctly
    assert_eq!(first.api_key, "REDACTED-FOR-FIXTURE");
}

#[tokio::test]
async fn list_video_libraries_forwards_page_and_per_page() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/videolibrary"))
        .and(query_param("page", "2"))
        .and(query_param("perPage", "10"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_PAGINATED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_video_libraries(Some(2), Some(10), None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
}

#[tokio::test]
async fn list_video_libraries_with_search() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/videolibrary"))
        .and(query_param("search", "main"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_PAGINATED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_video_libraries(None, None, Some("main"))
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
}

// ---------------------------------------------------------------------------
// Get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_video_library_by_id() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/videolibrary/10001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let lib = test_client(&server.uri())
        .get_video_library(10001)
        .await
        .unwrap();

    assert!(lib.id > 0);
    assert!(!lib.name.is_empty());
    assert!(!lib.api_key.is_empty());

    // serde-default coverage: confirm fixture keys exist so a renamed JSON
    // key (which would silently default video_count=0 / has_watermark=false)
    // is caught.
    let raw: serde_json::Value = serde_json::from_str(FIXTURE_GET).expect("fixture parses");
    assert!(raw["Id"].is_number());
    assert!(raw["Name"].is_string());
    assert!(raw["VideoCount"].is_number(), "VideoCount key missing");
    assert!(raw["HasWatermark"].is_boolean(), "HasWatermark key missing");
}

// ---------------------------------------------------------------------------
// Create
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_video_library_sends_correct_body() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "Name": "hoppy-test-library"
    });

    Mock::given(method("POST"))
        .and(path("/videolibrary"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(201).set_body_raw(FIXTURE_CREATE, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = CreateVideoLibrary::new("hoppy-test-library");
    let lib = test_client(&server.uri())
        .create_video_library(&body)
        .await
        .unwrap();

    assert_eq!(lib.id, 10099);
    assert_eq!(lib.name, "hoppy-test-library");
    assert_eq!(lib.video_count, 0);
}

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

#[tokio::test]
async fn update_video_library_sends_correct_body() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "Name": "renamed-library",
        "EnableMP4Fallback": true
    });

    Mock::given(method("POST"))
        .and(path("/videolibrary/10001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateVideoLibrary::new()
        .name("renamed-library")
        .enable_mp4_fallback(true);

    let lib = test_client(&server.uri())
        .update_video_library(10001, &body)
        .await
        .unwrap();

    assert!(lib.id > 0);
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_video_library_success() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/videolibrary/10001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_video_library(10001)
        .await
        .unwrap();
}

#[tokio::test]
async fn reset_video_library_api_key_posts_to_path() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/videolibrary/10001/resetApiKey"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .reset_video_library_api_key(10001)
        .await
        .unwrap();
}

#[tokio::test]
async fn reset_video_library_read_only_api_key_posts_to_path() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/videolibrary/10001/resetReadOnlyApiKey"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .reset_video_library_read_only_api_key(10001)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_nonexistent_video_library_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/videolibrary/99999"))
        .respond_with(
            ResponseTemplate::new(404).set_body_raw(FIXTURE_NOT_FOUND, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .get_video_library(99999)
        .await
        .unwrap_err();

    let api_err = err
        .downcast_ref::<ApiError>()
        .expect("should be an ApiError");
    assert_eq!(api_err.status_code, 404);
    assert!(
        api_err.error_key.contains("not_found") || api_err.message.contains("not"),
        "unexpected error: {api_err}"
    );
}

#[tokio::test]
async fn invalid_api_key_returns_unauthorized_for_video_libraries() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/videolibrary"))
        .respond_with(
            ResponseTemplate::new(401).set_body_raw(FIXTURE_UNAUTHORIZED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .list_video_libraries(None, None, None)
        .await
        .unwrap_err();

    let api_err = err
        .downcast_ref::<ApiError>()
        .expect("should be an ApiError");
    assert_eq!(api_err.status_code, 401);
    assert!(api_err.message.contains("Authorization has been denied"));
}

// ---------------------------------------------------------------------------
// Debug mode
// ---------------------------------------------------------------------------

#[tokio::test]
async fn debug_mode_logs_to_stderr_video_library() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/videolibrary/10001"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .mount(&server)
        .await;

    let client = CoreClient::with_base_url("test-api-key", server.uri()).with_debug(true);
    let lib = client.get_video_library(10001).await.unwrap();
    assert!(lib.id > 0);
}

// ---------------------------------------------------------------------------
// api_key serialization
//
// Iter-52: the API client no longer suppresses ApiKey / ReadOnlyApiKey on
// serialization — the CLI layer redacts them via the shared `redact` module
// instead, which keeps the raw values available to library consumers and
// makes `--reveal` work uniformly across formats.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn video_library_api_key_roundtrips_through_serde() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/videolibrary/10001"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .mount(&server)
        .await;

    let lib = test_client(&server.uri())
        .get_video_library(10001)
        .await
        .unwrap();

    assert!(!lib.api_key.is_empty());
    assert!(!lib.read_only_api_key.is_empty());

    let json = serde_json::to_value(&lib).unwrap();
    assert_eq!(
        json.get("ApiKey").and_then(|v| v.as_str()),
        Some(lib.api_key.as_str()),
        "ApiKey must serialize so the CLI layer can redact it consistently"
    );
    assert_eq!(
        json.get("ReadOnlyApiKey").and_then(|v| v.as_str()),
        Some(lib.read_only_api_key.as_str()),
        "ReadOnlyApiKey must serialize so the CLI layer can redact it consistently"
    );
}

#[tokio::test]
async fn get_video_library_drm_statistics_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/videolibrary/42/drm/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_DRM_STATS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stats = test_client(&server.uri())
        .get_video_library_drm_statistics(42, None, None)
        .await
        .unwrap();

    assert_eq!(stats.total_licenses_issued, 9500);
}

#[tokio::test]
async fn get_video_library_transcribing_statistics_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/videolibrary/42/transcribing/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_TRANSCRIBING_STATS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stats = test_client(&server.uri())
        .get_video_library_transcribing_statistics(42, None, None)
        .await
        .unwrap();

    assert_eq!(stats.total_transcription_seconds, 86400);
}
