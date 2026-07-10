use bunny_net_api::core::{CoreClient, StatisticsQuery};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_ACCOUNT_STATS: &str =
    include_str!("../../../../../fixtures/core/account_statistics.json");

fn test_client(uri: &str) -> CoreClient {
    CoreClient::with_base_url("test-api-key", uri)
}

#[tokio::test]
async fn get_account_statistics_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ACCOUNT_STATS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stats = test_client(&server.uri())
        .get_statistics(&bunny_net_api::core::StatisticsQuery::default())
        .await
        .unwrap();

    // JSON-key presence: confirm the integral fields are actually in the
    // fixture so a renamed/missing key doesn't pass vacuously via serde
    // default (unsigned types default to 0 which always satisfies `>= 0`).
    let json: serde_json::Value = serde_json::from_str(FIXTURE_ACCOUNT_STATS).unwrap();
    assert!(
        json["TotalBandwidthUsed"].is_number(),
        "TotalBandwidthUsed key missing or not a number"
    );
    assert!(
        json["TotalRequestsServed"].is_number(),
        "TotalRequestsServed key missing or not a number"
    );
    assert!(
        json["AverageOriginResponseTime"].is_number(),
        "AverageOriginResponseTime key missing or not a number"
    );
    assert!(
        json["CacheHitRate"].is_number(),
        "CacheHitRate key missing or not a number"
    );

    assert!(stats.cache_hit_rate.is_finite());
    assert!(stats.cache_hit_rate >= 0.0);
    assert!(stats.bandwidth_used_chart.is_some());
    assert!(!stats.bandwidth_used_chart.unwrap().is_empty());
}

#[tokio::test]
async fn get_statistics_forwards_all_query_params() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("dateFrom", "2026-01-01"))
        .and(query_param("dateTo", "2026-01-31"))
        .and(query_param("pullZone", "5"))
        .and(query_param("serverZoneId", "42"))
        .and(query_param("hourly", "true"))
        .and(query_param("loadErrors", "true"))
        .and(query_param("loadOriginResponseTimes", "true"))
        .and(query_param("loadOriginTraffic", "true"))
        .and(query_param("loadRequestsServed", "true"))
        .and(query_param("loadBandwidthUsed", "true"))
        .and(query_param("loadOriginShieldBandwidth", "true"))
        .and(query_param("loadGeographicTrafficDistribution", "true"))
        .and(query_param("loadUserBalanceHistory", "true"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ACCOUNT_STATS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let query = StatisticsQuery {
        date_from: Some("2026-01-01"),
        date_to: Some("2026-01-31"),
        pull_zone: Some(5),
        server_zone_id: Some(42),
        hourly: true,
        load_errors: true,
        load_origin_response_times: true,
        load_origin_traffic: true,
        load_requests_served: true,
        load_bandwidth_used: true,
        load_origin_shield_bandwidth: true,
        load_geographic_traffic_distribution: true,
        load_user_balance_history: true,
    };
    let stats = test_client(&server.uri())
        .get_statistics(&query)
        .await
        .unwrap();
    assert!(stats.cache_hit_rate.is_finite());
}
