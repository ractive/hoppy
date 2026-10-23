//! Iter-87: OpenAPI drift fixes (WAF operator names, forward-compat enums,
//! new enum values). Fixtures here carry the 2026-10-23 spec shapes.

use super::support;

use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn mount_get(server: &MockServer, p: &str, fixture: &str) {
    mount_get_n(server, p, fixture, 1).await;
}

async fn mount_get_n(server: &MockServer, p: &str, fixture: &str, times: u64) {
    Mock::given(method("GET"))
        .and(path(p))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(support::fixture(fixture), "application/json"),
        )
        .expect(times)
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

// ---------------------------------------------------------------------------
// New enum values (task 3)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shield_zone_get_and_list_accept_sensitivity_5_and_unknown_plan() {
    let server = MockServer::start().await;
    mount_get(
        &server,
        "/shield/shield-zone/55002",
        "shield/shield_zone_get_new_enums.json",
    )
    .await;
    mount_get(
        &server,
        "/shield/shield-zones",
        "shield/shield_zones_list_new_enums.json",
    )
    .await;
    let get = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "shield", "zone", "get", "--id", "55002"])
        .output()
        .unwrap();
    assert!(
        get.status.success(),
        "{}",
        String::from_utf8_lossy(&get.stderr)
    );
    // The raw values must survive a round trip through the typed response.
    let stdout = String::from_utf8_lossy(&get.stdout);
    assert!(
        stdout.contains("\"dDoSShieldSensitivity\": 5"),
        "got: {stdout}"
    );
    assert!(stdout.contains("\"planType\": 7"), "got: {stdout}");

    let list = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "shield", "zone", "list"])
        .output()
        .unwrap();
    assert!(
        list.status.success(),
        "{}",
        String::from_utf8_lossy(&list.stderr)
    );
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(stdout.contains("ChallengeInteractive"), "got: {stdout}");
    assert!(stdout.contains("Unknown(7)"), "got: {stdout}");
}

#[tokio::test]
async fn shield_bot_categorization_list_renders_ignore_and_unknown_actions() {
    let server = MockServer::start().await;
    mount_get(
        &server,
        "/shield/shield-zone/42/bot-categorization",
        "shield/bot_categorization_new_actions.json",
    )
    .await;
    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "table",
            "shield",
            "bot-categorization",
            "list",
            "--id",
            "42",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Ignore"), "got: {stdout}");
    assert!(stdout.contains("Unknown(9)"), "got: {stdout}");
}

#[tokio::test]
async fn shield_bot_categorization_set_category_rejects_ignore_locally() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "shield",
            "bot-categorization",
            "set-category",
            "--id",
            "42",
            "--category",
            "2",
            "--action",
            "3",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
}

#[tokio::test]
async fn shield_bot_categorization_set_bot_accepts_ignore() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/shield/shield-zone/42/bot-categorization/bots/7"))
        .and(body_string_contains("\"action\":3"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"data":{"botId":7,"category":2,"action":3,"isVerifiable":true}}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;
    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "shield",
            "bot-categorization",
            "set-bot",
            "--id",
            "42",
            "--bot-id",
            "7",
            "--action",
            "3",
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
async fn container_volume_list_accepts_maintenance_and_unknown_status() {
    let server = MockServer::start().await;
    mount_get_n(
        &server,
        "/apps/test-app-id/volumes",
        "containers/volumes_list_maintenance.json",
        2,
    )
    .await;
    for format in ["json", "table"] {
        let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
            .args([
                "--format",
                format,
                "container",
                "volume",
                "list",
                "--app-id",
                "test-app-id",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{format}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if format == "json" {
            assert!(String::from_utf8_lossy(&output.stdout).contains("maintenance"));
        }
    }
}

// ---------------------------------------------------------------------------
// Pull-zone execution phase (task 5)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pull_zone_update_pre_cache_phase_sends_2() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_string_contains("\"EdgeScriptExecutionPhase\":2"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;
    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "pull-zone",
            "update",
            "--id",
            "1001",
            "--edge-script-execution-phase",
            "pre-cache",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------------
// Deprecated --ddos-execution-mode (task 5)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shield_zone_update_ddos_execution_mode_is_warned_and_not_sent() {
    use wiremock::matchers::body_string_contains;
    let server = MockServer::start().await;
    // Matches only when the sensitivity is sent; a second, stricter mock
    // catches the deprecated field leaking into the body.
    Mock::given(method("PATCH"))
        .and(path("/shield/shield-zone"))
        .and(body_string_contains("dDoSExecutionMode"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/shield/shield-zone"))
        .and(body_string_contains("\"dDoSShieldSensitivity\":5"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/shield_zone_get.json"),
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
            "zone",
            "update",
            "--shield-zone-id",
            "55001",
            "--ddos-sensitivity",
            "5",
            "--ddos-execution-mode",
            "1",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("deprecated"));
}

// ---------------------------------------------------------------------------
// Edge-rule actions 35-37 (task 6)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_rule_list_shows_actions_35_to_37_by_name_and_unknown_raw() {
    let server = MockServer::start().await;
    mount_get(
        &server,
        "/pullzone/1001",
        "core/pullzone_get_edgerules_new_actions.json",
    )
    .await;
    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "table",
            "pull-zone",
            "edge-rule",
            "list",
            "--id",
            "1001",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    for needle in [
        "use-load-balancer",
        "force-load-balancer-sticky-session-mode",
        "strip-cookies-client-to-origin",
        "Unknown(99)",
    ] {
        assert!(stdout.contains(needle), "missing {needle}: {stdout}");
    }
}

/// `rate-limit update` must echo an operator this client does not know, and
/// keep `isNegated`, just like `waf update-rule`.
#[tokio::test]
async fn rate_limit_update_echoes_unknown_operator_and_is_negated() {
    let server = MockServer::start().await;
    mount_get(
        &server,
        "/shield/rate-limit/8201",
        "shield/rate_limit_rule_get_unknown_operator.json",
    )
    .await;
    Mock::given(method("PATCH"))
        .and(path("/shield/rate-limit/8201"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_string_contains("\"operatorType\":99"))
        .and(body_string_contains("\"isNegated\":true"))
        .and(body_string_contains("\"ruleDescription\":\"renamed\""))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/rate_limit_rule_get_unknown_operator.json"),
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
            "rate-limit",
            "update",
            "--id",
            "8201",
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

// ---------------------------------------------------------------------------
// --config-json strictness for chained conditions
// ---------------------------------------------------------------------------

/// An unknown chained operator in `--config-json` must be rejected before any
/// request reaches the API.
async fn assert_unknown_chained_operator_rejected(operator: u8, args: &[&str]) {
    let dir = tempfile::tempdir().unwrap();
    let cfg_path = dir.path().join("cfg.json");
    std::fs::write(
        &cfg_path,
        format!(r#"{{"chainedRuleConditions":[{{"operatorType":{operator},"value":"x"}}]}}"#),
    )
    .unwrap();

    let server = MockServer::start().await;
    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(args)
        .args(["--config-json", cfg_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success(), "unknown operator was accepted");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("chainedRuleConditions operatorType"),
        "unexpected error: {stderr}"
    );
    assert!(
        server.received_requests().await.unwrap().is_empty(),
        "a request was sent despite the invalid config"
    );
}

#[tokio::test]
async fn waf_add_rule_rejects_unknown_chained_operator() {
    for operator in [10u8, 11, 16, 99] {
        assert_unknown_chained_operator_rejected(
            operator,
            &[
                "--format",
                "json",
                "shield",
                "waf",
                "add-rule",
                "--shield-zone-id",
                "55001",
                "--name",
                "chained",
                "--action-type",
                "1",
                "--operator-type",
                "0",
                "--severity-type",
                "1",
                "--value",
                "x",
            ],
        )
        .await;
    }
}

#[tokio::test]
async fn rate_limit_create_rejects_unknown_chained_operator() {
    assert_unknown_chained_operator_rejected(
        99,
        &[
            "--format",
            "json",
            "shield",
            "rate-limit",
            "create",
            "--shield-zone-id",
            "55001",
            "--name",
            "chained",
            "--action-type",
            "1",
            "--operator-type",
            "0",
            "--severity-type",
            "1",
            "--value",
            "/api",
            "--request-count",
            "100",
            "--counter-key-type",
            "1",
            "--timeframe",
            "60",
            "--block-time",
            "60",
        ],
    )
    .await;
}
