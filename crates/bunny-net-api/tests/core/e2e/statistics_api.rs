use bunny_net_api::core::CoreClient;
use wiremock::matchers::{header, method, path};
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
        .get_statistics(None, None, None, false)
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
