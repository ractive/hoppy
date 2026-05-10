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
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
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
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
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
