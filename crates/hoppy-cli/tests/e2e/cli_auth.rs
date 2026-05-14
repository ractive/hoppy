use super::support;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn auth_check_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/billing"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/billing_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "auth", "check"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(
        json["Balance"].is_number(),
        "expected Balance to be a number"
    );
    assert!(
        json["ThisMonthCharges"].is_number(),
        "expected ThisMonthCharges to be a number"
    );
    assert!(
        json["BillingEnabled"].is_boolean(),
        "expected BillingEnabled to be a boolean"
    );
    assert!(
        json["MonthlyBandwidthUsed"].is_number(),
        "expected MonthlyBandwidthUsed to be a number"
    );
}

#[tokio::test]
async fn auth_check_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/billing"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/billing_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "auth", "check"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Column headers and fixed fields present
    assert!(stdout.contains("API Key"), "expected API Key row");
    assert!(stdout.contains("valid"), "expected API Key to be valid");
    assert!(stdout.contains("Balance"), "expected Balance row");
    assert!(
        stdout.contains("This Month Charges"),
        "expected This Month Charges row"
    );
    assert!(
        stdout.contains("Billing Enabled"),
        "expected Billing Enabled row"
    );
    assert!(
        stdout.contains("Monthly Bandwidth"),
        "expected Monthly Bandwidth row"
    );
    // Payment method shows card type from fixture (stable)
    assert!(stdout.contains("Visa"), "expected payment card type Visa");
}

#[tokio::test]
async fn auth_check_unauthorized() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/billing"))
        .respond_with(ResponseTemplate::new(401).set_body_raw(
            support::fixture("core/error_unauthorized.json"),
            "application/json",
        ))
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("bad-key", &server.uri())
        .args(["--format", "json", "auth", "check"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("rror") || stderr.contains("nauthorized"));
}

#[cfg(feature = "live-api")]
#[test]
fn live_auth_check() {
    let result = support::hoppy_live_json(&["auth", "check"]);
    assert!(result.success, "stderr: {}", result.stderr);
    let json = result.json.as_ref().expect("expected JSON output");
    assert!(
        json.get("Balance").is_some(),
        "expected 'Balance' key in response, got: {json}"
    );
}
