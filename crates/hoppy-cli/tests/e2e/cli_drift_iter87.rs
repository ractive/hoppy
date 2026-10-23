//! Iter-87: OpenAPI drift fixes (WAF operator names, forward-compat enums,
//! new enum values). Fixtures here carry the 2026-10-23 spec shapes.

use super::support;

use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn mount_get(server: &MockServer, p: &str, fixture: &str) {
    Mock::given(method("GET"))
        .and(path(p))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(support::fixture(fixture), "application/json"),
        )
        .expect(1)
        .mount(server)
        .await;
}

// ---------------------------------------------------------------------------
// WAF operator names
// ---------------------------------------------------------------------------

async fn add_rule_sends_operator(spelling: &str, wire: u8) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/shield/waf/custom-rule"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_string_contains(format!("\"operatorType\":{wire}")))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/waf_rule_create.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "shield",
            "waf",
            "add-rule",
            "--shield-zone-id",
            "55001",
            "--name",
            "ip rule",
            "--action-type",
            "1",
            "--operator-type",
            spelling,
            "--severity-type",
            "1",
            "--value",
            "10.0.0.0/8",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "--operator-type {spelling}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn waf_add_rule_operator_name_ipmatch_sends_13() {
    add_rule_sends_operator("ipmatch", 13).await;
    add_rule_sends_operator("IPMATCH", 13).await;
}

#[tokio::test]
async fn waf_add_rule_operator_number_13_sends_13() {
    add_rule_sends_operator("13", 13).await;
}

#[tokio::test]
async fn waf_add_rule_operator_rx_and_streq_use_spec_numbers() {
    add_rule_sends_operator("rx", 14).await;
    add_rule_sends_operator("streq", 15).await;
    add_rule_sends_operator("beginswith", 0).await;
}

#[tokio::test]
async fn waf_add_rule_rejects_old_operator_label() {
    let server = MockServer::start().await;
    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "shield",
            "waf",
            "add-rule",
            "--shield-zone-id",
            "55001",
            "--action-type",
            "1",
            "--operator-type",
            "regex",
            "--severity-type",
            "1",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid operator"));
}

#[tokio::test]
async fn waf_list_rules_with_unknown_and_ipmatch_operators_succeeds() {
    let server = MockServer::start().await;
    mount_get(
        &server,
        "/shield/waf/custom-rules/55001",
        "shield/waf_rules_list_unknown_operator.json",
    )
    .await;
    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "shield",
            "waf",
            "list-rules",
            "--shield-zone-id",
            "55001",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn rate_limit_list_with_unknown_operator_succeeds() {
    let server = MockServer::start().await;
    mount_get(
        &server,
        "/shield/rate-limits/55001",
        "shield/rate_limit_rules_list_unknown_operator.json",
    )
    .await;
    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "shield",
            "rate-limit",
            "list",
            "--shield-zone-id",
            "55001",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Read-modify-write must echo an operator this client does not know, and
/// keep `isNegated`, instead of dropping or rewriting them.
#[tokio::test]
async fn waf_update_rule_echoes_unknown_operator_and_is_negated() {
    let server = MockServer::start().await;
    mount_get(
        &server,
        "/shield/waf/custom-rule/7101",
        "shield/waf_rule_get_unknown_operator.json",
    )
    .await;
    Mock::given(method("PATCH"))
        .and(path("/shield/waf/custom-rule/7101"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_string_contains("\"operatorType\":99"))
        .and(body_string_contains("\"isNegated\":true"))
        .and(body_string_contains("\"ruleDescription\":\"renamed\""))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/waf_rule_get_unknown_operator.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "shield",
            "waf",
            "update-rule",
            "--id",
            "7101",
            "--description",
            "renamed",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
