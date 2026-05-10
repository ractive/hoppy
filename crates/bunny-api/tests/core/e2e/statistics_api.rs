use bunny_api::core::CoreClient;
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

    assert_eq!(stats.total_bandwidth_used, 5368709120);
    assert_eq!(stats.total_requests_served, 150000);
    assert_eq!(stats.average_origin_response_time, 245);
    assert!((stats.cache_hit_rate - 0.87).abs() < 0.001);
    assert!(stats.bandwidth_used_chart.is_some());
    assert_eq!(stats.bandwidth_used_chart.unwrap().len(), 3);
}
