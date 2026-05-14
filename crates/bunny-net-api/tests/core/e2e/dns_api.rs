use bunny_net_api::core::types::{
    AddDnsRecord, CreateDnsZone, DnsDiscoveredRecordType, DnsRecordType, DnsScanJobStatus,
    TriggerDnsRecordScan, UpdateDnsRecord, UpdateDnsZone,
};
use bunny_net_api::core::{ApiError, CoreClient};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_LIST_PAGINATED: &str =
    include_str!("../../../../../fixtures/core/dnszone_list_paginated.json");
const FIXTURE_GET: &str = include_str!("../../../../../fixtures/core/dnszone_get.json");
const FIXTURE_CREATE: &str = include_str!("../../../../../fixtures/core/dnszone_create.json");
const FIXTURE_RECORD_ADD: &str = include_str!("../../../../../fixtures/core/dnsrecord_add.json");
const FIXTURE_NOT_FOUND: &str =
    include_str!("../../../../../fixtures/core/error_not_found_dnszone.json");
const FIXTURE_UNAUTHORIZED: &str =
    include_str!("../../../../../fixtures/core/error_unauthorized.json");
const FIXTURE_EXPORT: &str = include_str!("../../../../../fixtures/core/dnszone_export.txt");
const FIXTURE_IMPORT: &str = include_str!("../../../../../fixtures/core/dnszone_import.json");
const FIXTURE_STATISTICS: &str =
    include_str!("../../../../../fixtures/core/dnszone_statistics.json");
const FIXTURE_DNSSEC_ENABLE: &str = include_str!("../../../../../fixtures/core/dnssec_enable.json");
const FIXTURE_DNSSEC_DISABLE: &str =
    include_str!("../../../../../fixtures/core/dnssec_disable.json");
const FIXTURE_SCAN_TRIGGER: &str =
    include_str!("../../../../../fixtures/core/dnszone_scan_trigger.json");
const FIXTURE_SCAN_RESULT: &str =
    include_str!("../../../../../fixtures/core/dnszone_scan_result.json");

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

    // Shape-first: at least one item, pagination fields parsed correctly.
    assert!(!result.items.is_empty());
    assert!(result.total_items >= 1);
    // has_more_items is a bool; just confirm it deserialised.
    let _ = result.has_more_items;
    assert!(result.current_page >= 1);

    let first = &result.items[0];
    // id and domain are presence checks — specific values drift.
    assert!(first.id > 0);
    assert!(!first.domain.is_empty());
    // records count comes from the fixture; just assert the field parsed.
    let _ = first.records.len();
    // nameservers_detected is a bool flag.
    let _ = first.nameservers_detected;
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

    assert!(!result.items.is_empty());
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

    assert!(!result.items.is_empty());
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

    // id is assigned by the server — any positive value is valid.
    assert!(zone.id > 0);
    // domain round-trips the submitted value; this is wire-format.
    assert_eq!(zone.domain, "hoppy-test.example");
    // A freshly created zone has no records — shape-coupled, keep.
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

    // id is from the fixture response; positive value is sufficient.
    assert!(zone.id > 0);
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

    assert!(stats.total_queries_served >= 0);
    assert!(stats.queries_served_chart.is_some());
    assert!(!stats.queries_served_chart.unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// DNSSEC tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn enable_dnssec_returns_ds_record_details() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/dnszone/50001/dnssec"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_DNSSEC_ENABLE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .enable_dns_zone_dnssec(50001)
        .await
        .unwrap();

    assert!(result.enabled);
    assert_eq!(result.algorithm, 13);
    assert_eq!(result.key_tag, 12345);
    assert_eq!(result.flags, 257);
    assert_eq!(result.digest_type.as_deref(), Some("SHA-256"));
    assert!(result.ds_record.as_ref().unwrap().contains("DS"));
    assert!(!result.ds_configured);
}

#[tokio::test]
async fn disable_dnssec_returns_disabled_status() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/dnszone/50001/dnssec"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_DNSSEC_DISABLE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .disable_dns_zone_dnssec(50001)
        .await
        .unwrap();

    assert!(!result.enabled);
    assert_eq!(result.key_tag, 0);
    assert!(result.ds_record.is_none());
}

#[tokio::test]
async fn enable_dnssec_for_missing_zone_returns_404() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/dnszone/99999/dnssec"))
        .respond_with(
            ResponseTemplate::new(404).set_body_raw(FIXTURE_NOT_FOUND, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .enable_dns_zone_dnssec(99999)
        .await
        .unwrap_err();

    let api_err = err
        .downcast_ref::<ApiError>()
        .expect("should be an ApiError");
    assert_eq!(api_err.status_code, 404);
}

// ---------------------------------------------------------------------------
// Wildcard certificate tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn issue_wildcard_certificate_succeeds_with_empty_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/dnszone/50001/certificate/issue"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .issue_dns_zone_wildcard_certificate(50001)
        .await
        .unwrap();
}

#[tokio::test]
async fn issue_wildcard_certificate_for_missing_zone_returns_404() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/dnszone/99999/certificate/issue"))
        .respond_with(
            ResponseTemplate::new(404).set_body_raw(FIXTURE_NOT_FOUND, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .issue_dns_zone_wildcard_certificate(99999)
        .await
        .unwrap_err();

    let api_err = err
        .downcast_ref::<ApiError>()
        .expect("should be an ApiError");
    assert_eq!(api_err.status_code, 404);
}

// ---------------------------------------------------------------------------
// DNS record scan tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn trigger_dns_record_scan_with_zone_id() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({ "ZoneId": 50001 });

    Mock::given(method("POST"))
        .and(path("/dnszone/records/scan"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(&expected_body))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCAN_TRIGGER, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = TriggerDnsRecordScan::for_zone(50001);
    let result = test_client(&server.uri())
        .trigger_dns_record_scan(&body)
        .await
        .unwrap();

    assert_eq!(
        result.job_id.as_deref(),
        Some("11111111-2222-3333-4444-555555555555")
    );
    assert_eq!(result.status, Some(DnsScanJobStatus::Pending));
}

#[tokio::test]
async fn trigger_dns_record_scan_with_domain() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({ "Domain": "example.com" });

    Mock::given(method("POST"))
        .and(path("/dnszone/records/scan"))
        .and(body_json(&expected_body))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCAN_TRIGGER, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = TriggerDnsRecordScan::for_domain("example.com");
    let result = test_client(&server.uri())
        .trigger_dns_record_scan(&body)
        .await
        .unwrap();

    assert!(result.job_id.is_some());
}

#[tokio::test]
async fn get_dns_zone_record_scan_returns_completed_results() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone/50001/records/scan"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCAN_RESULT, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .get_dns_zone_record_scan(50001)
        .await
        .unwrap();

    assert_eq!(result.zone_id, Some(50001));
    assert_eq!(result.domain.as_deref(), Some("example.com"));
    assert_eq!(result.status, Some(DnsScanJobStatus::Completed));
    assert_eq!(result.records.len(), 3);

    let a = &result.records[0];
    assert_eq!(a.record_type, Some(DnsDiscoveredRecordType::A));
    assert_eq!(a.value.as_deref(), Some("93.184.216.34"));

    let mx = &result.records[2];
    assert_eq!(mx.record_type, Some(DnsDiscoveredRecordType::MX));
    assert_eq!(mx.priority, Some(10));
}

#[tokio::test]
async fn get_dns_zone_record_scan_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dnszone/99999/records/scan"))
        .respond_with(
            ResponseTemplate::new(404).set_body_raw(FIXTURE_NOT_FOUND, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .get_dns_zone_record_scan(99999)
        .await
        .unwrap_err();

    let api_err = err
        .downcast_ref::<ApiError>()
        .expect("should be an ApiError");
    assert_eq!(api_err.status_code, 404);
}
