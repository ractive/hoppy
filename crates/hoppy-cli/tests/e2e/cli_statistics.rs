use super::support;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn account_statistics_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/account_statistics.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "statistics"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert_eq!(json["TotalBandwidthUsed"], 5368709120_i64);
    assert_eq!(json["TotalRequestsServed"], 150000);
    assert_eq!(json["AverageOriginResponseTime"], 245);
}

#[tokio::test]
async fn account_statistics_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/account_statistics.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "statistics"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Total Bandwidth Used"));
    assert!(stdout.contains("150000"));
    assert!(stdout.contains("87.00%"));
}

// ---------------------------------------------------------------------------
// Live API test
// ---------------------------------------------------------------------------

#[cfg(feature = "live-api")]
#[test]
fn live_account_statistics() {
    let result = support::hoppy_live_json(&["statistics"]);
    assert!(
        result.success,
        "account statistics failed — stderr: {}",
        result.stderr
    );
    let json = result.json.as_ref().unwrap();
    // Account stats should always have these fields
    assert!(
        json["TotalRequestsServed"].is_number(),
        "expected TotalRequestsServed to be a number"
    );
}
