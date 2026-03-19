mod e2e_support;

use e2e_support::{cmd, server, skip_in_live_mode};
use predicates::prelude::*;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn auth_check_shows_billing_info() {
    skip_in_live_mode!();
    let mock = server::start().await;
    let body = include_str!("fixtures/core/billing_get.json");

    Mock::given(method("GET"))
        .and(path("/billing"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["auth", "check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("42.5"));
}

#[tokio::test]
async fn auth_check_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;
    let body = include_str!("fixtures/core/billing_get.json");

    Mock::given(method("GET"))
        .and(path("/billing"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--format", "json", "auth", "check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"Balance\""));
}

#[tokio::test]
async fn auth_check_unauthorized() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/billing"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(401).set_body_string("{\"Message\":\"Unauthorized\"}"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock).args(["auth", "check"]).assert().failure();
}
