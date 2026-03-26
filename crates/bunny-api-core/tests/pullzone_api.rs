use bunny_api_core::{ApiError, CoreClient};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_LIST_PAGINATED: &str =
    include_str!("../../../fixtures/core/pullzone_list_paginated.json");
const FIXTURE_GET: &str = include_str!("../../../fixtures/core/pullzone_get.json");
const FIXTURE_UNAUTHORIZED: &str = include_str!("../../../fixtures/core/error_unauthorized.json");
const FIXTURE_NOT_FOUND: &str =
    include_str!("../../../fixtures/core/error_not_found_storagezone.json");
const FIXTURE_OPTIMIZER_STATS: &str =
    include_str!("../../../fixtures/core/pullzone_optimizer_statistics.json");
const FIXTURE_ORIGINSHIELD_STATS: &str =
    include_str!("../../../fixtures/core/pullzone_originshield_statistics.json");
const FIXTURE_SAFEHOP_STATS: &str =
    include_str!("../../../fixtures/core/pullzone_safehop_statistics.json");

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
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_PAGINATED, "application/json"),
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
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_PAGINATED, "application/json"),
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
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
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
            ResponseTemplate::new(401).set_body_raw(FIXTURE_UNAUTHORIZED, "application/json"),
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

// ---------------------------------------------------------------------------
// URL purge tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn purge_url_sends_url_as_query_param() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/purge"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("url", "https://cdn.example.com/style.css"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .purge_url("https://cdn.example.com/style.css")
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Pull Zone hostname & SSL tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn add_hostname_sends_correct_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/addHostname"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "cdn.example.com" }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .add_hostname(1001, "cdn.example.com")
        .await
        .unwrap();
}

#[tokio::test]
async fn remove_hostname_sends_correct_body() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/pullzone/1001/removeHostname"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "cdn.example.com" }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .remove_hostname(1001, "cdn.example.com")
        .await
        .unwrap();
}

#[tokio::test]
async fn load_free_certificate_sends_hostname_as_query_param() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/loadFreeCertificate"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("hostname", "cdn.example.com"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .load_free_certificate("cdn.example.com")
        .await
        .unwrap();
}

#[tokio::test]
async fn set_force_ssl_sends_correct_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/setForceSSL"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "cdn.example.com", "ForceSSL": true }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .set_force_ssl(1001, "cdn.example.com", true)
        .await
        .unwrap();
}

#[tokio::test]
async fn add_certificate_sends_correct_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/addCertificate"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "Hostname": "cdn.example.com",
            "Certificate": "base64cert",
            "CertificateKey": "base64key"
        })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .add_certificate(1001, "cdn.example.com", "base64cert", "base64key")
        .await
        .unwrap();
}

#[tokio::test]
async fn remove_certificate_sends_correct_body() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/pullzone/1001/removeCertificate"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "cdn.example.com" }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .remove_certificate(1001, "cdn.example.com")
        .await
        .unwrap();
}

#[tokio::test]
async fn add_hostname_not_found_returns_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/99999/addHostname"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(404).set_body_raw(FIXTURE_NOT_FOUND, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .add_hostname(99999, "cdn.example.com")
        .await
        .unwrap_err();

    let api_err = err
        .downcast_ref::<ApiError>()
        .expect("should be an ApiError");
    assert_eq!(api_err.status_code, 404);
}

#[tokio::test]
async fn get_pull_zone_optimizer_statistics_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone/42/optimizer/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_OPTIMIZER_STATS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stats = test_client(&server.uri())
        .get_pull_zone_optimizer_statistics(42, None, None, false)
        .await
        .unwrap();

    assert!((stats.total_requests_optimized - 45000.0).abs() < 0.001);
    assert!((stats.average_compression_ratio - 68.3).abs() < 0.001);
}

#[tokio::test]
async fn get_pull_zone_origin_shield_statistics_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone/42/originshield/queuestatistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ORIGINSHIELD_STATS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stats = test_client(&server.uri())
        .get_pull_zone_origin_shield_statistics(42, None, None, false)
        .await
        .unwrap();

    assert!(stats.concurrent_requests_chart.is_some());
    assert_eq!(stats.concurrent_requests_chart.unwrap().len(), 3);
}

#[tokio::test]
async fn get_pull_zone_safehop_statistics_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone/42/safehop/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SAFEHOP_STATS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stats = test_client(&server.uri())
        .get_pull_zone_safehop_statistics(42, None, None, false)
        .await
        .unwrap();

    assert!((stats.total_requests_retried - 320.0).abs() < 0.001);
    assert!((stats.total_requests_saved - 12800.0).abs() < 0.001);
}
