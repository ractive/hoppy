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
    // "valid" must appear without "invalid" — naive contains("valid") is true
    // even when the text is "invalid".
    assert!(
        !stdout.contains("invalid"),
        "expected API Key to be valid, got invalid"
    );
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
    // Payment method row is present; the card-type value comes from
    // the fixture and drifts on refresh.
    assert!(
        stdout.contains("Payment Method"),
        "expected Payment Method row"
    );
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

#[tokio::test]
async fn auth_check_quiet_success_silent() {
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
        .args(["--quiet", "auth", "check"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(
        output.stdout.is_empty(),
        "expected empty stdout under --quiet, got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("tip:"),
        "--quiet must suppress hints, got:\n{stderr}"
    );
}

#[tokio::test]
async fn auth_check_quiet_failure_prints_error() {
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
        .args(["--quiet", "auth", "check"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("rror") || stderr.contains("nauthorized"),
        "expected an error message on stderr, got:\n{stderr}"
    );
}

// ---------------------------------------------------------------------------
// iter-84: --reveal wasn't threaded into `auth::core_client` call sites that
// built `ClientOpts` via `..Default::default()` (which silently pinned
// `reveal_secrets: false`). `auth check` was one of them — `--debug --reveal`
// still showed redacted `--debug` request/response bodies. Now that
// `ClientOpts` has no `Default` impl, every call site must state
// `reveal_secrets` explicitly, and `cli.reveal` is threaded through.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_check_debug_redacts_by_default() {
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
        .args(["--debug", "--quiet", "auth", "check"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // "AutomaticPaymentCardType" matches the sensitive-key patterns
    // ("payment") — without --reveal its string value is masked to a
    // length-only placeholder, not printed raw.
    assert!(
        !stderr.contains("\"AutomaticPaymentCardType\": \"<redacted>\""),
        "expected the field to be masked without --reveal, got:\n{stderr}"
    );
    assert!(
        stderr.contains("<set, length="),
        "expected a redaction placeholder in --debug output, got:\n{stderr}"
    );
}

#[tokio::test]
async fn auth_check_debug_reveal_shows_unredacted_body() {
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
        .args(["--debug", "--reveal", "--quiet", "auth", "check"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // With --reveal threaded all the way to `auth::core_client`, the raw
    // fixture value now passes through the --debug response body dump
    // unmasked.
    assert!(
        stderr.contains("\"AutomaticPaymentCardType\": \"<redacted>\""),
        "expected --reveal to show the raw field value in --debug output, got:\n{stderr}"
    );
    assert!(
        !stderr.contains("<set, length="),
        "expected no redaction placeholders under --reveal, got:\n{stderr}"
    );
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
