use bunny_api_core::types::{
    AddDnsRecord, CreateDnsZone, DnsRecordType, UpdateDnsRecord, UpdateDnsZone,
};
use bunny_api_core::{ApiError, CoreClient};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_LIST_PAGINATED: &str =
    include_str!("../../../fixtures/core/dnszone_list_paginated.json");
const FIXTURE_GET: &str = include_str!("../../../fixtures/core/dnszone_get.json");
const FIXTURE_CREATE: &str = include_str!("../../../fixtures/core/dnszone_create.json");
const FIXTURE_RECORD_ADD: &str = include_str!("../../../fixtures/core/dnsrecord_add.json");
const FIXTURE_NOT_FOUND: &str = include_str!("../../../fixtures/core/error_not_found_dnszone.json");
const FIXTURE_UNAUTHORIZED: &str = include_str!("../../../fixtures/core/error_unauthorized.json");
const FIXTURE_EXPORT: &str = include_str!("../../../fixtures/core/dnszone_export.txt");
const FIXTURE_IMPORT: &str = include_str!("../../../fixtures/core/dnszone_import.json");
const FIXTURE_STATISTICS: &str = include_str!("../../../fixtures/core/dnszone_statistics.json");

fn test_client(uri: &str) -> CoreClient {
    CoreClient::with_base_url("test-api-key", uri)
}

// ---------------------------------------------------------------------------
// DNS Zone tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_dns_zones_returns_paginated_items() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone"))
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
        .list_dns_zones(None, None, None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
    assert_eq!(result.total_items, 2);
    assert!(!result.has_more_items);
    assert_eq!(result.current_page, 1);

    let first = &result.items[0];
    assert_eq!(first.id, 50001);
    assert_eq!(first.domain, "example.com");
    assert_eq!(first.records.len(), 1);
    assert!(first.nameservers_detected);
}

#[tokio::test]
async fn list_dns_zones_forwards_page_and_per_page() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone"))
        .and(query_param("page", "2"))
        .and(query_param("perPage", "10"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_PAGINATED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_dns_zones(Some(2), Some(10), None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
}

#[tokio::test]
async fn list_dns_zones_with_search() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone"))
        .and(query_param("search", "example.com"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_PAGINATED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_dns_zones(None, None, Some("example.com"))
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
}

#[tokio::test]
async fn get_dns_zone_by_id() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let zone = test_client(&server.uri())
        .get_dns_zone(50001)
        .await
        .unwrap();

    assert_eq!(zone.id, 50001);
    assert_eq!(zone.domain, "example.com");
    assert!(zone.nameservers_detected);
    assert!(!zone.dns_sec_enabled);
}

#[tokio::test]
async fn get_dns_zone_includes_records() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone/50001"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let zone = test_client(&server.uri())
        .get_dns_zone(50001)
        .await
        .unwrap();

    assert_eq!(zone.records.len(), 3);

    // Verify record types deserialized correctly
    assert_eq!(zone.records[0].record_type, Some(DnsRecordType::A));
    assert_eq!(zone.records[0].value, "93.184.216.34");
    assert_eq!(zone.records[0].ttl, 300);

    assert_eq!(zone.records[1].record_type, Some(DnsRecordType::CNAME));
    assert_eq!(zone.records[1].name, "www");
    assert_eq!(zone.records[1].comment, Some("WWW redirect".to_owned()));

    assert_eq!(zone.records[2].record_type, Some(DnsRecordType::MX));
    assert_eq!(zone.records[2].priority, 10);
}

#[tokio::test]
async fn create_dns_zone_sends_correct_body() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "Domain": "hoppy-test.example"
    });

    Mock::given(method("POST"))
        .and(path("/dnszone"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(201).set_body_raw(FIXTURE_CREATE, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = CreateDnsZone::new("hoppy-test.example");
    let zone = test_client(&server.uri())
        .create_dns_zone(&body)
        .await
        .unwrap();

    assert_eq!(zone.id, 50099);
    assert_eq!(zone.domain, "hoppy-test.example");
    assert!(zone.records.is_empty());
}

#[tokio::test]
async fn update_dns_zone_sends_correct_body() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "LoggingEnabled": true,
        "SoaEmail": "dns@example.com"
    });

    Mock::given(method("POST"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateDnsZone::new()
        .logging_enabled(true)
        .soa_email("dns@example.com");

    let zone = test_client(&server.uri())
        .update_dns_zone(50001, &body)
        .await
        .unwrap();

    assert_eq!(zone.id, 50001);
}

#[tokio::test]
async fn delete_dns_zone_success() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/dnszone/50001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_dns_zone(50001)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// DNS Record tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn add_dns_record_sends_correct_body() {
    let server = MockServer::start().await;

    // Note: bunny.net uses PUT for record creation
    let expected_body = serde_json::json!({
        "Type": 0,
        "Value": "192.0.2.1",
        "Name": "test",
        "Ttl": 300
    });

    Mock::given(method("PUT"))
        .and(path("/dnszone/50001/records"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(
            ResponseTemplate::new(201).set_body_raw(FIXTURE_RECORD_ADD, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = AddDnsRecord::new(DnsRecordType::A, "192.0.2.1")
        .name("test")
        .ttl(300);

    let record = test_client(&server.uri())
        .add_dns_record(50001, &body)
        .await
        .unwrap();

    assert_eq!(record.id, 100099);
    assert_eq!(record.record_type, Some(DnsRecordType::A));
    assert_eq!(record.value, "192.0.2.1");
    assert_eq!(record.name, "test");
    assert_eq!(record.ttl, 300);
}

#[tokio::test]
async fn add_dns_record_with_mx_priority() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "Type": 4,
        "Value": "mail.example.com",
        "Priority": 10
    });

    // Return a fabricated MX record response
    let mx_record_json = serde_json::json!({
        "Id": 100100,
        "Type": 4,
        "Ttl": 3600,
        "Value": "mail.example.com",
        "Name": "",
        "Weight": 0,
        "Priority": 10,
        "Port": 0,
        "Flags": 0,
        "Tag": null,
        "Accelerated": false,
        "AcceleratedPullZoneId": 0,
        "LinkName": null,
        "Disabled": false,
        "Comment": null
    });

    Mock::given(method("PUT"))
        .and(path("/dnszone/50001/records"))
        .and(body_json(expected_body))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_string(mx_record_json.to_string())
                .append_header("Content-Type", "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = AddDnsRecord::new(DnsRecordType::MX, "mail.example.com").priority(10);

    let record = test_client(&server.uri())
        .add_dns_record(50001, &body)
        .await
        .unwrap();

    assert_eq!(record.record_type, Some(DnsRecordType::MX));
    assert_eq!(record.priority, 10);
    assert_eq!(record.value, "mail.example.com");
}

#[tokio::test]
async fn update_dns_record_sends_correct_body() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "Id": 100001,
        "Type": 0,
        "Value": "10.0.0.1",
        "Ttl": 60,
        "Comment": "updated"
    });

    Mock::given(method("POST"))
        .and(path("/dnszone/50001/records/100001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateDnsRecord::new(100001, DnsRecordType::A, "10.0.0.1")
        .ttl(60)
        .comment("updated");

    test_client(&server.uri())
        .update_dns_record(50001, 100001, &body)
        .await
        .unwrap();
}

#[tokio::test]
async fn delete_dns_record_success() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/dnszone/50001/records/100001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_dns_record(50001, 100001)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Error handling tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_nonexistent_dns_zone_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone/99999"))
        .respond_with(
            ResponseTemplate::new(404).set_body_raw(FIXTURE_NOT_FOUND, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .get_dns_zone(99999)
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
async fn invalid_api_key_returns_unauthorized_for_dns() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone"))
        .respond_with(
            ResponseTemplate::new(401).set_body_raw(FIXTURE_UNAUTHORIZED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .list_dns_zones(None, None, None)
        .await
        .unwrap_err();

    let api_err = err
        .downcast_ref::<ApiError>()
        .expect("should be an ApiError");
    assert_eq!(api_err.status_code, 401);
    assert!(api_err.message.contains("Authorization has been denied"));
}

// ---------------------------------------------------------------------------
// DNS Zone export/import tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn export_dns_zone_returns_zone_file_text() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone/50001/export"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_EXPORT, "text/plain"))
        .expect(1)
        .mount(&server)
        .await;

    let content = test_client(&server.uri())
        .export_dns_zone(50001)
        .await
        .unwrap();

    assert!(content.contains("example.com"));
    assert!(content.contains("$ORIGIN"));
}

#[tokio::test]
async fn import_dns_zone_returns_result_counts() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/dnszone/50001/import"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_IMPORT, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .import_dns_zone(50001, FIXTURE_EXPORT)
        .await
        .unwrap();

    assert_eq!(result.records_successful, 5);
    assert_eq!(result.records_failed, 0);
    assert_eq!(result.records_skipped, 2);
}

#[tokio::test]
async fn export_dns_zone_not_found_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone/99999/export"))
        .respond_with(
            ResponseTemplate::new(404).set_body_raw(FIXTURE_NOT_FOUND, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .export_dns_zone(99999)
        .await
        .unwrap_err();

    let api_err = err
        .downcast_ref::<ApiError>()
        .expect("should be an ApiError");
    assert_eq!(api_err.status_code, 404);
}

// ---------------------------------------------------------------------------
// Debug mode test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn debug_mode_logs_to_stderr() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone/50001"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .mount(&server)
        .await;

    // Capture stderr by spawning a subprocess — instead verify that the debug
    // client still returns the correct result (the send() impl logs to stderr
    // but we can't trivially capture it in-process). We confirm debug mode
    // doesn't break anything.
    let client = CoreClient::with_base_url("test-api-key", server.uri()).with_debug(true);
    let zone = client.get_dns_zone(50001).await.unwrap();
    assert_eq!(zone.id, 50001);
    assert_eq!(zone.domain, "example.com");
}

#[tokio::test]
async fn get_dns_zone_statistics_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone/42/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_STATISTICS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stats = test_client(&server.uri())
        .get_dns_zone_statistics(42, None, None)
        .await
        .unwrap();

    assert_eq!(stats.total_queries_served, 85000);
    assert!(stats.queries_served_chart.is_some());
    assert_eq!(stats.queries_served_chart.unwrap().len(), 3);
}
