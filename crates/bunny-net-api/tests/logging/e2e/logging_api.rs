use bunny_net_api::logging::{LegacyLogParams, LogQueryParams, LoggingClient};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_LOGS_QUERY: &str = include_str!("../../../../../fixtures/logging/logs_query.json");

fn test_client(uri: &str) -> LoggingClient {
    LoggingClient::with_base_url("test-api-key", uri)
}

#[tokio::test]
async fn query_logs_v2_returns_entries() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/pullzones/12345/logs"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LOGS_QUERY, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let resp = test_client(&server.uri())
        .query_logs(12345, &LogQueryParams::default())
        .await
        .unwrap();

    assert_eq!(resp.data.len(), 2);
    assert_eq!(resp.data[0].status_code, 200);
    assert_eq!(resp.data[0].cache_status.as_deref(), Some("HIT"));
    assert_eq!(resp.data[1].status_code, 502);
    assert!(!resp.pagination.has_more);
    assert_eq!(resp.pagination.returned, 2);
    assert_eq!(resp.query.pull_zone_id, 12345);
}

#[tokio::test]
async fn query_logs_v2_forwards_all_filters() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/pullzones/7/logs"))
        .and(query_param("from", "2026-07-08T00:00:00Z"))
        .and(query_param("to", "2026-07-09T00:00:00Z"))
        .and(query_param("status", "5xx"))
        .and(query_param("cacheStatus", "MISS"))
        .and(query_param("country", "EE"))
        .and(query_param("edgeLocation", "TLL"))
        .and(query_param("remoteIp", "203.0.113.5"))
        .and(query_param("urlContains", "/api"))
        .and(query_param("userAgentContains", "curl"))
        .and(query_param("refererContains", "example"))
        .and(query_param("search", "boom"))
        .and(query_param("requestId", "req-1"))
        .and(query_param("includeOriginShield", "true"))
        .and(query_param("limit", "50"))
        .and(query_param("offset", "10"))
        .and(query_param("order", "asc"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LOGS_QUERY, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = LogQueryParams {
        from: Some("2026-07-08T00:00:00Z".into()),
        to: Some("2026-07-09T00:00:00Z".into()),
        status: Some("5xx".into()),
        cache_status: Some("MISS".into()),
        country: Some("EE".into()),
        edge_location: Some("TLL".into()),
        remote_ip: Some("203.0.113.5".into()),
        url_contains: Some("/api".into()),
        user_agent_contains: Some("curl".into()),
        referer_contains: Some("example".into()),
        search: Some("boom".into()),
        request_id: Some("req-1".into()),
        include_origin_shield: Some(true),
        limit: Some(50),
        offset: Some(10),
        order: Some("asc".into()),
    };

    let resp = test_client(&server.uri())
        .query_logs(7, &params)
        .await
        .unwrap();
    assert_eq!(resp.data.len(), 2);
}

#[tokio::test]
async fn query_logs_v2_surfaces_structured_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/pullzones/999/logs"))
        .respond_with(ResponseTemplate::new(404).set_body_raw(
            r#"{"error":{"code":"logging_not_enabled","message":"Logging is not enabled for the pull zone."}}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .query_logs(999, &LogQueryParams::default())
        .await
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("logging_not_enabled"), "msg was: {msg}");
    assert!(msg.contains("Logging is not enabled"), "msg was: {msg}");
}

#[tokio::test]
async fn stream_legacy_logs_writes_raw_body() {
    let server = MockServer::start().await;
    let raw = "1720440000|200|HIT|/a.js\n1720440001|502|MISS|/api\n";

    Mock::given(method("GET"))
        .and(path("/07-08-26/12345.log"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("start", "1000"))
        .and(query_param("end", "2000"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(raw, "text/plain"))
        .expect(1)
        .mount(&server)
        .await;

    let params = LegacyLogParams {
        start: Some(1000),
        end: Some(2000),
        ..Default::default()
    };
    let mut buf: Vec<u8> = Vec::new();
    let written = test_client(&server.uri())
        .stream_legacy_logs("07-08-26", 12345, &params, &mut buf)
        .await
        .unwrap();

    assert_eq!(written as usize, raw.len());
    assert_eq!(String::from_utf8(buf).unwrap(), raw);
}

#[tokio::test]
async fn stream_legacy_logs_surfaces_error_body() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/07-08-26/999.log"))
        .respond_with(ResponseTemplate::new(404).set_body_raw(
            r#"{"error":{"code":"not_found","message":"no logs"}}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let mut buf: Vec<u8> = Vec::new();
    let err = test_client(&server.uri())
        .stream_legacy_logs("07-08-26", 999, &LegacyLogParams::default(), &mut buf)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not_found"), "err was: {err}");
    assert!(buf.is_empty(), "nothing should be written on error");
}
