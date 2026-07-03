use bunny_net_api::core::types::{
    AddOrUpdateEdgeRule, EdgeRuleActionType, EdgeRuleTrigger, LogAnonymizationType, MatchingType,
    OptimizerWatermarkPosition, OriginType, PermaCacheType, PullZoneLogForwarderProtocolType,
    PullZoneTierType, StickySessionType, TriggerType, UpdatePullZone,
};
use bunny_net_api::core::{ApiError, CoreClient};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_LIST_PAGINATED: &str =
    include_str!("../../../../../fixtures/core/pullzone_list_paginated.json");
const FIXTURE_GET: &str = include_str!("../../../../../fixtures/core/pullzone_get.json");
const FIXTURE_GET_WITH_OPTIMIZER: &str =
    include_str!("../../../../../fixtures/core/pullzone_get_with_optimizer.json");
const FIXTURE_GET_WITH_EDGERULES: &str =
    include_str!("../../../../../fixtures/core/pullzone_get_with_edgerules.json");
const FIXTURE_GET_MAGIC_CONTAINER: &str =
    include_str!("../../../../../fixtures/core/pullzone_get_magic_container.json");
const FIXTURE_UNAUTHORIZED: &str =
    include_str!("../../../../../fixtures/core/error_unauthorized.json");
const FIXTURE_NOT_FOUND: &str =
    include_str!("../../../../../fixtures/core/error_not_found_storagezone.json");
const FIXTURE_OPTIMIZER_STATS: &str =
    include_str!("../../../../../fixtures/core/pullzone_optimizer_statistics.json");
const FIXTURE_ORIGINSHIELD_STATS: &str =
    include_str!("../../../../../fixtures/core/pullzone_originshield_statistics.json");
const FIXTURE_SAFEHOP_STATS: &str =
    include_str!("../../../../../fixtures/core/pullzone_safehop_statistics.json");

fn test_client(uri: &str) -> CoreClient {
    CoreClient::with_base_url("test-api-key", uri)
}

#[tokio::test]
async fn list_pull_zones_returns_paginated_items() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_PAGINATED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_pull_zones(None, None, None)
        .await
        .unwrap();

    // Shape/invariant checks — specific counts drift with fixture refreshes.
    assert!(!result.items.is_empty());
    assert!(result.total_items >= 1);
    assert!(!result.has_more_items);
    assert!(result.current_page >= 1);

    // JSON-key presence: confirm the fields are actually in the response and
    // not silently defaulted by serde.
    let json: serde_json::Value = serde_json::from_str(FIXTURE_LIST_PAGINATED).unwrap();
    assert!(
        json["TotalItems"].is_number(),
        "TotalItems key missing or not a number"
    );
    assert!(
        json["CurrentPage"].is_number(),
        "CurrentPage key missing or not a number"
    );
    assert!(
        json["HasMoreItems"].is_boolean(),
        "HasMoreItems key missing or not a bool"
    );
    assert!(
        json["Items"].is_array(),
        "Items key missing or not an array"
    );
}

#[tokio::test]
async fn list_pull_zones_forwards_explicit_page_and_per_page() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(query_param("page", "3"))
        .and(query_param("perPage", "25"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_PAGINATED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_pull_zones(Some(3), Some(25), None)
        .await
        .unwrap();

    assert!(!result.items.is_empty());
}

#[tokio::test]
async fn get_pull_zone_returns_single_zone() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let zone = test_client(&server.uri())
        .get_pull_zone(1001)
        .await
        .unwrap();

    // The mock URL is /pullzone/1001, so whatever the fixture's Id field
    // contains, the round-trip ID is not what we're testing — we're testing
    // that the response deserialised at all and has a non-empty name.
    assert!(zone.id > 0);
    assert!(!zone.name.is_empty());
    // Confirm the Enabled key is present in the fixture JSON (not silently
    // defaulted by serde) and is a bool.
    let json: serde_json::Value = serde_json::from_str(FIXTURE_GET).unwrap();
    assert!(
        json["Enabled"].is_boolean(),
        "Enabled key missing or not a bool"
    );
}

#[tokio::test]
async fn get_pull_zone_with_magic_container_origin_does_not_panic() {
    // Regression: bunny.net returns OriginType=5 for Magic-Container-backed
    // Pull Zones; before iter-19 this panicked at deserialize time. The value
    // is now recognised as `OriginType::MagicContainerEndpoint`. The
    // fallback-to-None behaviour for *unknown* repr integers is exercised by
    // `get_pull_zone_with_unknown_origin_type_falls_back_to_none`.
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone/5719318"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_GET_MAGIC_CONTAINER, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let zone = test_client(&server.uri())
        .get_pull_zone(5719318)
        .await
        .unwrap();

    assert_eq!(zone.id, 5719318);
    // OriginType: 5 is recognised as MagicContainerEndpoint.
    assert_eq!(zone.origin_type, Some(OriginType::MagicContainerEndpoint));
    assert_eq!(zone.origin_url, "");
}

#[tokio::test]
async fn get_pull_zone_with_unknown_origin_type_falls_back_to_none() {
    // Future-proofing: an unrecognised OriginType integer must not panic.
    let server = MockServer::start().await;

    let payload = serde_json::json!({
        "Id": 9999,
        "Name": "future-zone",
        "OriginType": 99,
        "Type": 0
    });

    Mock::given(method("GET"))
        .and(path("/pullzone/9999"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(payload.to_string(), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let zone = test_client(&server.uri())
        .get_pull_zone(9999)
        .await
        .unwrap();

    assert_eq!(zone.id, 9999);
    assert_eq!(zone.origin_type, None);
}

#[tokio::test]
async fn invalid_api_key_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .respond_with(
            ResponseTemplate::new(401).set_body_raw(FIXTURE_UNAUTHORIZED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .list_pull_zones(None, None, None)
        .await
        .unwrap_err();

    let api_err = err
        .downcast_ref::<ApiError>()
        .expect("should be an ApiError");
    assert_eq!(api_err.status_code, 401);
    assert!(api_err.message.contains("Authorization has been denied"));
}

// ---------------------------------------------------------------------------
// URL purge tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn purge_url_sends_url_as_query_param() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/purge"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("url", "https://cdn.example.com/style.css"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .purge_url("https://cdn.example.com/style.css")
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Pull Zone hostname & SSL tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn add_hostname_sends_correct_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/addHostname"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "cdn.example.com" }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .add_hostname(1001, "cdn.example.com")
        .await
        .unwrap();
}

#[tokio::test]
async fn remove_hostname_sends_correct_body() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/pullzone/1001/removeHostname"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "cdn.example.com" }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .remove_hostname(1001, "cdn.example.com")
        .await
        .unwrap();
}

#[tokio::test]
async fn load_free_certificate_sends_hostname_as_query_param() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/loadFreeCertificate"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("hostname", "cdn.example.com"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .load_free_certificate("cdn.example.com")
        .await
        .unwrap();
}

#[tokio::test]
async fn set_force_ssl_sends_correct_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/setForceSSL"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "cdn.example.com", "ForceSSL": true }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .set_force_ssl(1001, "cdn.example.com", true)
        .await
        .unwrap();
}

#[tokio::test]
async fn add_certificate_sends_correct_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/addCertificate"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "Hostname": "cdn.example.com",
            "Certificate": "base64cert",
            "CertificateKey": "base64key"
        })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .add_certificate(1001, "cdn.example.com", "base64cert", "base64key")
        .await
        .unwrap();
}

#[tokio::test]
async fn remove_certificate_sends_correct_body() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/pullzone/1001/removeCertificate"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "cdn.example.com" }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .remove_certificate(1001, "cdn.example.com")
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Pull Zone access-control tests (referrer / IP)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn add_allowed_referrer_sends_correct_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/addAllowedReferrer"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "*.example.com" }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .add_allowed_referrer(1001, "*.example.com")
        .await
        .unwrap();
}

#[tokio::test]
async fn remove_allowed_referrer_sends_correct_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/removeAllowedReferrer"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "*.example.com" }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .remove_allowed_referrer(1001, "*.example.com")
        .await
        .unwrap();
}

#[tokio::test]
async fn add_blocked_referrer_sends_correct_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/addBlockedReferrer"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({ "Hostname": "badsite.com" })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .add_blocked_referrer(1001, "badsite.com")
        .await
        .unwrap();
}

#[tokio::test]
async fn remove_blocked_referrer_sends_correct_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/removeBlockedReferrer"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({ "Hostname": "badsite.com" })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .remove_blocked_referrer(1001, "badsite.com")
        .await
        .unwrap();
}

#[tokio::test]
async fn add_blocked_ip_sends_correct_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/addBlockedIp"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({ "BlockedIp": "192.0.2.1" })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .add_blocked_ip(1001, "192.0.2.1")
        .await
        .unwrap();
}

#[tokio::test]
async fn remove_blocked_ip_sends_correct_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/removeBlockedIp"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({ "BlockedIp": "192.0.2.1" })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .remove_blocked_ip(1001, "192.0.2.1")
        .await
        .unwrap();
}

#[tokio::test]
async fn add_hostname_not_found_returns_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/99999/addHostname"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(404).set_body_raw(FIXTURE_NOT_FOUND, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .add_hostname(99999, "cdn.example.com")
        .await
        .unwrap_err();

    let api_err = err
        .downcast_ref::<ApiError>()
        .expect("should be an ApiError");
    assert_eq!(api_err.status_code, 404);
}

#[tokio::test]
async fn get_pull_zone_optimizer_statistics_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone/42/optimizer/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_OPTIMIZER_STATS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stats = test_client(&server.uri())
        .get_pull_zone_optimizer_statistics(42, None, None, false)
        .await
        .unwrap();

    assert!(stats.total_requests_optimized.is_finite());
    assert!(stats.total_requests_optimized >= 0.0);
    assert!(stats.average_compression_ratio.is_finite());
    assert!(stats.average_compression_ratio >= 0.0);
}

#[tokio::test]
async fn get_pull_zone_origin_shield_statistics_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone/42/originshield/queuestatistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_ORIGINSHIELD_STATS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stats = test_client(&server.uri())
        .get_pull_zone_origin_shield_statistics(42, None, None, false)
        .await
        .unwrap();

    assert!(stats.concurrent_requests_chart.is_some());
    assert!(!stats.concurrent_requests_chart.unwrap().is_empty());
}

#[tokio::test]
async fn get_pull_zone_safehop_statistics_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone/42/safehop/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SAFEHOP_STATS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stats = test_client(&server.uri())
        .get_pull_zone_safehop_statistics(42, None, None, false)
        .await
        .unwrap();

    assert!(stats.total_requests_retried.is_finite());
    assert!(stats.total_requests_retried >= 0.0);
    assert!(stats.total_requests_saved.is_finite());
    assert!(stats.total_requests_saved >= 0.0);
}

// ---------------------------------------------------------------------------
// Edge rule tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn add_or_update_edge_rule_sends_correct_request() {
    let server = MockServer::start().await;

    let trigger = EdgeRuleTrigger {
        trigger_type: Some(TriggerType::Url),
        pattern_matches: vec!["*/old-path*".to_string()],
        pattern_matching_type: Some(MatchingType::MatchAny),
        parameter1: None,
    };
    let body = AddOrUpdateEdgeRule::new(EdgeRuleActionType::Redirect)
        .action_parameter1("https://example.com/new-path")
        .action_parameter2("301")
        .trigger(trigger)
        .description("Redirect old paths");

    Mock::given(method("POST"))
        .and(path("/pullzone/1001/edgerules/addOrUpdate"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "ActionType": 1,
            "ActionParameter1": "https://example.com/new-path",
            "ActionParameter2": "301",
            "Triggers": [
                {
                    "Type": 0,
                    "PatternMatches": ["*/old-path*"],
                    "PatternMatchingType": 0,
                    "Parameter1": null
                }
            ],
            "Description": "Redirect old paths"
        })))
        .respond_with(ResponseTemplate::new(201))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .add_or_update_edge_rule(1001, &body)
        .await
        .unwrap();
}

#[tokio::test]
async fn delete_edge_rule_sends_correct_request() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path(
            "/pullzone/1001/edgerules/a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        ))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_edge_rule(1001, "a1b2c3d4-e5f6-7890-abcd-ef1234567890")
        .await
        .unwrap();
}

#[tokio::test]
async fn set_edge_rule_enabled_sends_correct_request() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/pullzone/1001/edgerules/a1b2c3d4-e5f6-7890-abcd-ef1234567890/setEdgeRuleEnabled",
        ))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "Id": 1001,
            "Value": false
        })))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .set_edge_rule_enabled(1001, "a1b2c3d4-e5f6-7890-abcd-ef1234567890", false)
        .await
        .unwrap();
}

#[tokio::test]
async fn get_pull_zone_deserializes_edge_rules() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_GET_WITH_EDGERULES, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let zone = test_client(&server.uri())
        .get_pull_zone(1001)
        .await
        .unwrap();

    assert_eq!(zone.edge_rules.len(), 2);

    let redirect_rule = &zone.edge_rules[0];
    assert_eq!(
        redirect_rule.guid.as_deref(),
        Some("a1b2c3d4-e5f6-7890-abcd-ef1234567890")
    );
    assert_eq!(
        redirect_rule.action_type,
        Some(EdgeRuleActionType::Redirect)
    );
    assert_eq!(
        redirect_rule.action_parameter1.as_deref(),
        Some("https://example.com/new-path")
    );
    assert_eq!(
        redirect_rule.description.as_deref(),
        Some("Redirect old paths")
    );
    assert!(redirect_rule.enabled);
    assert_eq!(redirect_rule.triggers.len(), 1);
    assert_eq!(
        redirect_rule.triggers[0].trigger_type,
        Some(TriggerType::Url)
    );
    assert_eq!(
        redirect_rule.triggers[0].pattern_matches,
        vec!["*/old-path*"]
    );

    let block_rule = &zone.edge_rules[1];
    assert_eq!(
        block_rule.guid.as_deref(),
        Some("b2c3d4e5-f6a7-8901-bcde-f12345678901")
    );
    assert_eq!(
        block_rule.action_type,
        Some(EdgeRuleActionType::BlockRequest)
    );
    assert_eq!(block_rule.description.as_deref(), Some("Block countries"));
    assert!(block_rule.enabled);
    assert_eq!(block_rule.triggers.len(), 1);
    assert_eq!(
        block_rule.triggers[0].trigger_type,
        Some(TriggerType::CountryCode)
    );
    assert_eq!(block_rule.triggers[0].pattern_matches, vec!["CN", "RU"]);
}

// ---------------------------------------------------------------------------
// Optimizer field tests
// ---------------------------------------------------------------------------

/// Only the three explicitly-set fields should appear in the serialised body;
/// no other Optimizer keys should be present.
#[tokio::test]
async fn update_pull_zone_with_optimizer_enabled_sends_only_set_fields() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "OptimizerEnabled": true,
        "OptimizerImageQuality": 80,
        "OptimizerEnableWebP": true
    });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new()
        .optimizer_enabled(true)
        .optimizer_image_quality(80)
        .optimizer_enable_web_p(true);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

/// `false` must NOT be skipped by `skip_serializing_if = "Option::is_none"`.
#[tokio::test]
async fn update_pull_zone_with_optimizer_enabled_false_serializes_false() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({ "OptimizerEnabled": false });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new().optimizer_enabled(false);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

/// The wire key must be the long form `OptimizerMinifyJavaScript`, not
/// `OptimizerMinifyJs` or any abbreviation.
#[tokio::test]
async fn update_pull_zone_with_optimizer_minify_javascript_uses_long_form_key() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({ "OptimizerMinifyJavaScript": true });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new().optimizer_minify_java_script(true);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

/// The watermark position enum must serialise as its integer discriminant.
/// `Center` is 4 on the wire.
#[tokio::test]
async fn update_pull_zone_with_optimizer_watermark_position_serializes_as_int() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({ "OptimizerWatermarkPosition": 4 });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body =
        UpdatePullZone::new().optimizer_watermark_position(OptimizerWatermarkPosition::Center);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

/// The Optimizer-rich fixture must deserialise without panic, and all Optimizer
/// fields must match the fixture values. Also verifies that `OptimizerEnableWebP`
/// (capital P) deserialises correctly via PascalCase rename.
#[tokio::test]
async fn get_pull_zone_with_optimizer_round_trips_all_fields() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/pullzone/2002"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_GET_WITH_OPTIMIZER, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let zone = test_client(&server.uri())
        .get_pull_zone(2002)
        .await
        .unwrap();

    assert_eq!(zone.id, 2002);

    // Master switches
    assert_eq!(zone.optimizer_enabled, Some(true));
    assert_eq!(zone.optimizer_automatic_optimization_enabled, Some(true));

    // Image dimensions & quality
    assert_eq!(zone.optimizer_desktop_max_width, Some(1920));
    assert_eq!(zone.optimizer_mobile_max_width, Some(960));
    assert_eq!(zone.optimizer_image_quality, Some(80));
    assert_eq!(zone.optimizer_mobile_image_quality, Some(65));

    // Format & upscale — verifies `OptimizerEnableWebP` (capital P)
    assert_eq!(zone.optimizer_enable_web_p, Some(true));
    assert_eq!(zone.optimizer_enable_upscaling, Some(true));

    // Minify — verifies `OptimizerMinifyJavaScript` long form
    assert_eq!(zone.optimizer_minify_css, Some(true));
    assert_eq!(zone.optimizer_minify_java_script, Some(true));

    // Manipulation engine — verifies string form of classes
    assert_eq!(zone.optimizer_enable_manipulation_engine, Some(true));
    assert_eq!(
        zone.optimizer_classes.as_deref(),
        Some("{\"thumb\":\"width=200,quality=80\"}")
    );
    assert_eq!(zone.optimizer_force_classes, Some(true));

    // Watermark
    assert_eq!(zone.optimizer_watermark_enabled, Some(true));
    assert_eq!(
        zone.optimizer_watermark_url.as_deref(),
        Some("https://example.com/watermark.png")
    );
    assert_eq!(
        zone.optimizer_watermark_position,
        Some(OptimizerWatermarkPosition::Center)
    );
    assert_eq!(zone.optimizer_watermark_offset, Some(5.0));
    assert_eq!(zone.optimizer_watermark_min_image_size, Some(500));

    // Static HTML
    assert_eq!(zone.optimizer_static_html_enabled, Some(true));
    assert_eq!(
        zone.optimizer_static_html_word_press_path.as_deref(),
        Some("/wp-content")
    );
    assert_eq!(
        zone.optimizer_static_html_word_press_bypass_cookie
            .as_deref(),
        Some("bypass_cache")
    );

    // Prerender & tunnel
    assert_eq!(zone.optimizer_prerender_html, Some(true));
    assert_eq!(zone.optimizer_tunnel_enabled, Some(true));

    // Read-only pricing (float, not int)
    assert_eq!(zone.optimizer_pricing, Some(9.5));

    // Round-trip: serialise back to JSON and verify key Optimizer fields survive
    let json_val = serde_json::to_value(&zone).unwrap();
    assert_eq!(json_val["OptimizerEnabled"], serde_json::json!(true));
    assert_eq!(json_val["OptimizerEnableWebP"], serde_json::json!(true));
    assert_eq!(
        json_val["OptimizerMinifyJavaScript"],
        serde_json::json!(true)
    );
    assert_eq!(json_val["OptimizerWatermarkPosition"], serde_json::json!(4));
}

// ---------------------------------------------------------------------------
// Log forwarding — update serialisation tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn update_pull_zone_with_log_forwarding_enabled_sends_field() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({ "LogForwardingEnabled": true });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new().log_forwarding_enabled(true);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

#[tokio::test]
async fn update_pull_zone_with_log_forwarding_hostname_sends_field() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({ "LogForwardingHostname": "logs.example.com" });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new().log_forwarding_hostname("logs.example.com");

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

#[tokio::test]
async fn update_pull_zone_with_log_forwarding_port_sends_field() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({ "LogForwardingPort": 514 });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new().log_forwarding_port(514);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

#[tokio::test]
async fn update_pull_zone_with_log_forwarding_token_sends_field() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({ "LogForwardingToken": "my-secret-token" });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new().log_forwarding_token("my-secret-token");

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

/// The log forwarding protocol enum must serialise as its integer discriminant.
/// `Tcp` is 1 on the wire.
#[tokio::test]
async fn update_pull_zone_with_log_forwarding_protocol_serializes_as_int() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({ "LogForwardingProtocol": 1 });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new().log_forwarding_protocol(PullZoneLogForwarderProtocolType::Tcp);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

#[tokio::test]
async fn update_pull_zone_with_logging_save_to_storage_sends_field() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({ "LoggingSaveToStorage": true });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new().logging_save_to_storage(true);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

#[tokio::test]
async fn update_pull_zone_with_logging_storage_zone_id_sends_field() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({ "LoggingStorageZoneId": 42 });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new().logging_storage_zone_id(42);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

/// All seven log-forwarding fields must deserialise correctly from a fixture
/// that includes them. Uses an inline minimal PullZone JSON.
#[tokio::test]
async fn get_pull_zone_with_log_forwarding_fields_round_trips() {
    let server = MockServer::start().await;

    // Minimal PullZone fixture with all log-forwarding fields populated.
    // `LogForwardingProtocol: 1` corresponds to `Tcp`.
    let fixture = serde_json::json!({
        "Id": 9001,
        "Name": "lf-test",
        "OriginUrl": "https://origin.example.com",
        "Enabled": true,
        "Suspended": false,
        "Hostnames": [],
        "StorageZoneId": 0,
        "AllowedReferrers": [],
        "BlockedReferrers": [],
        "BlockedIps": [],
        "EnableGeoZoneUS": true,
        "EnableGeoZoneEU": true,
        "EnableGeoZoneASIA": true,
        "EnableGeoZoneSA": true,
        "EnableGeoZoneAF": true,
        "ZoneSecurityEnabled": false,
        "MonthlyBandwidthUsed": 0,
        "MonthlyBandwidthLimit": 0,
        "CnameDomain": "b-cdn.net",
        "Type": 0,
        "EdgeRules": [],
        "LogForwardingEnabled": true,
        "LogForwardingHostname": "logs.example.com",
        "LogForwardingPort": 514,
        "LogForwardingToken": "secret-token",
        "LogForwardingProtocol": 1,
        "LoggingSaveToStorage": true,
        "LoggingStorageZoneId": 99
    })
    .to_string();

    Mock::given(method("GET"))
        .and(path("/pullzone/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(fixture.as_str(), "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let zone = test_client(&server.uri())
        .get_pull_zone(9001)
        .await
        .unwrap();

    assert_eq!(zone.id, 9001);
    assert_eq!(zone.log_forwarding_enabled, Some(true));
    assert_eq!(
        zone.log_forwarding_hostname.as_deref(),
        Some("logs.example.com")
    );
    assert_eq!(zone.log_forwarding_port, Some(514));
    assert_eq!(zone.log_forwarding_token.as_deref(), Some("secret-token"));
    assert_eq!(
        zone.log_forwarding_protocol,
        Some(PullZoneLogForwarderProtocolType::Tcp)
    );
    assert_eq!(zone.logging_save_to_storage, Some(true));
    assert_eq!(zone.logging_storage_zone_id, Some(99));
}

// ---------------------------------------------------------------------------
// Security / compliance (iter-44)
// ---------------------------------------------------------------------------

/// Sparse update with two security/compliance fields must serialise to only
/// those two keys — confirms sparse update semantics + correct PascalCase keys
/// (`EnableTLS1` uppercase, `VerifyOriginSSL` uppercase, no extras).
#[tokio::test]
async fn update_pull_zone_with_security_fields_sends_only_set_keys() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "EnableTLS1": false,
        "VerifyOriginSSL": true,
    });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new()
        .enable_tls1(false)
        .verify_origin_ssl(true);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

/// `LogAnonymizationType` must serialise as its integer discriminant
/// (`Drop` is 1 on the wire) — mirrors the watermark-position test.
#[tokio::test]
async fn update_pull_zone_with_log_anonymization_type_serializes_as_int() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({ "LogAnonymizationType": 1 });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new().log_anonymization_type(LogAnonymizationType::Drop);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

/// Round-trip for the `LogAnonymizationType` enum: both variants must
/// serialise to and deserialise from their integer discriminants.
#[test]
fn log_anonymization_type_round_trips() {
    let one_digit = serde_json::to_string(&LogAnonymizationType::OneDigit).unwrap();
    let drop = serde_json::to_string(&LogAnonymizationType::Drop).unwrap();
    assert_eq!(one_digit, "0");
    assert_eq!(drop, "1");

    let parsed_one: LogAnonymizationType = serde_json::from_str("0").unwrap();
    let parsed_drop: LogAnonymizationType = serde_json::from_str("1").unwrap();
    assert_eq!(parsed_one, LogAnonymizationType::OneDigit);
    assert_eq!(parsed_drop, LogAnonymizationType::Drop);
}

/// `AWSSigningKey` / `AWSSigningSecret` use the uppercase `AWS` rename. A
/// sparse update containing both must serialise with the exact PascalCase keys
/// the API expects.
#[tokio::test]
async fn update_pull_zone_with_aws_signing_uses_uppercase_keys() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "AWSSigningEnabled": true,
        "AWSSigningKey": "AKIAEXAMPLE",
        "AWSSigningSecret": "secret",
        "AWSSigningRegionName": "us-east-1",
    });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new()
        .aws_signing_enabled(true)
        .aws_signing_key("AKIAEXAMPLE")
        .aws_signing_secret("secret")
        .aws_signing_region_name("us-east-1");

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

/// CORS extensions list must round-trip as a JSON array under the
/// `AccessControlOriginHeaderExtensions` key.
#[tokio::test]
async fn update_pull_zone_with_cors_extensions_sends_array() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "EnableAccessControlOriginHeader": true,
        "AccessControlOriginHeaderExtensions": ["woff", "woff2", "ttf"],
    });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new()
        .enable_access_control_origin_header(true)
        .access_control_origin_header_extensions(vec![
            "woff".to_string(),
            "woff2".to_string(),
            "ttf".to_string(),
        ]);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Vary headers + performance / caching (iter-45)
// ---------------------------------------------------------------------------

/// Sparse update with one vary flag + one cache override must serialise to
/// exactly those two PascalCase keys.
#[tokio::test]
async fn update_pull_zone_with_vary_and_cache_sends_only_set_keys() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "EnableWebPVary": true,
        "UseStaleWhileUpdating": true,
    });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new()
        .enable_webp_vary(true)
        .use_stale_while_updating(true);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

/// `EnableAvifVary` uses the explicit rename (rather than auto-PascalCase from
/// `enable_avif_vary` → `EnableAvifVary`, which happens to match). Verify the
/// exact wire spelling.
#[tokio::test]
async fn update_pull_zone_with_avif_vary_uses_pascal_case_key() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({ "EnableAvifVary": true });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new().enable_avif_vary(true);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

/// Cache-control max-age override fields must serialise as integers (not
/// strings) and only when explicitly set.
#[tokio::test]
async fn update_pull_zone_with_cache_control_overrides_serializes_as_ints() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "CacheControlMaxAgeOverride": 3600,
        "CacheControlPublicMaxAgeOverride": 1800,
        "CacheControlBrowserMaxAgeOverride": 600,
    });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new()
        .cache_control_max_age_override(3600)
        .cache_control_public_max_age_override(1800)
        .cache_control_browser_max_age_override(600);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

/// `QueryStringVaryParameters` and `CookieVaryParameters` round-trip as JSON
/// arrays.
#[tokio::test]
async fn update_pull_zone_with_vary_parameter_lists_sends_arrays() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "QueryStringVaryParameters": ["v", "locale"],
        "CookieVaryParameters": ["session", "locale"],
    });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new()
        .query_string_vary_parameters(vec!["v".to_string(), "locale".to_string()])
        .cookie_vary_parameters(vec!["session".to_string(), "locale".to_string()]);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

/// `PermaCacheType` must serialise as its integer discriminant (`Manual` is 1).
#[tokio::test]
async fn update_pull_zone_with_perma_cache_type_serializes_as_int() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "PermaCacheStorageZoneId": 4242,
        "PermaCacheType": 1,
    });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new()
        .perma_cache_storage_zone_id(4242)
        .perma_cache_type(PermaCacheType::Manual);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

/// Round-trip for the `PermaCacheType` enum: both variants must serialise to
/// and deserialise from their integer discriminants.
#[test]
fn perma_cache_type_round_trips() {
    let automatic = serde_json::to_string(&PermaCacheType::Automatic).unwrap();
    let manual = serde_json::to_string(&PermaCacheType::Manual).unwrap();
    assert_eq!(automatic, "0");
    assert_eq!(manual, "1");

    let parsed_automatic: PermaCacheType = serde_json::from_str("0").unwrap();
    let parsed_manual: PermaCacheType = serde_json::from_str("1").unwrap();
    assert_eq!(parsed_automatic, PermaCacheType::Automatic);
    assert_eq!(parsed_manual, PermaCacheType::Manual);
}

// ── iter-46: StickySessionType ───────────────────────────────────────────────

/// Round-trip for `StickySessionType`: both variants must serialise to and
/// deserialise from their integer discriminants.
#[test]
fn sticky_session_type_round_trips() {
    let none = serde_json::to_string(&StickySessionType::None).unwrap();
    let cookie = serde_json::to_string(&StickySessionType::Cookie).unwrap();
    assert_eq!(none, "0");
    assert_eq!(cookie, "1");

    let parsed_none: StickySessionType = serde_json::from_str("0").unwrap();
    let parsed_cookie: StickySessionType = serde_json::from_str("1").unwrap();
    assert_eq!(parsed_none, StickySessionType::None);
    assert_eq!(parsed_cookie, StickySessionType::Cookie);
}

/// `StickySessionType` must serialise as its integer discriminant on the wire.
#[tokio::test]
async fn update_pull_zone_with_sticky_session_type_serializes_as_int() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "StickySessionType": 1,
        "StickySessionCookieName": "sticky",
    });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new()
        .sticky_session_type(StickySessionType::Cookie)
        .sticky_session_cookie_name("sticky");

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

// ── iter-46: PullZoneTierType ────────────────────────────────────────────────

/// Round-trip for `PullZoneTierType`: both variants must serialise to and
/// deserialise from their integer discriminants.
#[test]
fn pull_zone_tier_type_round_trips() {
    let standard = serde_json::to_string(&PullZoneTierType::Standard).unwrap();
    let volume = serde_json::to_string(&PullZoneTierType::Volume).unwrap();
    assert_eq!(standard, "0");
    assert_eq!(volume, "1");

    let parsed_standard: PullZoneTierType = serde_json::from_str("0").unwrap();
    let parsed_volume: PullZoneTierType = serde_json::from_str("1").unwrap();
    assert_eq!(parsed_standard, PullZoneTierType::Standard);
    assert_eq!(parsed_volume, PullZoneTierType::Volume);
}

/// `PullZoneTierType` must serialise under the wire key `Type` and as an
/// integer discriminant.
#[tokio::test]
async fn update_pull_zone_with_tier_type_uses_type_key() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({ "Type": 1 });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new().pull_zone_tier_type(PullZoneTierType::Volume);

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

// ── iter-46: origin timeout + retry + sticky session integration test ────────

/// Sparse update with one timeout, one retry flag, and a sticky session cookie
/// name — verifies the correct PascalCase keys appear in the wire payload.
#[tokio::test]
async fn update_pull_zone_origin_timeout_retry_sticky_session() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "OriginConnectTimeout": 10,
        "OriginRetry5XXResponses": true,
        "StickySessionCookieName": "my-session",
    });

    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdatePullZone::new()
        .origin_connect_timeout(10)
        .origin_retry_5xx_responses(true)
        .sticky_session_cookie_name("my-session");

    test_client(&server.uri())
        .update_pull_zone(1001, &body)
        .await
        .unwrap();
}

// ── Geo-zone casing regression tests (iter-49) ───────────────────────────────

/// Verify that the five geo-zone fields in `PullZone` correctly deserialise
/// from the real API key names (`EnableGeoZoneUS`, `EnableGeoZoneEU`,
/// `EnableGeoZoneASIA`, `EnableGeoZoneSA`, `EnableGeoZoneAF`).
///
/// Before iter-49 the fields had no explicit `#[serde(rename)]` and `serde`'s
/// PascalCase conversion produced `EnableGeoZoneUs` etc., which silently
/// defaulted to `false` instead of mapping to the true values in the response.
#[tokio::test]
async fn geo_zone_fields_deserialise_from_real_api_key_names() {
    let server = MockServer::start().await;

    // FIXTURE_GET has all five geo-zone flags set to true; use it as the
    // response body so we get a realistic round-trip without an inline JSON blob.
    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let zone = test_client(&server.uri())
        .get_pull_zone(1001)
        .await
        .unwrap();

    assert!(
        zone.enable_geo_zone_us,
        "enable_geo_zone_us was false — serde key mismatch (expected EnableGeoZoneUS)"
    );
    assert!(
        zone.enable_geo_zone_eu,
        "enable_geo_zone_eu was false — serde key mismatch (expected EnableGeoZoneEU)"
    );
    assert!(
        zone.enable_geo_zone_asia,
        "enable_geo_zone_asia was false — serde key mismatch (expected EnableGeoZoneASIA)"
    );
    assert!(
        zone.enable_geo_zone_sa,
        "enable_geo_zone_sa was false — serde key mismatch (expected EnableGeoZoneSA)"
    );
    assert!(
        zone.enable_geo_zone_af,
        "enable_geo_zone_af was false — serde key mismatch (expected EnableGeoZoneAF)"
    );
}

/// Verify that `UpdatePullZone` serialises geo-zone fields with the correct
/// uppercase acronym key names expected by the bunny.net API.
#[test]
fn update_pull_zone_geo_zone_fields_serialise_with_correct_key_names() {
    let body = UpdatePullZone::new()
        .enable_geo_zone_us(true)
        .enable_geo_zone_eu(false)
        .enable_geo_zone_asia(true)
        .enable_geo_zone_sa(false)
        .enable_geo_zone_af(true);

    let json: serde_json::Value = serde_json::to_value(&body).unwrap();

    assert_eq!(
        json["EnableGeoZoneUS"],
        serde_json::Value::Bool(true),
        "EnableGeoZoneUS key absent or wrong"
    );
    assert_eq!(
        json["EnableGeoZoneEU"],
        serde_json::Value::Bool(false),
        "EnableGeoZoneEU key absent or wrong"
    );
    assert_eq!(
        json["EnableGeoZoneASIA"],
        serde_json::Value::Bool(true),
        "EnableGeoZoneASIA key absent or wrong"
    );
    assert_eq!(
        json["EnableGeoZoneSA"],
        serde_json::Value::Bool(false),
        "EnableGeoZoneSA key absent or wrong"
    );
    assert_eq!(
        json["EnableGeoZoneAF"],
        serde_json::Value::Bool(true),
        "EnableGeoZoneAF key absent or wrong"
    );

    // Confirm the old wrong PascalCase keys are not present.
    assert!(
        json["EnableGeoZoneUs"].is_null(),
        "stale key EnableGeoZoneUs present"
    );
    assert!(
        json["EnableGeoZoneEu"].is_null(),
        "stale key EnableGeoZoneEu present"
    );
    assert!(
        json["EnableGeoZoneAsia"].is_null(),
        "stale key EnableGeoZoneAsia present"
    );
    assert!(
        json["EnableGeoZoneSa"].is_null(),
        "stale key EnableGeoZoneSa present"
    );
    assert!(
        json["EnableGeoZoneAf"].is_null(),
        "stale key EnableGeoZoneAf present"
    );
}

// ── Remaining toggles regression tests (iter-65) ─────────────────────────────

/// Verify that the last four pull-zone toggles (`EnableBunnyImageAi`,
/// `EnableLogging`, `EnableExtendedLogging`, `EnableWebSockets`) correctly
/// deserialise from the real API key names. Default PascalCase conversion of
/// the snake_case field names already produces the correct wire keys, but a
/// future rename of any of these fields could silently reintroduce a casing
/// mismatch that defaults to `false` instead of surfacing the real value.
#[tokio::test]
async fn remaining_toggle_fields_deserialise_from_real_api_key_names() {
    let server = MockServer::start().await;

    // The recorded FIXTURE_GET has EnableLogging=true and EnableWebSockets=true
    // but EnableBunnyImageAi=false and EnableExtendedLogging=false. A `false`
    // value can't catch a casing regression (field missing → `#[serde(default)]`
    // → false looks identical), so flip those two to true in memory: every
    // assertion below then fails if its wire key stops matching.
    let mut body: serde_json::Value = serde_json::from_str(FIXTURE_GET).unwrap();
    body["EnableBunnyImageAi"] = serde_json::Value::Bool(true);
    body["EnableExtendedLogging"] = serde_json::Value::Bool(true);

    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let zone = test_client(&server.uri())
        .get_pull_zone(1001)
        .await
        .unwrap();

    assert!(
        zone.enable_logging,
        "enable_logging was false — serde key mismatch (expected EnableLogging)"
    );
    assert!(
        zone.enable_web_sockets,
        "enable_web_sockets was false — serde key mismatch (expected EnableWebSockets)"
    );
    assert!(
        zone.enable_bunny_image_ai,
        "enable_bunny_image_ai was false — serde key mismatch (expected EnableBunnyImageAi)"
    );
    assert!(
        zone.enable_extended_logging,
        "enable_extended_logging was false — serde key mismatch (expected EnableExtendedLogging)"
    );
}

/// Verify that `UpdatePullZone` serialises the remaining toggle fields with
/// the correct PascalCase key names expected by the bunny.net API.
#[test]
fn update_pull_zone_remaining_toggle_fields_serialise_with_correct_key_names() {
    let body = UpdatePullZone::new()
        .enable_bunny_image_ai(true)
        .enable_logging(false)
        .enable_extended_logging(true)
        .enable_web_sockets(false);

    let json: serde_json::Value = serde_json::to_value(&body).unwrap();

    assert_eq!(
        json["EnableBunnyImageAi"],
        serde_json::Value::Bool(true),
        "EnableBunnyImageAi key absent or wrong"
    );
    assert_eq!(
        json["EnableLogging"],
        serde_json::Value::Bool(false),
        "EnableLogging key absent or wrong"
    );
    assert_eq!(
        json["EnableExtendedLogging"],
        serde_json::Value::Bool(true),
        "EnableExtendedLogging key absent or wrong"
    );
    assert_eq!(
        json["EnableWebSockets"],
        serde_json::Value::Bool(false),
        "EnableWebSockets key absent or wrong"
    );
}
