//! e2e coverage for the account/admin endpoints added in iter-75: API keys,
//! billing summary, payment requests, invoice PDFs (streamed), region and
//! country reference data, global search, and the user audit log.

use bunny_net_api::core::CoreClient;
use bunny_net_api::core::types::{AuditLogOrder, UserAuditQuery};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_APIKEY: &str = include_str!("../../../../../fixtures/core/apikey_list.json");
const FIXTURE_BILLING_SUMMARY: &str =
    include_str!("../../../../../fixtures/core/billing_summary.json");
const FIXTURE_PAYMENT_REQUESTS: &str =
    include_str!("../../../../../fixtures/core/billing_payment_requests.json");
const FIXTURE_REGION: &str = include_str!("../../../../../fixtures/core/region_list.json");
const FIXTURE_COUNTRY: &str = include_str!("../../../../../fixtures/core/country_list.json");
const FIXTURE_SEARCH: &str = include_str!("../../../../../fixtures/core/search_results.json");
const FIXTURE_AUDIT: &str = include_str!("../../../../../fixtures/core/user_audit.json");
const FIXTURE_UNAUTHORIZED: &str =
    include_str!("../../../../../fixtures/core/error_unauthorized.json");

fn test_client(uri: &str) -> CoreClient {
    CoreClient::with_base_url("test-api-key", uri)
}

// ---------------------------------------------------------------------------
// API keys
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_api_keys_returns_paginated_envelope() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/apikey"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_APIKEY, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let list = test_client(&server.uri())
        .list_api_keys(None, None)
        .await
        .unwrap();

    assert_eq!(list.total_items, 2);
    assert_eq!(list.items.len(), 2);
    assert_eq!(list.items[0].id, 1001);
    assert!(list.items[0].key.is_some());
    assert_eq!(
        list.items[1].roles.as_ref().unwrap(),
        &vec!["Billing".to_owned(), "ReadOnly".to_owned()]
    );

    // JSON-key presence guards.
    let json: serde_json::Value = serde_json::from_str(FIXTURE_APIKEY).unwrap();
    assert!(json["Items"][0]["Key"].is_string());
    assert!(json["Items"][0]["Roles"].is_array());
}

#[tokio::test]
async fn list_api_keys_forwards_explicit_pagination() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/apikey"))
        .and(query_param("page", "2"))
        .and(query_param("perPage", "25"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_APIKEY, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let list = test_client(&server.uri())
        .list_api_keys(Some(2), Some(25))
        .await
        .unwrap();
    assert_eq!(list.items.len(), 2);
}

#[tokio::test]
async fn list_api_keys_unauthorized_surfaces_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/apikey"))
        .respond_with(
            ResponseTemplate::new(401).set_body_raw(FIXTURE_UNAUTHORIZED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .list_api_keys(None, None)
        .await
        .unwrap_err();
    let api_err = err.downcast_ref::<bunny_net_api::core::ApiError>().unwrap();
    assert_eq!(api_err.status_code, 401);
}

// ---------------------------------------------------------------------------
// Billing summary + payment requests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_billing_summary_parses_entries() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/billing/summary"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_BILLING_SUMMARY, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let summary = test_client(&server.uri())
        .get_billing_summary()
        .await
        .unwrap();

    assert_eq!(summary.len(), 2);
    assert_eq!(summary[0].pull_zone_id, 500123);
    assert!(summary[0].monthly_usage.is_finite());
    assert!(summary[0].monthly_bandwidth_used > 0);

    let json: serde_json::Value = serde_json::from_str(FIXTURE_BILLING_SUMMARY).unwrap();
    assert!(json[0]["PullZoneId"].is_number());
    assert!(json[0]["MonthlyUsage"].is_number());
    assert!(json[0]["MonthlyBandwidthUsed"].is_number());
}

#[tokio::test]
async fn list_payment_requests_parses_entries() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/billing/payment-requests"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_PAYMENT_REQUESTS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let requests = test_client(&server.uri())
        .list_payment_requests()
        .await
        .unwrap();

    assert_eq!(requests.len(), 2);
    assert!(requests[0].paid);
    assert!(!requests[1].paid);
    assert!(requests[1].date_paid.is_none());
    assert_eq!(
        requests[1].bank_transfer_reference.as_deref(),
        Some("REF-90002")
    );

    let json: serde_json::Value = serde_json::from_str(FIXTURE_PAYMENT_REQUESTS).unwrap();
    assert!(json[0]["Id"].is_number());
    assert!(json[0]["Amount"].is_number());
    assert!(json[0]["Paid"].is_boolean());
}

// ---------------------------------------------------------------------------
// Invoice PDFs (streamed to a writer)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn download_billing_invoice_pdf_streams_bytes() {
    let server = MockServer::start().await;

    // Minimal PDF payload.
    let pdf: &[u8] = b"%PDF-1.4\n%mock invoice\n%%EOF\n";
    Mock::given(method("GET"))
        .and(path("/billing/summary/44001/pdf"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(pdf.to_vec(), "application/pdf"))
        .expect(1)
        .mount(&server)
        .await;

    let mut buf: Vec<u8> = Vec::new();
    let n = test_client(&server.uri())
        .download_billing_invoice_pdf(44001, &mut buf)
        .await
        .unwrap();

    assert_eq!(n as usize, pdf.len());
    assert_eq!(buf, pdf);
    assert!(buf.starts_with(b"%PDF"));
}

#[tokio::test]
async fn download_billing_invoice_pdf_error_is_parsed() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/billing/summary/999/pdf"))
        .respond_with(
            ResponseTemplate::new(404).set_body_raw(FIXTURE_UNAUTHORIZED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let mut buf: Vec<u8> = Vec::new();
    let err = test_client(&server.uri())
        .download_billing_invoice_pdf(999, &mut buf)
        .await
        .unwrap_err();
    let api_err = err.downcast_ref::<bunny_net_api::core::ApiError>().unwrap();
    assert_eq!(api_err.status_code, 404);
    // Nothing was written to the sink on error.
    assert!(buf.is_empty());
}

#[tokio::test]
async fn download_payment_request_pdf_streams_bytes() {
    let server = MockServer::start().await;

    let pdf: &[u8] = b"%PDF-1.5\n%payment request\n%%EOF\n";
    Mock::given(method("GET"))
        .and(path("/billing/payment-request-invoice/90002/pdf"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(pdf.to_vec(), "application/pdf"))
        .expect(1)
        .mount(&server)
        .await;

    let mut buf: Vec<u8> = Vec::new();
    let n = test_client(&server.uri())
        .download_payment_request_pdf(90002, &mut buf)
        .await
        .unwrap();

    assert_eq!(n as usize, pdf.len());
    assert_eq!(buf, pdf);
}

// ---------------------------------------------------------------------------
// Reference data: regions + countries
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_regions_parses_pricing() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/region"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_REGION, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let regions = test_client(&server.uri()).list_regions().await.unwrap();
    assert_eq!(regions.len(), 2);
    assert_eq!(regions[0].region_code.as_deref(), Some("DE"));
    assert!(regions[0].price_per_gigabyte.is_finite());
    assert!(regions[0].allow_latency_routing);

    let json: serde_json::Value = serde_json::from_str(FIXTURE_REGION).unwrap();
    assert!(json[0]["RegionCode"].is_string());
    assert!(json[0]["PricePerGigabyte"].is_number());
}

#[tokio::test]
async fn list_countries_parses_iso_codes() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/country"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_COUNTRY, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let countries = test_client(&server.uri()).list_countries().await.unwrap();
    assert_eq!(countries.len(), 2);
    assert_eq!(countries[0].iso_code.as_deref(), Some("DE"));
    assert!(countries[0].is_eu);
    assert!(!countries[1].is_eu);

    let json: serde_json::Value = serde_json::from_str(FIXTURE_COUNTRY).unwrap();
    assert!(json[0]["IsoCode"].is_string());
    assert!(json[0]["IsEU"].is_boolean());
}

// ---------------------------------------------------------------------------
// Global search
// ---------------------------------------------------------------------------

#[tokio::test]
async fn search_forwards_query_and_parses_results() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search"))
        .and(query_param("search", "example"))
        .and(query_param("from", "0"))
        .and(query_param("size", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_SEARCH, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let results = test_client(&server.uri())
        .search("example", Some(0), Some(10))
        .await
        .unwrap();

    assert_eq!(results.total, 3);
    let items = results.search_results.unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].result_type.as_deref(), Some("PullZone"));
    assert_eq!(items[0].id, 500123);

    let json: serde_json::Value = serde_json::from_str(FIXTURE_SEARCH).unwrap();
    assert!(json["SearchResults"][0]["Type"].is_string());
    assert!(json["SearchResults"][0]["Id"].is_number());
}

#[tokio::test]
async fn search_omits_unset_pagination() {
    let server = MockServer::start().await;

    // Only `search` is sent; from/size are absent.
    Mock::given(method("GET"))
        .and(path("/search"))
        .and(query_param("search", "test"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_SEARCH, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let results = test_client(&server.uri())
        .search("test", None, None)
        .await
        .unwrap();
    assert_eq!(results.total, 3);
}

// ---------------------------------------------------------------------------
// User audit log
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_user_audit_forwards_all_filters() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/user/audit/2026-07-01"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("Product", "CDN"))
        .and(query_param("ResourceType", "PullZone"))
        .and(query_param("ResourceId", "500123"))
        .and(query_param("ActorId", "user-42"))
        .and(query_param("Order", "Descending"))
        .and(query_param("ContinuationToken", "tok"))
        .and(query_param("Limit", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_AUDIT, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let query = UserAuditQuery {
        product: vec!["CDN".to_owned()],
        resource_type: vec!["PullZone".to_owned()],
        resource_id: vec!["500123".to_owned()],
        actor_id: vec!["user-42".to_owned()],
        order: Some(AuditLogOrder::Descending),
        continuation_token: Some("tok".to_owned()),
        limit: Some(50),
    };
    let log = test_client(&server.uri())
        .get_user_audit("2026-07-01", &query)
        .await
        .unwrap();

    assert!(log.has_more_data);
    assert_eq!(
        log.continuation_token.as_deref(),
        Some("next-page-token-abc")
    );
    let entries = log.logs.unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].action.as_deref(), Some("Update"));
    assert_eq!(entries[1].actor_type.as_deref(), Some("ApiKey"));

    let json: serde_json::Value = serde_json::from_str(FIXTURE_AUDIT).unwrap();
    assert!(json["Logs"][0]["Timestamp"].is_string());
    assert!(json["HasMoreData"].is_boolean());
}

#[tokio::test]
async fn get_user_audit_minimal_query_sends_no_filters() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/user/audit/2026-07-01"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_AUDIT, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let log = test_client(&server.uri())
        .get_user_audit("2026-07-01", &UserAuditQuery::default())
        .await
        .unwrap();
    assert_eq!(log.logs.unwrap().len(), 2);
}
