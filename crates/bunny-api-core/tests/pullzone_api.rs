use bunny_api_core::{ApiError, CoreClient};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_LIST_PAGINATED: &str =
    include_str!("fixtures/pullzone_list_paginated.json");
const FIXTURE_GET: &str = include_str!("fixtures/pullzone_get.json");
const FIXTURE_UNAUTHORIZED: &str =
    include_str!("fixtures/error_unauthorized.json");

fn test_client(uri: &str) -> CoreClient {
    CoreClient::with_base_url("test-api-key", uri)
}

#[tokio::test]
async fn list_pull_zones_returns_paginated_items() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_LIST_PAGINATED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_pull_zones(None, None, None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
    assert_eq!(result.total_items, 2);
    assert!(!result.has_more_items);
    assert_eq!(result.current_page, 1);
}

#[tokio::test]
async fn list_pull_zones_forwards_explicit_page_and_per_page() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(query_param("page", "3"))
        .and(query_param("perPage", "25"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_LIST_PAGINATED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_pull_zones(Some(3), Some(25), None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
}

#[tokio::test]
async fn get_pull_zone_returns_single_zone() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let zone = test_client(&server.uri())
        .get_pull_zone(1001)
        .await
        .unwrap();

    assert_eq!(zone.id, 1001);
    assert_eq!(zone.name, "test-zone-19");
    assert!(zone.enabled);
}

#[tokio::test]
async fn invalid_api_key_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_raw(FIXTURE_UNAUTHORIZED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .list_pull_zones(None, None, None)
        .await
        .unwrap_err();

    let api_err = err
        .downcast_ref::<ApiError>()
        .expect("should be an ApiError");
    assert_eq!(api_err.status_code, 401);
    assert!(api_err.message.contains("Authorization has been denied"));
}
