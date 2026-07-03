use std::sync::LazyLock;

use super::support;

use regex::Regex;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

static RE_TOTAL_REQUESTS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Total Requests Served\s*\|\s*\d+").unwrap());
static RE_CACHE_HIT_RATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Cache Hit Rate\s*\|\s*\d+\.\d+%").unwrap());

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
    assert!(
        json["TotalBandwidthUsed"].is_number(),
        "expected TotalBandwidthUsed to be a number"
    );
    assert!(
        json["TotalBandwidthUsed"].as_i64().unwrap_or(-1) >= 0,
        "expected TotalBandwidthUsed >= 0"
    );
    assert!(
        json["TotalRequestsServed"].is_number(),
        "expected TotalRequestsServed to be a number"
    );
    assert!(
        json["AverageOriginResponseTime"].is_number(),
        "expected AverageOriginResponseTime to be a number"
    );
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
    assert!(
        RE_TOTAL_REQUESTS.is_match(&stdout),
        "expected a numeric Total Requests Served value in table output"
    );
    assert!(
        RE_CACHE_HIT_RATE.is_match(&stdout),
        "expected a percentage Cache Hit Rate in table output"
    );
}

// ---------------------------------------------------------------------------
// --hourly hint (table renders the same summary rows regardless of --hourly;
// a stderr hint tells the user where the per-hour buckets actually live)
// ---------------------------------------------------------------------------

async fn mock_account_statistics() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/account_statistics.json"),
            "application/json",
        ))
        .mount(&server)
        .await;
    server
}

#[tokio::test]
async fn hourly_table_prints_json_hint() {
    let server = mock_account_statistics().await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "statistics", "--hourly"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("tip:") && stderr.contains("--format json"),
        "expected a --format json hint in stderr for --hourly table output, got:\n{stderr}"
    );
}

#[tokio::test]
async fn non_hourly_table_has_no_hint() {
    let server = mock_account_statistics().await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "statistics"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("tip:"),
        "non-hourly table output must not print the hourly hint, got:\n{stderr}"
    );
}

#[tokio::test]
async fn hourly_table_hint_suppressed_by_quiet() {
    let server = mock_account_statistics().await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--quiet", "--format", "table", "statistics", "--hourly"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Total Bandwidth Used"),
        "expected the summary table to still print under --quiet, got:\n{stdout}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("tip:"),
        "--quiet must suppress the hourly hint, got:\n{stderr}"
    );
}

#[tokio::test]
async fn hourly_json_has_no_hint() {
    let server = mock_account_statistics().await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "statistics", "--hourly"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("tip:"),
        "--format json must stay pure — no hint expected, got:\n{stderr}"
    );
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(
        json["BandwidthUsedChart"].is_object(),
        "expected hourly chart data in the JSON payload"
    );
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
