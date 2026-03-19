mod e2e_support;

use e2e_support::{cmd, server, skip_in_live_mode};
use predicates::prelude::*;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, ResponseTemplate};

const FIXTURE_ZONES_LIST: &str = include_str!("fixtures/shield/shield_zones_list.json");
const FIXTURE_ZONE_GET: &str = include_str!("fixtures/shield/shield_zone_get.json");
const FIXTURE_WAF_RULES_LIST: &str = include_str!("fixtures/shield/waf_rules_list.json");
const FIXTURE_WAF_RULE_GET: &str = include_str!("fixtures/shield/waf_rule_get.json");
const FIXTURE_RATE_LIMIT_RULES_LIST: &str =
    include_str!("fixtures/shield/rate_limit_rules_list.json");
const FIXTURE_RATE_LIMIT_RULE_GET: &str = include_str!("fixtures/shield/rate_limit_rule_get.json");
const FIXTURE_ACCESS_LISTS_GET: &str = include_str!("fixtures/shield/access_lists_get.json");
const FIXTURE_ACCESS_LIST_GET: &str = include_str!("fixtures/shield/access_list_get.json");
const FIXTURE_BOT_DETECTION_GET: &str = include_str!("fixtures/shield/bot_detection_get.json");

// ---------------------------------------------------------------------------
// Shield Zone — list
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shield_zone_list_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zones"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ZONES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["shield", "zone", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("55001"))
        .stdout(predicate::str::contains("55002"));
}

#[tokio::test]
async fn shield_zone_list_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zones"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ZONES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--format", "json", "shield", "zone", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"CurrentPage\""))
        .stdout(predicate::str::contains("\"TotalItems\""));
}

// ---------------------------------------------------------------------------
// Shield Zone — get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shield_zone_get_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/55001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_ZONE_GET, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["shield", "zone", "get", "--shield-zone-id", "55001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("55001"))
        .stdout(predicate::str::contains("100001"));
}

#[tokio::test]
async fn shield_zone_get_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/55001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_ZONE_GET, "application/json"))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format",
            "json",
            "shield",
            "zone",
            "get",
            "--shield-zone-id",
            "55001",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"shieldZoneId\""))
        .stdout(predicate::str::contains("55001"));
}

#[tokio::test]
async fn shield_zone_get_not_found() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/99999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_string("{\"Message\":\"Object with the requested ID does not exist.\"}"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["shield", "zone", "get", "--shield-zone-id", "99999"])
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// WAF — list rules
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shield_waf_list_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/waf/custom-rules/55001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_WAF_RULES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["shield", "waf", "list-rules", "--shield-zone-id", "55001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Block SQL Injection"))
        .stdout(predicate::str::contains("Log XSS Attempts"));
}

// ---------------------------------------------------------------------------
// WAF — get rule
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shield_waf_get_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/waf/custom-rule/7001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_WAF_RULE_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["shield", "waf", "get-rule", "--id", "7001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Block SQL Injection"))
        .stdout(predicate::str::contains("55001"));
}

#[tokio::test]
async fn shield_waf_get_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/waf/custom-rule/7001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_WAF_RULE_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format", "json", "shield", "waf", "get-rule", "--id", "7001",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\""))
        .stdout(predicate::str::contains("7001"));
}

// ---------------------------------------------------------------------------
// Rate limit — list
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shield_rate_limit_list_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/rate-limits/55001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_RATE_LIMIT_RULES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["shield", "rate-limit", "list", "--shield-zone-id", "55001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Limit Login Attempts"))
        .stdout(predicate::str::contains("8001"));
}

// ---------------------------------------------------------------------------
// Rate limit — get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shield_rate_limit_get_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/rate-limit/8001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_RATE_LIMIT_RULE_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["shield", "rate-limit", "get", "--id", "8001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Limit Login Attempts"))
        .stdout(predicate::str::contains("8001"));
}

#[tokio::test]
async fn shield_rate_limit_get_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/rate-limit/8001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_RATE_LIMIT_RULE_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format",
            "json",
            "shield",
            "rate-limit",
            "get",
            "--id",
            "8001",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\""))
        .stdout(predicate::str::contains("8001"));
}

// ---------------------------------------------------------------------------
// Access list — list
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shield_access_list_get_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/55001/access-lists"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ACCESS_LISTS_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["shield", "access-list", "list", "--shield-zone-id", "55001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Known Bad IPs"))
        .stdout(predicate::str::contains("Blocked Countries"));
}

// ---------------------------------------------------------------------------
// Access list — get custom
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shield_access_list_get_custom_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/55001/access-lists/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ACCESS_LIST_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "shield",
            "access-list",
            "get",
            "--shield-zone-id",
            "55001",
            "--id",
            "9001",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Blocked Countries"))
        .stdout(predicate::str::contains("9001"));
}

#[tokio::test]
async fn shield_access_list_get_custom_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/55001/access-lists/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ACCESS_LIST_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format",
            "json",
            "shield",
            "access-list",
            "get",
            "--shield-zone-id",
            "55001",
            "--id",
            "9001",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\""))
        .stdout(predicate::str::contains("9001"));
}

// ---------------------------------------------------------------------------
// Bot detection — get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shield_bot_detection_get_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/55001/bot-detection"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_BOT_DETECTION_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "shield",
            "bot-detection",
            "get",
            "--shield-zone-id",
            "55001",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("55001"));
}

#[tokio::test]
async fn shield_bot_detection_get_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/55001/bot-detection"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_BOT_DETECTION_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format",
            "json",
            "shield",
            "bot-detection",
            "get",
            "--shield-zone-id",
            "55001",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"shieldZoneId\""))
        .stdout(predicate::str::contains("55001"));
}
