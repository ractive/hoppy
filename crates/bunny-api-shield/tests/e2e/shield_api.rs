use bunny_api_shield::ShieldClient;
use bunny_api_shield::types::{
    AccessListType, CreateCustomAccessList, CreateCustomWafRule, CreateRateLimitRule,
    RateLimitCounterKey, RateLimitRuleConfiguration, ReviewActionType,
    UpdateAccessListConfiguration, UpdateApiGuardianEndpointRequest, UpdateApiGuardianRequest,
    UpdateBotDetection, UpdateCustomAccessList, UpdateCustomWafRule, UpdateRateLimitRule,
    UpdateReviewTriggeredRuleRequest, UpdateShieldZoneRequest,
    UpdateUploadScanningConfigurationRequest, UploadOpenApiSpecificationRequest,
    UploadScanningScannerMode, WafRuleActionType, WafRuleConfiguration, WafRuleOperatorType,
    WafRuleSeverityType,
};
use bunny_api_shield::types::{
    ShieldBotDetectionMetricsResponse, ShieldDetailedMetricsResponse, ShieldMetricsResponse,
    ShieldRateLimitMetricsResponse, ShieldRateLimitsMetricsResponse,
    ShieldUploadScanningMetricsResponse, ShieldWafRuleMetricsResponse,
};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_SHIELD_ZONE_GET: &str =
    include_str!("../../../../fixtures/shield/shield_zone_get.json");
const FIXTURE_SHIELD_ZONES_LIST: &str =
    include_str!("../../../../fixtures/shield/shield_zones_list.json");
const FIXTURE_WAF_RULES_LIST: &str =
    include_str!("../../../../fixtures/shield/waf_rules_list.json");
const FIXTURE_WAF_RULE_GET: &str = include_str!("../../../../fixtures/shield/waf_rule_get.json");
const FIXTURE_WAF_RULE_CREATE: &str =
    include_str!("../../../../fixtures/shield/waf_rule_create.json");
const FIXTURE_RATE_LIMIT_RULES_LIST: &str =
    include_str!("../../../../fixtures/shield/rate_limit_rules_list.json");
const FIXTURE_RATE_LIMIT_RULE_GET: &str =
    include_str!("../../../../fixtures/shield/rate_limit_rule_get.json");
const FIXTURE_RATE_LIMIT_RULE_CREATE: &str =
    include_str!("../../../../fixtures/shield/rate_limit_rule_create.json");
const FIXTURE_ACCESS_LISTS_GET: &str =
    include_str!("../../../../fixtures/shield/access_lists_get.json");
const FIXTURE_ACCESS_LIST_GET: &str =
    include_str!("../../../../fixtures/shield/access_list_get.json");
const FIXTURE_ACCESS_LIST_CREATE: &str =
    include_str!("../../../../fixtures/shield/access_list_create.json");
const FIXTURE_BOT_DETECTION_GET: &str =
    include_str!("../../../../fixtures/shield/bot_detection_get.json");
const FIXTURE_BOT_DETECTION_UPDATE: &str =
    include_str!("../../../../fixtures/shield/bot_detection_update.json");
const FIXTURE_WAF_PROFILES_LIST: &str =
    include_str!("../../../../fixtures/shield/waf_profiles_list.json");
const FIXTURE_ERROR_UNAUTHORIZED: &str =
    include_str!("../../../../fixtures/shield/error_unauthorized.json");
const FIXTURE_ERROR_NOT_FOUND: &str =
    include_str!("../../../../fixtures/shield/error_not_found.json");
const FIXTURE_METRICS_OVERVIEW: &str =
    include_str!("../../../../fixtures/shield/metrics_overview.json");
const FIXTURE_METRICS_OVERVIEW_DETAILED: &str =
    include_str!("../../../../fixtures/shield/metrics_overview_detailed.json");
const FIXTURE_METRICS_RATE_LIMITS: &str =
    include_str!("../../../../fixtures/shield/metrics_rate_limits.json");
const FIXTURE_METRICS_RATE_LIMIT: &str =
    include_str!("../../../../fixtures/shield/metrics_rate_limit.json");
const FIXTURE_METRICS_WAF_RULE: &str =
    include_str!("../../../../fixtures/shield/metrics_waf_rule.json");
const FIXTURE_METRICS_BOT_DETECTION: &str =
    include_str!("../../../../fixtures/shield/metrics_bot_detection.json");
const FIXTURE_METRICS_UPLOAD_SCANNING: &str =
    include_str!("../../../../fixtures/shield/metrics_upload_scanning.json");
const FIXTURE_API_GUARDIAN_GET: &str =
    include_str!("../../../../fixtures/shield/api_guardian_get.json");
const FIXTURE_API_GUARDIAN_UPLOAD: &str =
    include_str!("../../../../fixtures/shield/api_guardian_upload.json");
const FIXTURE_API_GUARDIAN_UPDATE: &str =
    include_str!("../../../../fixtures/shield/api_guardian_update.json");
const FIXTURE_API_GUARDIAN_ENDPOINT_UPDATE: &str =
    include_str!("../../../../fixtures/shield/api_guardian_endpoint_update.json");
const FIXTURE_UPLOAD_SCANNING_GET: &str =
    include_str!("../../../../fixtures/shield/upload_scanning_get.json");
const FIXTURE_UPLOAD_SCANNING_UPDATE: &str =
    include_str!("../../../../fixtures/shield/upload_scanning_update.json");
const FIXTURE_EVENT_LOGS: &str = include_str!("../../../../fixtures/shield/event_logs.json");
const FIXTURE_WAF_TRIGGERED_RULES: &str =
    include_str!("../../../../fixtures/shield/waf_triggered_rules.json");
const FIXTURE_WAF_TRIGGERED_REVIEW: &str =
    include_str!("../../../../fixtures/shield/waf_triggered_review.json");
const FIXTURE_WAF_RECOMMENDATION: &str =
    include_str!("../../../../fixtures/shield/waf_recommendation.json");
const FIXTURE_WAF_PLAN_SEGMENTATION: &str =
    include_str!("../../../../fixtures/shield/waf_plan_segmentation.json");
const FIXTURE_WAF_ENGINE_CONFIG: &str =
    include_str!("../../../../fixtures/shield/waf_engine_config.json");
const FIXTURE_DDOS_ENUMS: &str = include_str!("../../../../fixtures/shield/ddos_enums.json");
const FIXTURE_PULLZONE_MAPPING: &str =
    include_str!("../../../../fixtures/shield/pullzone_mapping.json");
const FIXTURE_ACCESS_LIST_ENUMS: &str =
    include_str!("../../../../fixtures/shield/access_list_enums.json");

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

// ---------------------------------------------------------------------------
// Metrics
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_metrics_overview_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/metrics/overview/55001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_METRICS_OVERVIEW, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result: ShieldMetricsResponse = test_client(&server.uri())
        .get_metrics_overview(55001)
        .await
        .unwrap();

    let data = result.data.unwrap();
    let overview = data.overview.unwrap();
    assert_eq!(overview.d_do_s_mitigated, 142);
    assert_eq!(overview.waf_triggered_rules, 87);
    assert_eq!(overview.ratelimit_breaches, 23);
    assert_eq!(overview.bot_detection_challenged, 456);
    assert_eq!(data.total_billable_requests, Some(245678));
}

#[tokio::test]
async fn get_metrics_detailed_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/metrics/overview/55001/detailed"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_METRICS_OVERVIEW_DETAILED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result: ShieldDetailedMetricsResponse = test_client(&server.uri())
        .get_metrics_detailed(55001)
        .await
        .unwrap();

    let data = result.data.unwrap();
    let waf = data.waf.unwrap();
    let totals = waf.totals.unwrap();
    assert_eq!(totals.blocked_requests, 54);
    assert_eq!(totals.logged_requests, 28);
    assert_eq!(totals.challenged_requests, 5);

    let ddos = data.ddos.unwrap();
    let ddos_totals = ddos.totals.unwrap();
    assert_eq!(ddos_totals.blocked_requests, 95);
    assert_eq!(ddos_totals.verified_requests, 170);

    let rl = data.rate_limit.unwrap();
    let rl_totals = rl.totals.unwrap();
    assert_eq!(rl_totals.total_breaches, 23);

    assert_eq!(data.total_billable_requests_this_month, Some(245678));
    assert_eq!(data.resolution, Some(3));
}

#[tokio::test]
async fn get_metrics_rate_limits_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/metrics/rate-limits/55001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_METRICS_RATE_LIMITS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result: ShieldRateLimitsMetricsResponse = test_client(&server.uri())
        .get_metrics_rate_limits(55001)
        .await
        .unwrap();

    let data = result.data.unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0].ratelimit_id, Some(8001));
    let overview = data[0].overview.as_ref().unwrap();
    assert_eq!(overview.total_breaches, 15);
    assert_eq!(overview.blocked_breaches, 7);
}

#[tokio::test]
async fn get_metrics_rate_limit_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/metrics/rate-limit/8001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_METRICS_RATE_LIMIT, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result: ShieldRateLimitMetricsResponse = test_client(&server.uri())
        .get_metrics_rate_limit(8001)
        .await
        .unwrap();

    let data = result.data.unwrap();
    assert_eq!(data.ratelimit_id, Some(8001));
    let overview = data.overview.unwrap();
    assert_eq!(overview.total_breaches, 15);
    assert_eq!(overview.blocked_breaches, 7);
}

#[tokio::test]
async fn get_metrics_waf_rule_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/metrics/shield-zone/55001/waf-rule/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_METRICS_WAF_RULE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result: ShieldWafRuleMetricsResponse = test_client(&server.uri())
        .get_metrics_waf_rule(55001, 9001)
        .await
        .unwrap();

    let data = result.data.unwrap();
    assert_eq!(data.total_triggers, 87);
    assert_eq!(data.blocked_requests, 54);
    assert_eq!(data.logged_requests, 28);
    assert_eq!(data.challenged_requests, 5);
}

#[tokio::test]
async fn get_metrics_bot_detection_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/metrics/shield-zone/55001/bot-detection"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_METRICS_BOT_DETECTION, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result: ShieldBotDetectionMetricsResponse = test_client(&server.uri())
        .get_metrics_bot_detection(55001)
        .await
        .unwrap();

    let data = result.data.unwrap();
    assert_eq!(data.total_logged_requests, 234);
    assert_eq!(data.total_challenged_requests, 456);
    let history = data.overview_past_twenty_eight_days.unwrap();
    assert_eq!(history.len(), 5);
}

#[tokio::test]
async fn get_metrics_upload_scanning_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/metrics/shield-zone/55001/upload-scanning"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_METRICS_UPLOAD_SCANNING, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result: ShieldUploadScanningMetricsResponse = test_client(&server.uri())
        .get_metrics_upload_scanning(55001)
        .await
        .unwrap();

    let data = result.data.unwrap();
    assert_eq!(data.total_logged_requests, 6);
    assert_eq!(data.total_blocked_requests, 3);
    assert_eq!(data.total_files_scanned, 155);
    let history = data.overview_past_twenty_eight_days.unwrap();
    assert_eq!(history.len(), 3);
}

// ---------------------------------------------------------------------------
// API Guardian
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_api_guardian_returns_endpoints() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/42/api-guardian"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_API_GUARDIAN_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .get_api_guardian(42)
        .await
        .unwrap();

    let data = result.data.unwrap();
    let endpoints = data.endpoints.unwrap();
    assert_eq!(endpoints.len(), 1);
    assert_eq!(endpoints[0].api_guardian_endpoint_id, Some(1001));
    assert_eq!(endpoints[0].request_path.as_deref(), Some("/api/v1/users"));
    assert_eq!(endpoints[0].enabled, Some(true));
}

#[tokio::test]
async fn upload_api_guardian_spec_returns_endpoints() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/shield/shield-zone/42/api-guardian"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_API_GUARDIAN_UPLOAD, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = UploadOpenApiSpecificationRequest {
        content: Some("openapi: 3.0.0".to_string()),
        enforce_authorisation_validation: Some(false),
    };
    let result = test_client(&server.uri())
        .upload_api_guardian_spec(42, body)
        .await
        .unwrap();

    let data = result.data.unwrap();
    let endpoints = data.endpoints.unwrap();
    assert_eq!(endpoints.len(), 1);
    assert_eq!(endpoints[0].api_guardian_endpoint_id, Some(2001));
}

#[tokio::test]
async fn update_api_guardian_returns_endpoints() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/shield/shield-zone/42/api-guardian"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_API_GUARDIAN_UPDATE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateApiGuardianRequest {
        content: "openapi: 3.0.0\ninfo:\n  version: '1.1.0'".to_string(),
        enforce_authorisation_validation: Some(true),
    };
    let result = test_client(&server.uri())
        .update_api_guardian(42, body)
        .await
        .unwrap();

    let data = result.data.unwrap();
    let endpoints = data.endpoints.unwrap();
    assert_eq!(endpoints.len(), 1);
    assert_eq!(endpoints[0].validate_response_body_schema, Some(true));
}

#[tokio::test]
async fn update_api_guardian_endpoint_returns_updated_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/shield/shield-zone/42/api-guardian/endpoint/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_API_GUARDIAN_ENDPOINT_UPDATE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateApiGuardianEndpointRequest {
        enabled: Some(false),
        validate_request_body_schema: None,
        validate_response_body_schema: None,
        validate_authorization: None,
    };
    let result = test_client(&server.uri())
        .update_api_guardian_endpoint(42, 1001, body)
        .await
        .unwrap();

    let endpoint = result.data.unwrap();
    assert_eq!(endpoint.api_guardian_endpoint_id, Some(1001));
    assert_eq!(endpoint.enabled, Some(false));
}

// ---------------------------------------------------------------------------
// Upload Scanning
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_upload_scanning_returns_config() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/42/upload-scanning"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_UPLOAD_SCANNING_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .get_upload_scanning(42)
        .await
        .unwrap();

    let data = result.data.unwrap();
    assert_eq!(data.shield_zone_id, Some(42));
    assert_eq!(data.is_enabled, Some(true));
    assert_eq!(
        data.csam_scanning_mode,
        Some(UploadScanningScannerMode::Block)
    );
    assert_eq!(
        data.antivirus_scanning_mode,
        Some(UploadScanningScannerMode::LogOnly)
    );
}

#[tokio::test]
async fn update_upload_scanning_returns_updated_config() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/shield/shield-zone/42/upload-scanning"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_UPLOAD_SCANNING_UPDATE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateUploadScanningConfigurationRequest {
        shield_zone_id: 42,
        is_enabled: Some(true),
        antivirus_scanning_mode: Some(UploadScanningScannerMode::Block),
        csam_scanning_mode: None,
    };
    let result = test_client(&server.uri())
        .update_upload_scanning(42, body)
        .await
        .unwrap();

    let data = result.data.unwrap();
    assert_eq!(
        data.antivirus_scanning_mode,
        Some(UploadScanningScannerMode::Block)
    );
}

// ---------------------------------------------------------------------------
// Event Logs
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_event_logs_returns_logs() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/event-logs/42/05-01-2025/"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_EVENT_LOGS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .get_event_logs(42, "05-01-2025", "")
        .await
        .unwrap();

    let logs = result.logs.unwrap();
    assert_eq!(logs.len(), 2);
    assert_eq!(logs[0].log_id.as_deref(), Some("log-abc-123"));
    let labels = logs[0].labels.as_ref().unwrap();
    assert_eq!(labels.rule_id.as_deref(), Some("941100"));
    assert_eq!(labels.method.as_deref(), Some("POST"));
    assert_eq!(result.has_more_data, Some(false));
}

// ---------------------------------------------------------------------------
// WAF Triggered Rules
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_triggered_waf_rules_returns_rules() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/waf/rules/review-triggered/42"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_WAF_TRIGGERED_RULES, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .get_triggered_waf_rules(42)
        .await
        .unwrap();

    let rules = result.triggered_rules.unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].rule_id.as_deref(), Some("941100"));
    assert_eq!(rules[0].total_triggered_requests, Some(23));
    assert_eq!(result.total_triggered_rules, Some(2));
}

#[tokio::test]
async fn review_triggered_waf_rule_returns_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/shield/waf/rules/review-triggered/42"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_WAF_TRIGGERED_REVIEW, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateReviewTriggeredRuleRequest {
        rule_id: Some("941100".to_string()),
        action: ReviewActionType::Approve,
    };
    let result = test_client(&server.uri())
        .review_triggered_waf_rule(42, body)
        .await
        .unwrap();

    assert_eq!(result.success, Some(true));
}

#[tokio::test]
async fn get_triggered_waf_rule_recommendation_returns_text() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/shield/waf/rules/review-triggered/ai-recommendation/42/941100",
        ))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_WAF_RECOMMENDATION, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .get_triggered_waf_rule_recommendation(42, "941100")
        .await
        .unwrap();

    assert_eq!(result.rule_id.as_deref(), Some("941100"));
    assert_eq!(result.success, Some(true));
    assert!(result.recommendation.is_some());
}

// ---------------------------------------------------------------------------
// Supplementary endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_waf_plan_segmentation_returns_plans() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/waf/rules/plan-segmentation"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_WAF_PLAN_SEGMENTATION, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .get_waf_plan_segmentation()
        .await
        .unwrap();

    let data = result.data.unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0].plan_name.as_deref(), Some("Basic"));
}

#[tokio::test]
async fn get_waf_engine_config_returns_variables() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/waf/engine-config"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_WAF_ENGINE_CONFIG, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .get_waf_engine_config()
        .await
        .unwrap();

    let data = result.data.unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0].name.as_deref(), Some("tx.allowed_methods"));
}

#[tokio::test]
async fn get_ddos_enums_returns_enum_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/ddos/enums"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_DDOS_ENUMS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri()).get_ddos_enums().await.unwrap();

    let data = result.data.unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0].enum_name.as_deref(), Some("DDoSShieldSensitivity"));
    let values = data[0].enum_values.as_ref().unwrap();
    assert_eq!(values.len(), 4);
}

#[tokio::test]
async fn get_shield_zones_pullzone_mapping_returns_mappings() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zones/pullzone-mapping"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_PULLZONE_MAPPING, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .get_shield_zones_pullzone_mapping()
        .await
        .unwrap();

    let data = result.data.unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0].shield_zone_id, Some(55001));
    assert_eq!(data[0].pull_zone_id, Some(100001));
}

#[tokio::test]
async fn get_access_list_enums_returns_map() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/42/access-lists/enums"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ACCESS_LIST_ENUMS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .get_access_list_enums(42)
        .await
        .unwrap();

    assert!(result.contains_key("AccessListAction"));
    assert!(result.contains_key("AccessListType"));
    let action_map = &result["AccessListAction"];
    assert_eq!(action_map.get("Block").map(String::as_str), Some("1"));
}

#[tokio::test]
async fn get_promo_state_handles_empty_body() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/promo/state"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri()).get_promo_state().await.unwrap();
    assert_eq!(result, serde_json::Value::Null);
}

#[tokio::test]
async fn get_promo_state_decodes_json_body() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/promo/state"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(r#"{"active":true}"#, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri()).get_promo_state().await.unwrap();
    assert_eq!(result["active"], serde_json::Value::Bool(true));
}

#[tokio::test]
async fn get_promo_state_propagates_problem_details() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/shield/promo/state"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(500).set_body_raw(
            r#"{"title":"Internal Error","status":500,"detail":"boom"}"#,
            "application/problem+json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .get_promo_state()
        .await
        .unwrap_err();
    let msg = format!("{err:#}");
    assert!(
        msg.contains("Internal Error") || msg.contains("boom"),
        "expected ProblemDetails error, got: {msg}"
    );
}
