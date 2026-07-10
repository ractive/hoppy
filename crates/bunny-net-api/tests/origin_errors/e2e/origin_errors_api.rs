use bunny_net_api::origin_errors::OriginErrorsClient;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_ORIGIN_ERRORS: &str =
    include_str!("../../../../../fixtures/origin-errors/origin_errors.json");

fn test_client(uri: &str) -> OriginErrorsClient {
    OriginErrorsClient::with_base_url("test-api-key", uri)
}

#[tokio::test]
async fn get_origin_errors_returns_entries() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/12345/10-29-2025"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ORIGIN_ERRORS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let resp = test_client(&server.uri())
        .get_origin_errors(12345, "10-29-2025")
        .await
        .unwrap();

    assert_eq!(resp.logs.len(), 2);
    let first = &resp.logs[0];
    assert_eq!(first.timestamp, Some(1728952065848));
    let labels = first.labels.as_ref().unwrap();
    assert_eq!(labels.error_code.as_deref(), Some("dns_lookup"));
    assert_eq!(labels.status_code.as_deref(), Some("502"));
    assert_eq!(labels.server_zone.as_deref(), Some("CA"));
    assert_eq!(
        resp.logs[1].labels.as_ref().unwrap().status_code.as_deref(),
        Some("504")
    );
}

#[tokio::test]
async fn get_origin_errors_rejects_bad_date_locally() {
    // No mock — validation should reject before any HTTP.
    let err = test_client("http://127.0.0.1:1")
        .get_origin_errors(1, "2025-10-29")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("MM-DD-YYYY"), "err was: {err}");
}

#[tokio::test]
async fn get_origin_errors_surfaces_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/1/10-29-2025"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .get_origin_errors(1, "10-29-2025")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("401"), "err was: {err}");
}
