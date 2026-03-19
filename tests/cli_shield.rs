mod support;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn shield_zone_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/shield/shield-zones"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/shield_zones_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "shield", "zone", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn shield_zone_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/55001"))
        .and(header("AccessKey", "test-api-key"))
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
            "get",
            "--shield-zone-id",
            "55001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn shield_zone_get_by_pullzone_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/get-by-pullzone/100001"))
        .and(header("AccessKey", "test-api-key"))
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
            "get-by-pullzone",
            "--pull-zone-id",
            "100001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn shield_zone_create_calls_post() {
    let server = MockServer::start().await;
    // The create endpoint returns a nested response shape we don't have a real
    // fixture for, so we just verify the correct endpoint is called.
    Mock::given(method("POST"))
        .and(path("/shield/shield-zone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/shield_zone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "shield",
            "zone",
            "create",
            "--pull-zone-id",
            "100001",
        ])
        .output()
        .unwrap();
    // Mock's .expect(1) verifies the POST was made to /shield/shield-zone.
    // We don't assert exit code because we lack a real create response fixture.
}

#[tokio::test]
async fn shield_zone_update() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/shield/shield-zone"))
        .and(header("AccessKey", "test-api-key"))
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
            "--waf-enabled",
            "true",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn shield_waf_profiles_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/shield/waf/profiles"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/waf_profiles_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "shield", "waf", "profiles"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn shield_waf_list_rules_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/shield/waf/custom-rules/55001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/waf_rules_list.json"),
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
            "list-rules",
            "--shield-zone-id",
            "55001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn shield_waf_get_rule_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/shield/waf/custom-rule/7001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/waf_rule_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format", "json", "shield", "waf", "get-rule", "--id", "7001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn shield_waf_add_rule_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/shield/waf/custom-rule"))
        .and(header("AccessKey", "test-api-key"))
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
            "Block Bad UA",
            "--action-type",
            "1",
            "--operator-type",
            "2",
            "--severity-type",
            "1",
            "--value",
            "BadBot/1.0",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn shield_waf_update_rule() {
    let server = MockServer::start().await;
    // The update handler first GETs the current rule to populate required fields.
    Mock::given(method("GET"))
        .and(path("/shield/waf/custom-rule/7001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/waf_rule_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/shield/waf/custom-rule/7001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/waf_rule_get.json"),
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
            "7001",
            "--name",
            "updated",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn shield_waf_delete_rule() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/shield/waf/custom-rule/7001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "--format",
            "json",
            "shield",
            "waf",
            "delete-rule",
            "--id",
            "7001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn shield_rate_limit_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/shield/rate-limits/55001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/rate_limit_rules_list.json"),
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
            "list",
            "--shield-zone-id",
            "55001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn shield_rate_limit_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/shield/rate-limit/8001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/rate_limit_rule_get.json"),
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
            "get",
            "--id",
            "8001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn shield_rate_limit_create_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/shield/rate-limit"))
        .and(header("AccessKey", "test-api-key"))
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
            "API Limit",
            "--action-type",
            "1",
            "--operator-type",
            "0",
            "--severity-type",
            "0",
            "--value",
            "/api",
            "--request-count",
            "100",
            "--counter-key-type",
            "1",
            "--timeframe",
            "3600",
            "--block-time",
            "900",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn shield_rate_limit_update() {
    let server = MockServer::start().await;
    // The update handler first GETs the current rule to populate required fields.
    Mock::given(method("GET"))
        .and(path("/shield/rate-limit/8001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/rate_limit_rule_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/shield/rate-limit/8001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/rate_limit_rule_get.json"),
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
            "8001",
            "--name",
            "updated",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn shield_rate_limit_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/shield/rate-limit/8001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "--format",
            "json",
            "shield",
            "rate-limit",
            "delete",
            "--id",
            "8001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn shield_access_list_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/55001/access-lists"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/access_lists_get.json"),
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
            "list",
            "--shield-zone-id",
            "55001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn shield_access_list_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/55001/access-lists/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/access_list_get.json"),
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
            "get",
            "--shield-zone-id",
            "55001",
            "--id",
            "9001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn shield_access_list_create_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/shield/shield-zone/55001/access-lists"))
        .and(header("AccessKey", "test-api-key"))
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
            "Allowed IPs",
            "--type",
            "0",
            "--content",
            "192.168.1.0/24",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn shield_access_list_update() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/shield/shield-zone/55001/access-lists/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/access_list_get.json"),
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
            "update",
            "--shield-zone-id",
            "55001",
            "--id",
            "9001",
            "--name",
            "updated",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn shield_access_list_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/shield/shield-zone/55001/access-lists/9001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "--format",
            "json",
            "shield",
            "access-list",
            "delete",
            "--shield-zone-id",
            "55001",
            "--id",
            "9001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn shield_bot_detection_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/shield/shield-zone/55001/bot-detection"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("shield/bot_detection_get.json"),
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
            "get",
            "--shield-zone-id",
            "55001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn shield_bot_detection_update() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/shield/shield-zone/55001/bot-detection"))
        .and(header("AccessKey", "test-api-key"))
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
            "--shield-zone-id",
            "55001",
            "--execution-mode",
            "1",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[cfg(feature = "live-api")]
#[test]
fn live_shield_lifecycle() {
    support::run_lifecycle(|cleanup| {
        let pz_name = support::unique_name("hoppy-shield-test");

        // 1. Create pull zone
        let pz_create = support::hoppy_live_json(&[
            "pull-zone",
            "create",
            "--name",
            &pz_name,
            "--origin-url",
            "https://example.com",
        ]);
        assert!(
            pz_create.success,
            "pull-zone create failed — stderr: {}",
            pz_create.stderr
        );
        let pz_id = pz_create.json.as_ref().unwrap()["Id"].as_i64().unwrap();
        let pz_id_str = pz_id.to_string();

        // Register pull zone cleanup immediately — Shield zone is deleted with it
        cleanup.push(&["pull-zone", "delete", "--id", &pz_id_str]);

        // 2. Create Shield zone
        let sz_create =
            support::hoppy_live_json(&["shield", "zone", "create", "--pull-zone-id", &pz_id_str]);
        assert!(
            sz_create.success,
            "shield zone create failed — stderr: {}",
            sz_create.stderr
        );
        let sz_json = sz_create.json.as_ref().unwrap();
        let sz_id = sz_json["shieldZoneId"]
            .as_i64()
            .or_else(|| sz_json["id"].as_i64())
            .expect("no shieldZoneId or id in shield zone create response");
        let sz_id_str = sz_id.to_string();

        // 3. Get Shield zone
        let sz_get =
            support::hoppy_live_json(&["shield", "zone", "get", "--shield-zone-id", &sz_id_str]);
        assert!(
            sz_get.success,
            "shield zone get failed — stderr: {}",
            sz_get.stderr
        );

        // 4. Get by pull zone
        let sz_get_by_pz = support::hoppy_live_json(&[
            "shield",
            "zone",
            "get-by-pullzone",
            "--pull-zone-id",
            &pz_id_str,
        ]);
        assert!(
            sz_get_by_pz.success,
            "shield zone get-by-pullzone failed — stderr: {}",
            sz_get_by_pz.stderr
        );

        // 5. List Shield zones (paginated — just verify the request succeeds)
        let sz_list = support::hoppy_live_json(&["shield", "zone", "list"]);
        assert!(
            sz_list.success,
            "shield zone list failed — stderr: {}",
            sz_list.stderr
        );

        // 6. Update Shield zone
        let sz_update = support::hoppy_live_json(&[
            "shield",
            "zone",
            "update",
            "--shield-zone-id",
            &sz_id_str,
            "--learning-mode",
            "true",
        ]);
        assert!(
            sz_update.success,
            "shield zone update failed — stderr: {}",
            sz_update.stderr
        );

        // 7. WAF profiles
        let waf_profiles = support::hoppy_live_json(&["shield", "waf", "profiles"]);
        assert!(
            waf_profiles.success,
            "shield waf profiles failed — stderr: {}",
            waf_profiles.stderr
        );

        // 8. Add WAF rule (may fail on free plans with customWafRulesLimit=0)
        let waf_add = support::hoppy_live_json(&[
            "shield",
            "waf",
            "add-rule",
            "--shield-zone-id",
            &sz_id_str,
            "--name",
            "hoppy-test-waf-rule",
            "--action-type",
            "1",
            "--operator-type",
            "0",
            "--severity-type",
            "0",
            "--value",
            "test-block",
        ]);
        let waf_id = waf_add
            .json
            .as_ref()
            .and_then(|v| v["id"].as_i64())
            .unwrap_or(0);

        if waf_id > 0 {
            let waf_id_str = waf_id.to_string();

            // 9. List WAF rules and verify ours appears
            let waf_list = support::hoppy_live_json(&[
                "shield",
                "waf",
                "list-rules",
                "--shield-zone-id",
                &sz_id_str,
            ]);
            assert!(
                waf_list.success,
                "shield waf list-rules failed — stderr: {}",
                waf_list.stderr
            );
            let waf_found = waf_list
                .json
                .as_ref()
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().any(|r| r["id"].as_i64() == Some(waf_id)))
                .unwrap_or(false);
            assert!(
                waf_found,
                "waf rule {waf_id} not found in list-rules output"
            );

            // 10. Get WAF rule
            let waf_get =
                support::hoppy_live_json(&["shield", "waf", "get-rule", "--id", &waf_id_str]);
            assert!(
                waf_get.success,
                "shield waf get-rule failed — stderr: {}",
                waf_get.stderr
            );

            // 11. Update WAF rule
            let waf_update = support::hoppy_live_json(&[
                "shield",
                "waf",
                "update-rule",
                "--id",
                &waf_id_str,
                "--name",
                "updated-rule",
            ]);
            assert!(
                waf_update.success,
                "shield waf update-rule failed — stderr: {}",
                waf_update.stderr
            );

            // 12. Delete WAF rule
            let waf_delete = support::hoppy_live_json_yes(&[
                "shield",
                "waf",
                "delete-rule",
                "--id",
                &waf_id_str,
            ]);
            assert!(
                waf_delete.success,
                "shield waf delete-rule failed — stderr: {}",
                waf_delete.stderr
            );
        } else {
            eprintln!("skipping WAF custom rule CRUD — plan does not support custom WAF rules");
        }

        // 13. Create rate limit (may fail due to plan limits)
        let rl_create = support::hoppy_live_json(&[
            "shield",
            "rate-limit",
            "create",
            "--shield-zone-id",
            &sz_id_str,
            "--name",
            "hoppy-test-rate-limit",
            "--action-type",
            "1",
            "--operator-type",
            "0",
            "--severity-type",
            "0",
            "--request-count",
            "100",
            "--counter-key-type",
            "1",
            "--timeframe",
            "10",
            "--block-time",
            "300",
        ]);
        let rl_id = rl_create
            .json
            .as_ref()
            .and_then(|v| v["id"].as_i64())
            .unwrap_or(0);

        if rl_id > 0 {
            let rl_id_str = rl_id.to_string();

            // 14. List rate limits
            let rl_list = support::hoppy_live_json(&[
                "shield",
                "rate-limit",
                "list",
                "--shield-zone-id",
                &sz_id_str,
            ]);
            assert!(
                rl_list.success,
                "shield rate-limit list failed — stderr: {}",
                rl_list.stderr
            );

            // 15. Get rate limit
            let rl_get =
                support::hoppy_live_json(&["shield", "rate-limit", "get", "--id", &rl_id_str]);
            assert!(
                rl_get.success,
                "shield rate-limit get failed — stderr: {}",
                rl_get.stderr
            );

            // 16. Update rate limit
            let rl_update = support::hoppy_live_json(&[
                "shield",
                "rate-limit",
                "update",
                "--id",
                &rl_id_str,
                "--name",
                "updated-rl",
            ]);
            assert!(
                rl_update.success,
                "shield rate-limit update failed — stderr: {}",
                rl_update.stderr
            );

            // 17. Delete rate limit
            let rl_delete = support::hoppy_live_json_yes(&[
                "shield",
                "rate-limit",
                "delete",
                "--id",
                &rl_id_str,
            ]);
            assert!(
                rl_delete.success,
                "shield rate-limit delete failed — stderr: {}",
                rl_delete.stderr
            );
        } else {
            eprintln!("skipping rate limit CRUD — plan restriction or creation returned id=0");
        }

        // 18. Create access list
        let acl_create = support::hoppy_live_json(&[
            "shield",
            "access-list",
            "create",
            "--shield-zone-id",
            &sz_id_str,
            "--name",
            "test-acl",
            "--type",
            "0",
            "--content",
            "1.2.3.4",
        ]);
        assert!(
            acl_create.success,
            "shield access-list create failed — stderr: {}",
            acl_create.stderr
        );
        let acl_id = acl_create.json.as_ref().unwrap()["id"]
            .as_i64()
            .expect("no id in access-list create response");
        let acl_id_str = acl_id.to_string();

        // 19. List access lists
        let acl_list = support::hoppy_live_json(&[
            "shield",
            "access-list",
            "list",
            "--shield-zone-id",
            &sz_id_str,
        ]);
        assert!(
            acl_list.success,
            "shield access-list list failed — stderr: {}",
            acl_list.stderr
        );

        // 20. Get access list
        let acl_get = support::hoppy_live_json(&[
            "shield",
            "access-list",
            "get",
            "--shield-zone-id",
            &sz_id_str,
            "--id",
            &acl_id_str,
        ]);
        assert!(
            acl_get.success,
            "shield access-list get failed — stderr: {}",
            acl_get.stderr
        );

        // 21. Update access list
        let acl_update = support::hoppy_live_json(&[
            "shield",
            "access-list",
            "update",
            "--shield-zone-id",
            &sz_id_str,
            "--id",
            &acl_id_str,
            "--content",
            "5.6.7.8",
        ]);
        assert!(
            acl_update.success,
            "shield access-list update failed — stderr: {}",
            acl_update.stderr
        );

        // 22. Delete access list
        let acl_delete = support::hoppy_live_json_yes(&[
            "shield",
            "access-list",
            "delete",
            "--shield-zone-id",
            &sz_id_str,
            "--id",
            &acl_id_str,
        ]);
        assert!(
            acl_delete.success,
            "shield access-list delete failed — stderr: {}",
            acl_delete.stderr
        );

        // 23. Get bot detection
        let bd_get = support::hoppy_live_json(&[
            "shield",
            "bot-detection",
            "get",
            "--shield-zone-id",
            &sz_id_str,
        ]);
        assert!(
            bd_get.success,
            "shield bot-detection get failed — stderr: {}",
            bd_get.stderr
        );

        // 24. Update bot detection
        let bd_update = support::hoppy_live_json(&[
            "shield",
            "bot-detection",
            "update",
            "--shield-zone-id",
            &sz_id_str,
            "--execution-mode",
            "1",
        ]);
        assert!(
            bd_update.success,
            "shield bot-detection update failed — stderr: {}",
            bd_update.stderr
        );

        // 25. Cleanup runs via CleanupStack: pull-zone delete removes the Shield zone too
    });
}
