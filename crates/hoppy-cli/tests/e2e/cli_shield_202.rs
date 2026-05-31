/// E2E tests asserting that Shield 202 plan-gate error envelopes surface as
/// non-zero exit codes with the API message in stderr.
use super::support;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Body returned by bunny.net Shield when a feature is gated by plan tier.
const PLAN_GATE_202_BODY: &str = r#"{
    "data": null,
    "error": {
        "statusCode": 202,
        "success": false,
        "message": "Unable to make changes whilst on the Basic tier of Bunny Shield. Please upgrade to Advanced to enable Bot Detection.",
        "errorKey": "invalid_plan_type.bot_detection"
    }
}"#;

#[tokio::test]
async fn bot_detection_get_202_plan_gate_exits_nonzero_with_message() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/42/bot-detection"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(202).set_body_raw(PLAN_GATE_202_BODY, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["shield", "bot-detection", "get", "--shield-zone-id", "42"])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "expected non-zero exit code for 202 plan-gate error"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Basic tier"),
        "stderr should contain plan upgrade message; got: {stderr}"
    );
    assert!(
        stderr.contains("invalid_plan_type.bot_detection"),
        "stderr should contain errorKey; got: {stderr}"
    );
}

#[tokio::test]
async fn rate_limit_list_202_plan_gate_exits_nonzero_with_message() {
    const RATE_LIMIT_202_BODY: &str = r#"{
        "data": null,
        "error": {
            "statusCode": 202,
            "success": false,
            "message": "Unable to make changes whilst on the Basic tier of Bunny Shield. Please upgrade to Advanced to enable Rate Limiting.",
            "errorKey": "invalid_plan_type.rate_limiting"
        }
    }"#;

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/shield/rate-limits/99"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(202).set_body_raw(RATE_LIMIT_202_BODY, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["shield", "rate-limit", "list", "--shield-zone-id", "99"])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "expected non-zero exit code for 202 plan-gate error"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Basic tier"),
        "stderr should contain plan upgrade message; got: {stderr}"
    );
}
