mod support;

use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn pull_zone_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_list_paginated.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "pull-zone", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn pull_zone_list_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_list_paginated.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "pull-zone", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn pull_zone_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "pull-zone", "get", "--id", "1001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn pull_zone_get_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "pull-zone", "get", "--id", "1001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn pull_zone_create_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
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
            "create",
            "--name",
            "test-zone",
            "--origin-url",
            "https://example.com",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn pull_zone_update() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001"))
        .and(header("AccessKey", "test-api-key"))
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
            "--origin-url",
            "https://new.example.com",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn pull_zone_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/pullzone/1001"))
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
            "pull-zone",
            "delete",
            "--id",
            "1001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn pull_zone_purge() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/purgeCache"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "pull-zone", "purge", "--id", "1001"])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn pull_zone_get_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/999999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(404).set_body_raw(
            support::fixture("core/error_not_found_storagezone.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "pull-zone", "get", "--id", "999999"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not_found") || stderr.contains("not found") || stderr.contains("404"));
}

#[tokio::test]
async fn pull_zone_update_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/999999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(404).set_body_raw(
            support::fixture("core/error_not_found_storagezone.json"),
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
            "999999",
            "--origin-url",
            "https://example.com",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not_found") || stderr.contains("not found") || stderr.contains("404"));
}

// ---------------------------------------------------------------------------
// URL purge E2E test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn purge_url_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/purge"))
        .and(header("AccessKey", "test-key"))
        .and(query_param("url", "https://cdn.example.com/index.html"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args(["purge", "--url", "https://cdn.example.com/index.html"]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Purged"));
}

// ---------------------------------------------------------------------------
// Pull Zone hostname & SSL E2E tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pull_zone_hostname_add() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/addHostname"))
        .and(header("AccessKey", "test-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "cdn.example.com" }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "hostname",
        "add",
        "--id",
        "1001",
        "--hostname",
        "cdn.example.com",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Added hostname"));
}

#[tokio::test]
async fn pull_zone_hostname_remove() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/pullzone/1001/removeHostname"))
        .and(header("AccessKey", "test-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "cdn.example.com" }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "hostname",
        "remove",
        "--id",
        "1001",
        "--hostname",
        "cdn.example.com",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Removed hostname"));
}

#[tokio::test]
async fn pull_zone_hostname_load_free_cert() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/loadFreeCertificate"))
        .and(header("AccessKey", "test-key"))
        .and(query_param("hostname", "cdn.example.com"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "hostname",
        "load-free-cert",
        "--hostname",
        "cdn.example.com",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Loaded free certificate"));
}

#[tokio::test]
async fn pull_zone_hostname_force_ssl() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pullzone/1001/setForceSSL"))
        .and(header("AccessKey", "test-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "cdn.example.com", "ForceSSL": true }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "hostname",
        "force-ssl",
        "--id",
        "1001",
        "--hostname",
        "cdn.example.com",
        "--enabled=true",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Force SSL enabled"));
}

#[tokio::test]
async fn pull_zone_hostname_remove_cert() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/pullzone/1001/removeCertificate"))
        .and(header("AccessKey", "test-key"))
        .and(body_json(
            serde_json::json!({ "Hostname": "cdn.example.com" }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let mut cmd = support::hoppy_mock_cmd("test-key", &server.uri());
    cmd.args([
        "pull-zone",
        "hostname",
        "remove-cert",
        "--id",
        "1001",
        "--hostname",
        "cdn.example.com",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Removed certificate"));
}

#[cfg(feature = "live-api")]
#[test]
fn live_pull_zone_lifecycle() {
    support::run_lifecycle(|cleanup| {
        let name = support::unique_name("hoppy-test");

        // 1. Create
        let create = support::hoppy_live_json(&[
            "pull-zone",
            "create",
            "--name",
            &name,
            "--origin-url",
            "https://example.com",
        ]);
        assert!(create.success, "create failed — stderr: {}", create.stderr);
        let id = create.json.as_ref().unwrap()["Id"].as_i64().unwrap();
        let id_str = id.to_string();

        // Register cleanup early so it runs even on panic
        cleanup.push(&["pull-zone", "delete", "--id", &id_str]);

        // 2. Get by id
        let get = support::hoppy_live_json(&["pull-zone", "get", "--id", &id_str]);
        assert!(get.success, "get failed — stderr: {}", get.stderr);

        // 3. List and verify zone appears
        let list = support::hoppy_live_json(&["pull-zone", "list"]);
        assert!(list.success, "list failed — stderr: {}", list.stderr);
        let zones = list.json.as_ref().unwrap()["Items"].as_array().unwrap();
        let found = zones.iter().any(|z| z["Id"].as_i64() == Some(id));
        assert!(found, "zone {id} not found in list output");

        // 4. Update origin URL
        let update = support::hoppy_live_json(&[
            "pull-zone",
            "update",
            "--id",
            &id_str,
            "--origin-url",
            "https://new.example.com",
        ]);
        assert!(update.success, "update failed — stderr: {}", update.stderr);

        // 5. Get again and verify OriginUrl changed
        let get2 = support::hoppy_live_json(&["pull-zone", "get", "--id", &id_str]);
        assert!(get2.success, "second get failed — stderr: {}", get2.stderr);
        let origin = get2.json.as_ref().unwrap()["OriginUrl"]
            .as_str()
            .unwrap_or("");
        assert!(
            origin.contains("new.example.com"),
            "expected updated OriginUrl, got: {origin}"
        );

        // 6. Purge
        let purge = support::hoppy_live_json(&["pull-zone", "purge", "--id", &id_str]);
        assert!(purge.success, "purge failed — stderr: {}", purge.stderr);

        // 7. Optimizer statistics
        let opt_stats = support::hoppy_live_json(&[
            "pull-zone",
            "statistics",
            "--id",
            &id_str,
            "--type",
            "optimizer",
        ]);
        assert!(
            opt_stats.success,
            "optimizer statistics failed — stderr: {}",
            opt_stats.stderr
        );

        // 8. Origin shield statistics
        let os_stats = support::hoppy_live_json(&[
            "pull-zone",
            "statistics",
            "--id",
            &id_str,
            "--type",
            "origin-shield",
        ]);
        assert!(
            os_stats.success,
            "origin-shield statistics failed — stderr: {}",
            os_stats.stderr
        );

        // 9. SafeHop statistics
        let sh_stats = support::hoppy_live_json(&[
            "pull-zone",
            "statistics",
            "--id",
            &id_str,
            "--type",
            "safehop",
        ]);
        assert!(
            sh_stats.success,
            "safehop statistics failed — stderr: {}",
            sh_stats.stderr
        );

        // 10. Cleanup runs via CleanupStack on exit
    });
}

#[tokio::test]
async fn pull_zone_statistics_optimizer_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/1001/optimizer/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_optimizer_statistics.json"),
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
            "statistics",
            "--id",
            "1001",
            "--type",
            "optimizer",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let _json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
}

#[tokio::test]
async fn pull_zone_statistics_origin_shield_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/1001/originshield/queuestatistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_originshield_statistics.json"),
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
            "statistics",
            "--id",
            "1001",
            "--type",
            "origin-shield",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let _json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
}

#[tokio::test]
async fn pull_zone_statistics_safehop_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/1001/safehop/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/pullzone_safehop_statistics.json"),
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
            "statistics",
            "--id",
            "1001",
            "--type",
            "safehop",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let _json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
}

#[cfg(feature = "live-api")]
#[test]
fn live_pull_zone_get_nonexistent() {
    let result = support::hoppy_live_json(&["pull-zone", "get", "--id", "999999999"]);
    assert!(
        !result.success,
        "expected failure for nonexistent pull zone"
    );
}

#[cfg(feature = "live-api")]
#[test]
fn live_pull_zone_update_nonexistent() {
    let result = support::hoppy_live_json(&[
        "pull-zone",
        "update",
        "--id",
        "999999999",
        "--origin-url",
        "https://example.com",
    ]);
    assert!(
        !result.success,
        "expected failure for nonexistent pull zone update"
    );
}
