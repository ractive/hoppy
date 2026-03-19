use bunny_api_shield::ShieldClient;
use bunny_api_shield::types::{
    AccessListType, CreateCustomAccessList, CreateCustomWafRule, CreateRateLimitRule,
    RateLimitCounterKey, RateLimitRuleConfiguration, UpdateAccessListConfiguration,
    UpdateBotDetection, UpdateCustomAccessList, UpdateCustomWafRule, UpdateRateLimitRule,
    UpdateShieldZoneRequest, WafRuleActionType, WafRuleConfiguration, WafRuleOperatorType,
    WafRuleSeverityType,
};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_SHIELD_ZONE_GET: &str = include_str!("fixtures/shield_zone_get.json");
const FIXTURE_SHIELD_ZONES_LIST: &str = include_str!("fixtures/shield_zones_list.json");
const FIXTURE_WAF_RULES_LIST: &str = include_str!("fixtures/waf_rules_list.json");
const FIXTURE_WAF_RULE_GET: &str = include_str!("fixtures/waf_rule_get.json");
const FIXTURE_WAF_RULE_CREATE: &str = include_str!("fixtures/waf_rule_create.json");
const FIXTURE_RATE_LIMIT_RULES_LIST: &str = include_str!("fixtures/rate_limit_rules_list.json");
const FIXTURE_RATE_LIMIT_RULE_GET: &str = include_str!("fixtures/rate_limit_rule_get.json");
const FIXTURE_RATE_LIMIT_RULE_CREATE: &str = include_str!("fixtures/rate_limit_rule_create.json");
const FIXTURE_ACCESS_LISTS_GET: &str = include_str!("fixtures/access_lists_get.json");
const FIXTURE_ACCESS_LIST_GET: &str = include_str!("fixtures/access_list_get.json");
const FIXTURE_ACCESS_LIST_CREATE: &str = include_str!("fixtures/access_list_create.json");
const FIXTURE_BOT_DETECTION_GET: &str = include_str!("fixtures/bot_detection_get.json");
const FIXTURE_BOT_DETECTION_UPDATE: &str = include_str!("fixtures/bot_detection_update.json");
const FIXTURE_WAF_PROFILES_LIST: &str = include_str!("fixtures/waf_profiles_list.json");
const FIXTURE_ERROR_UNAUTHORIZED: &str = include_str!("fixtures/error_unauthorized.json");
const FIXTURE_ERROR_NOT_FOUND: &str = include_str!("fixtures/error_not_found.json");

fn test_client(uri: &str) -> ShieldClient {
    ShieldClient::with_base_url("test-api-key", uri)
}

// ---------------------------------------------------------------------------
// Shield Zones
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_shield_zones_returns_items() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zones"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SHIELD_ZONES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_shield_zones()
        .await
        .unwrap();

    let zones = result.data.unwrap();
    assert_eq!(zones.len(), 2);
    assert_eq!(zones[0].shield_zone_id, 55001);
    assert_eq!(zones[0].pull_zone_id, Some(100001));
    assert_eq!(zones[1].shield_zone_id, 55002);
}

#[tokio::test]
async fn get_shield_zone_returns_zone() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/55001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SHIELD_ZONE_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let zone = test_client(&server.uri())
        .get_shield_zone(55001)
        .await
        .unwrap();

    assert_eq!(zone.shield_zone_id, 55001);
    assert_eq!(zone.pull_zone_id, Some(100001));
    assert_eq!(zone.waf_enabled, Some(true));
}

#[tokio::test]
async fn get_shield_zone_by_pull_zone_returns_zone() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/get-by-pullzone/100001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SHIELD_ZONE_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let zone = test_client(&server.uri())
        .get_shield_zone_by_pull_zone(100001)
        .await
        .unwrap();

    assert_eq!(zone.shield_zone_id, 55001);
    assert_eq!(zone.pull_zone_id, Some(100001));
}

#[tokio::test]
async fn create_shield_zone_returns_zone() {
    let server = MockServer::start().await;

    // The create endpoint returns a nested { data: { shieldZone: {...} } } shape.
    let body = serde_json::json!({
        "data": {
            "shieldZone": {
                "shieldZoneId": 55003,
                "pullZoneId": 100003,
                "wafEnabled": false,
                "rateLimitRulesLimit": 5,
                "customWafRulesLimit": 10
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/shield/shield-zone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let zone = test_client(&server.uri())
        .create_shield_zone(100003)
        .await
        .unwrap();

    assert_eq!(zone.shield_zone_id, 55003);
    assert_eq!(zone.pull_zone_id, Some(100003));
}

#[tokio::test]
async fn update_shield_zone_succeeds() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/shield/shield-zone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateShieldZoneRequest {
        shield_zone_id: 55001,
        shield_zone: None,
    };

    test_client(&server.uri())
        .update_shield_zone(body)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// WAF rules
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_waf_rules_returns_items() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/waf/custom-rules/55001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_WAF_RULES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let rules = test_client(&server.uri())
        .list_waf_rules(55001)
        .await
        .unwrap();

    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].id, 7001);
    assert_eq!(rules[0].rule_name.as_deref(), Some("Block SQL Injection"));
    assert_eq!(rules[1].id, 7002);
}

#[tokio::test]
async fn get_waf_rule_returns_rule() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/waf/custom-rule/7001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_WAF_RULE_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let rule = test_client(&server.uri()).get_waf_rule(7001).await.unwrap();

    assert_eq!(rule.id, 7001);
    assert_eq!(rule.shield_zone_id, 55001);
    assert_eq!(rule.rule_name.as_deref(), Some("Block SQL Injection"));
}

#[tokio::test]
async fn create_waf_rule_returns_created_rule() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/shield/waf/custom-rule"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "shieldZoneId": 55001,
            "ruleName": "Block Bad UA",
            "ruleConfiguration": {
                "actionType": 1,
                "operatorType": 2,
                "severityType": 1,
                "value": "BadBot/1.0"
            }
        })))
        .respond_with(
            ResponseTemplate::new(201).set_body_raw(FIXTURE_WAF_RULE_CREATE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = CreateCustomWafRule {
        shield_zone_id: 55001,
        rule_name: Some("Block Bad UA".to_string()),
        rule_description: None,
        rule_configuration: WafRuleConfiguration {
            action_type: WafRuleActionType::Block,
            variable_types: None,
            operator_type: WafRuleOperatorType::Contains,
            severity_type: WafRuleSeverityType::Medium,
            transformation_types: None,
            value: Some("BadBot/1.0".to_string()),
            chained_rule_conditions: None,
        },
    };

    let rule = test_client(&server.uri())
        .create_waf_rule(body)
        .await
        .unwrap();

    assert_eq!(rule.id, 7003);
    assert_eq!(rule.rule_name.as_deref(), Some("Block Bad UA"));
}

#[tokio::test]
async fn update_waf_rule_returns_updated_rule() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/shield/waf/custom-rule/7001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_WAF_RULE_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateCustomWafRule {
        rule_name: Some("Block SQL Injection".to_string()),
        rule_description: None,
        rule_configuration: None,
    };

    let rule = test_client(&server.uri())
        .update_waf_rule(7001, body)
        .await
        .unwrap();

    assert_eq!(rule.id, 7001);
}

#[tokio::test]
async fn delete_waf_rule_succeeds() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/shield/waf/custom-rule/7001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_waf_rule(7001)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Rate limit rules
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_rate_limit_rules_returns_items() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/rate-limits/55001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_RATE_LIMIT_RULES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let rules = test_client(&server.uri())
        .list_rate_limit_rules(55001)
        .await
        .unwrap();

    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].id, 8001);
    assert_eq!(rules[0].rule_name.as_deref(), Some("Limit Login Attempts"));
}

#[tokio::test]
async fn get_rate_limit_rule_returns_rule() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/rate-limit/8001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_RATE_LIMIT_RULE_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let rule = test_client(&server.uri())
        .get_rate_limit_rule(8001)
        .await
        .unwrap();

    assert_eq!(rule.id, 8001);
    let config = rule.rule_configuration.unwrap();
    assert_eq!(config.request_count, 10);
}

#[tokio::test]
async fn create_rate_limit_rule_returns_created_rule() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/shield/rate-limit"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "shieldZoneId": 55001,
            "ruleName": "API Rate Limit",
            "ruleConfiguration": {
                "actionType": 1,
                "operatorType": 0,
                "severityType": 0,
                "requestCount": 100,
                "counterKeyType": 1,
                "timeframe": 3600,
                "blockTime": 900
            }
        })))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_raw(FIXTURE_RATE_LIMIT_RULE_CREATE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    use bunny_api_shield::types::{
        RateLimitBlockDuration, RateLimitTimeframe, WafRuleOperatorType,
    };

    let body = CreateRateLimitRule {
        shield_zone_id: 55001,
        rule_name: Some("API Rate Limit".to_string()),
        rule_description: None,
        rule_configuration: RateLimitRuleConfiguration {
            action_type: bunny_api_shield::types::RateLimitActionType::Block,
            variable_types: None,
            operator_type: WafRuleOperatorType::Eq,
            severity_type: WafRuleSeverityType::Low,
            transformation_types: None,
            value: None,
            request_count: 100,
            counter_key_type: RateLimitCounterKey::PerIp,
            timeframe: RateLimitTimeframe::Sec3600,
            block_time: RateLimitBlockDuration::Sec900,
            chained_rule_conditions: None,
        },
    };

    let rule = test_client(&server.uri())
        .create_rate_limit_rule(body)
        .await
        .unwrap();

    assert_eq!(rule.id, 8002);
    assert_eq!(rule.rule_name.as_deref(), Some("API Rate Limit"));
}

#[tokio::test]
async fn update_rate_limit_rule_returns_updated_rule() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/shield/rate-limit/8001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_RATE_LIMIT_RULE_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateRateLimitRule {
        rule_name: Some("Limit Login Attempts".to_string()),
        rule_description: None,
        rule_configuration: None,
    };

    let rule = test_client(&server.uri())
        .update_rate_limit_rule(8001, body)
        .await
        .unwrap();

    assert_eq!(rule.id, 8001);
}

#[tokio::test]
async fn delete_rate_limit_rule_succeeds() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/shield/rate-limit/8001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_rate_limit_rule(8001)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Access lists
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_access_lists_returns_managed_and_custom() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/55001/access-lists"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ACCESS_LISTS_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .get_access_lists(55001)
        .await
        .unwrap();

    let managed = result.managed_lists.unwrap();
    let custom = result.custom_lists.unwrap();
    assert_eq!(managed.len(), 1);
    assert_eq!(custom.len(), 1);
    assert_eq!(custom[0].list_id, 9001);
}

#[tokio::test]
async fn get_custom_access_list_returns_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/55001/access-lists/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ACCESS_LIST_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let list = test_client(&server.uri())
        .get_custom_access_list(55001, 9001)
        .await
        .unwrap();

    assert_eq!(list.id, 9001);
    assert_eq!(list.name.as_deref(), Some("Blocked Countries"));
    assert_eq!(list.list_type, AccessListType::Country);
}

#[tokio::test]
async fn create_access_list_returns_created_list() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/shield/shield-zone/55001/access-lists"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(201).set_body_raw(FIXTURE_ACCESS_LIST_CREATE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = CreateCustomAccessList {
        name: "Allowed IPs".to_string(),
        description: None,
        list_type: AccessListType::Ip,
        content: "192.168.1.1\n10.0.0.1".to_string(),
        checksum: None,
    };

    let list = test_client(&server.uri())
        .create_access_list(55001, body)
        .await
        .unwrap();

    assert_eq!(list.id, 9002);
    assert_eq!(list.name.as_deref(), Some("Allowed IPs"));
}

#[tokio::test]
async fn update_access_list_returns_updated_list() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/shield/shield-zone/55001/access-lists/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ACCESS_LIST_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateCustomAccessList {
        name: None,
        content: Some("CN\nRU\nKP".to_string()),
        checksum: None,
    };

    let list = test_client(&server.uri())
        .update_custom_access_list(55001, 9001, body)
        .await
        .unwrap();

    assert_eq!(list.id, 9001);
}

#[tokio::test]
async fn delete_access_list_succeeds() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/shield/shield-zone/55001/access-lists/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_access_list(55001, 9001)
        .await
        .unwrap();
}

#[tokio::test]
async fn update_access_list_config_succeeds() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path(
            "/shield/shield-zone/55001/access-lists/configurations/901",
        ))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateAccessListConfiguration {
        is_enabled: Some(true),
        action: None,
    };

    test_client(&server.uri())
        .update_access_list_configuration(55001, 901, body)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Bot detection
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_bot_detection_returns_config() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/55001/bot-detection"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_BOT_DETECTION_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .get_bot_detection(55001)
        .await
        .unwrap();

    let state = result.data.unwrap();
    assert_eq!(state.shield_zone_id, 55001);
    assert!(state.execution_mode.is_some());
}

#[tokio::test]
async fn update_bot_detection_returns_updated_config() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/shield/shield-zone/55001/bot-detection"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_BOT_DETECTION_UPDATE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateBotDetection {
        shield_zone_id: 55001,
        execution_mode: None,
        request_integrity: None,
        ip_address: None,
        browser_fingerprint: None,
    };

    let result = test_client(&server.uri())
        .update_bot_detection(55001, body)
        .await
        .unwrap();

    let state = result.data.unwrap();
    assert_eq!(state.shield_zone_id, 55001);
}

// ---------------------------------------------------------------------------
// WAF profiles
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_waf_profiles_flattens_nested_groups() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/waf/profiles"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_WAF_PROFILES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let profiles = test_client(&server.uri())
        .list_waf_profiles()
        .await
        .unwrap();

    // Two groups, each with one profile — should be flattened to 2
    assert_eq!(profiles.len(), 2);
    assert_eq!(profiles[0].id, 1);
    assert_eq!(profiles[0].name.as_deref(), Some("OWASP Core Rule Set"));
    assert!(!profiles[0].is_premium);
    assert!(profiles[1].is_premium);
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unauthorized_error_contains_status_code() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/55001"))
        .respond_with(
            ResponseTemplate::new(401).set_body_raw(FIXTURE_ERROR_UNAUTHORIZED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .get_shield_zone(55001)
        .await
        .unwrap_err();

    let msg = err.to_string();
    assert!(msg.contains("401"), "expected 401 in error: {msg}");
}

#[tokio::test]
async fn not_found_error_contains_status_code() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/99999"))
        .respond_with(
            ResponseTemplate::new(404).set_body_raw(FIXTURE_ERROR_NOT_FOUND, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .get_shield_zone(99999)
        .await
        .unwrap_err();

    let msg = err.to_string();
    assert!(msg.contains("404"), "expected 404 in error: {msg}");
}

// ---------------------------------------------------------------------------
// Debug mode
// ---------------------------------------------------------------------------

#[tokio::test]
async fn debug_client_works_without_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zones"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SHIELD_ZONES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    // with_debug(true) should not change behaviour — just emit to stderr
    let result = ShieldClient::with_base_url("test-api-key", server.uri())
        .with_debug(true)
        .list_shield_zones()
        .await
        .unwrap();

    assert!(!result.data.unwrap().is_empty());
}
