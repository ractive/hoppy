//! Iter-90: Shield enum labels follow the spec. Names (case-insensitive) and
//! numbers both work on the flags; the wire numbers are what the API sees.

use super::support;

use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn ok(output: &std::process::Output, what: &str) {
    assert!(
        output.status.success(),
        "{what}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

async fn access_list_update_config_sends(spelling: &str, wire: u8) {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path(
            "/shield/shield-zone/55001/access-lists/configurations/901",
        ))
        .and(header("AccessKey", "test-api-key"))
        .and(body_string_contains(format!("\"action\":{wire}")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "shield",
            "access-list",
            "update-config",
            "--shield-zone-id",
            "55001",
            "--configuration-id",
            "901",
            "--action",
            spelling,
        ])
        .output()
        .unwrap();
    ok(&output, spelling);
}

/// The headline fix: wire 1 is Allow and wire 2 is Block.
#[tokio::test]
async fn access_list_action_allow_is_1_and_block_is_2() {
    access_list_update_config_sends("allow", 1).await;
    access_list_update_config_sends("Allow", 1).await;
    access_list_update_config_sends("1", 1).await;
    access_list_update_config_sends("block", 2).await;
    access_list_update_config_sends("2", 2).await;
    access_list_update_config_sends("log", 4).await;
    access_list_update_config_sends("bypass", 5).await;
}

#[tokio::test]
async fn access_list_action_rejects_old_labels() {
    let server = MockServer::start().await;
    for old in ["noaction", "logonly", "challengeinterstitial"] {
        let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
            .args([
                "shield",
                "access-list",
                "update-config",
                "--shield-zone-id",
                "55001",
                "--configuration-id",
                "901",
                "--action",
                old,
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{old}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("invalid access list action"),
            "{old}"
        );
    }
}

async fn access_list_create_sends_type(spelling: &str, wire: u8) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/shield/shield-zone/55001/access-lists"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_string_contains(format!("\"type\":{wire}")))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/access_list_create.json"),
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
            "access-list",
            "create",
            "--shield-zone-id",
            "55001",
            "--name",
            "t",
            "--type",
            spelling,
            "--content",
            "203.0.113.0/24",
        ])
        .output()
        .unwrap();
    ok(&output, spelling);
}

#[tokio::test]
async fn access_list_type_cidr_is_1_and_country_is_3() {
    access_list_create_sends_type("cidr", 1).await;
    access_list_create_sends_type("CIDR", 1).await;
    access_list_create_sends_type("1", 1).await;
    access_list_create_sends_type("country", 3).await;
    access_list_create_sends_type("3", 3).await;
    access_list_create_sends_type("ip", 0).await;
    access_list_create_sends_type("ja4", 5).await;
}

#[tokio::test]
async fn access_list_type_rejects_old_labels() {
    let server = MockServer::start().await;
    for old in ["hostname", "useragent", "custom"] {
        let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
            .args([
                "shield",
                "access-list",
                "create",
                "--shield-zone-id",
                "55001",
                "--name",
                "t",
                "--type",
                old,
                "--content",
                "x",
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{old}");
    }
}

async fn waf_add_rule_sends(action: &str, severity: &str, action_wire: u8, severity_wire: u8) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/shield/waf/custom-rule"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_string_contains(format!(
            "\"actionType\":{action_wire}"
        )))
        .and(body_string_contains(format!(
            "\"severityType\":{severity_wire}"
        )))
        .and(body_string_contains("\"transformationTypes\":[8,19]"))
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
            "r",
            "--action-type",
            action,
            "--operator-type",
            "contains",
            "--severity-type",
            severity,
            "--transformation-type",
            "lowercase",
            "--transformation-type",
            "19",
            "--value",
            "x",
        ])
        .output()
        .unwrap();
    ok(&output, &format!("{action}/{severity}"));
}

#[tokio::test]
async fn waf_add_rule_accepts_spec_action_severity_and_transformation_names() {
    waf_add_rule_sends("allow", "critical", 4, 2).await;
    waf_add_rule_sends("4", "2", 4, 2).await;
    waf_add_rule_sends("Bypass", "NOTICE", 5, 0).await;
    waf_add_rule_sends("log", "warning", 2, 1).await;
}

#[tokio::test]
async fn waf_add_rule_rejects_old_severity_and_action_labels() {
    let server = MockServer::start().await;
    for (flag, old) in [
        ("--severity-type", "high"),
        ("--severity-type", "low"),
        ("--action-type", "logonly"),
        ("--action-type", "challengeinterstitial"),
    ] {
        let mut args = vec![
            "shield",
            "waf",
            "add-rule",
            "--shield-zone-id",
            "55001",
            "--operator-type",
            "eq",
        ];
        if flag == "--severity-type" {
            args.extend(["--action-type", "1"]);
        } else {
            args.extend(["--severity-type", "1"]);
        }
        args.extend([flag, old]);
        let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{flag} {old}");
    }
}

#[tokio::test]
async fn rate_limit_create_accepts_spec_names() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/shield/rate-limit"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_string_contains("\"actionType\":1"))
        .and(body_string_contains("\"counterKeyType\":7"))
        .and(body_string_contains("\"severityType\":1"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/rate_limit_rule_create.json"),
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
            "create",
            "--shield-zone-id",
            "55001",
            "--name",
            "rl",
            "--action-type",
            "ratelimit",
            "--operator-type",
            "eq",
            "--severity-type",
            "warning",
            "--request-count",
            "100",
            "--counter-key-type",
            "ip_ja4",
            "--timeframe",
            "60",
            "--block-time",
            "60",
        ])
        .output()
        .unwrap();
    ok(&output, "rate-limit create");
}

#[tokio::test]
async fn zone_update_accepts_waf_execution_mode_and_ddos_sensitivity_names() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/shield/shield-zone"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_string_contains("\"wafExecutionMode\":1"))
        .and(body_string_contains("\"dDoSShieldSensitivity\":4"))
        .respond_with(ResponseTemplate::new(204))
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
            "--id",
            "55001",
            "--waf-execution-mode",
            "block",
            "--ddos-sensitivity",
            "challenge",
        ])
        .output()
        .unwrap();
    ok(&output, "zone update");
}

#[tokio::test]
async fn bot_detection_update_accepts_spec_names() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/shield/shield-zone/55001/bot-detection"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_string_contains("\"executionMode\":1"))
        .and(body_string_contains("\"aggression\":4"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/bot_detection_update.json"),
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
            "bot-detection",
            "update",
            "--id",
            "55001",
            "--execution-mode",
            "challenge",
            "--fingerprint-aggression",
            "custom",
        ])
        .output()
        .unwrap();
    ok(&output, "bot-detection update");
}
