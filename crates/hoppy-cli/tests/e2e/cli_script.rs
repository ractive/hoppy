use super::support;

use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn script_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/compute/script"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("compute/scripts_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "script", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn script_list_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/compute/script"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("compute/scripts_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "script", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn script_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/compute/script/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("compute/script_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "script", "get", "--id", "1001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(json["Id"].is_number(), "expected Id to be a number");
    assert!(json["Name"].is_string(), "expected Name to be a string");
    assert!(
        json["ScriptType"].is_number(),
        "expected ScriptType to be a number"
    );
    assert!(
        json["EdgeScriptVariables"].is_array(),
        "expected EdgeScriptVariables to be an array"
    );
    assert!(
        json["Deleted"].is_boolean(),
        "expected Deleted to be a boolean"
    );
}

#[tokio::test]
async fn script_get_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/compute/script/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("compute/script_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "script", "get", "--id", "1001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Check column headers
    assert!(stdout.contains("ID"), "expected ID column");
    assert!(stdout.contains("Name"), "expected Name column");
    assert!(stdout.contains("Type"), "expected Type column");
    assert!(
        stdout.contains("Last Modified"),
        "expected Last Modified column"
    );
    assert!(
        stdout.contains("Monthly Cost"),
        "expected Monthly Cost column"
    );
    // At least one data row present beneath the header.
    let data_rows = stdout
        .lines()
        .filter(|l| support::DATA_ROW_RE.is_match(l))
        .count();
    assert!(
        data_rows >= 1,
        "expected at least one data row, got {data_rows} matching lines"
    );
}

#[tokio::test]
async fn script_create_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/compute/script"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
            support::fixture("compute/script_create.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "script",
            "create",
            "--name",
            "new-script",
            "--script-type",
            "cdn",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn script_update() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/compute/script/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("compute/script_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format", "json", "script", "update", "--id", "1001", "--name", "updated",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn script_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/compute/script/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes", "--format", "json", "script", "delete", "--id", "1001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn script_code_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/compute/script/1001/code"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("compute/script_code_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "script", "code", "get", "--id", "1001"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn script_code_update() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/compute/script/1001/code"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "script",
            "code",
            "update",
            "--id",
            "1001",
            "--code",
            "console.log('hi')",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn script_publish() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/compute/script/1001/publish"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "script",
            "publish",
            "--id",
            "1001",
            "--note",
            "test release",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn script_release_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/compute/script/1001/releases"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("compute/releases_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format", "json", "script", "release", "list", "--id", "1001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn script_release_get_active_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/compute/script/1001/releases/active"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("compute/release_active.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "script",
            "release",
            "get-active",
            "--id",
            "1001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn script_variable_add_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/compute/script/1001/variables/add"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
            support::fixture("compute/variable_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "script",
            "variable",
            "add",
            "--id",
            "1001",
            "--name",
            "MY_VAR",
            "--default-value",
            "hello",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn script_variable_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/compute/script/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("compute/script_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format", "json", "script", "variable", "list", "--id", "1001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(
        json.is_array(),
        "expected top-level JSON array of variables"
    );
    let vars = json.as_array().unwrap();
    assert!(!vars.is_empty(), "expected at least one variable");
    assert!(
        vars[0]["Id"].is_number(),
        "expected variable Id to be a number"
    );
    assert!(
        vars[0]["Name"].is_string(),
        "expected variable Name to be a string"
    );
    assert!(
        vars[0]["Required"].is_boolean(),
        "expected variable Required to be a boolean"
    );
}

#[tokio::test]
async fn script_variable_update() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/compute/script/1001/variables/201"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("compute/variable_update.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "script",
            "variable",
            "update",
            "--id",
            "1001",
            "--variable-id",
            "201",
            "--default-value",
            "updated",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn script_variable_upsert() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/compute/script/1001/variables"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("compute/variable_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "script",
            "variable",
            "upsert",
            "--id",
            "1001",
            "--name",
            "MY_VAR",
            "--default-value",
            "upserted",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn script_variable_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/compute/script/1001/variables/201"))
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
            "script",
            "variable",
            "delete",
            "--id",
            "1001",
            "--variable-id",
            "201",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn script_secret_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/compute/script/1001/secrets"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("compute/secrets_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format", "json", "script", "secret", "list", "--id", "1001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(json.is_array(), "expected top-level JSON array of secrets");
    let secrets = json.as_array().unwrap();
    assert!(!secrets.is_empty(), "expected at least one secret");
    assert!(
        secrets[0]["Id"].is_number(),
        "expected secret Id to be a number"
    );
    assert!(
        secrets[0]["Name"].is_string(),
        "expected secret Name to be a string"
    );
}

#[tokio::test]
async fn script_secret_add_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/compute/script/1001/secrets"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
            support::fixture("compute/secret_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "script",
            "secret",
            "add",
            "--id",
            "1001",
            "--name",
            "MY_SECRET",
            "--value",
            "secret123",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(json["Id"].is_number(), "expected Id to be a number");
    assert!(json["Name"].is_string(), "expected Name to be a string");
    assert!(
        json["LastModified"].is_string(),
        "expected LastModified to be a string"
    );
}

#[tokio::test]
async fn script_secret_update() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/compute/script/1001/secrets/401"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("compute/secret_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "script",
            "secret",
            "update",
            "--id",
            "1001",
            "--secret-id",
            "401",
            "--value",
            "updated-secret",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn script_secret_upsert() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/compute/script/1001/secrets"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("compute/secret_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "script",
            "secret",
            "upsert",
            "--id",
            "1001",
            "--name",
            "MY_SECRET_2",
            "--value",
            "upserted-secret",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn script_secret_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/compute/script/1001/secrets/401"))
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
            "script",
            "secret",
            "delete",
            "--id",
            "1001",
            "--secret-id",
            "401",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

// ---------------------------------------------------------------------------
// iter-69: script list filters + statistics load-latest
// ---------------------------------------------------------------------------

#[tokio::test]
async fn script_list_forwards_type_integration_and_linked_filters() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/compute/script"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("type", "1"))
        .and(query_param("integrationId", "77"))
        .and(query_param("includeLinkedPullZones", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("compute/scripts_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "script",
            "list",
            "--type",
            "1",
            "--integration-id",
            "77",
            "--include-linked-pullzones",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn script_statistics_forwards_load_latest() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/compute/script/1001/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("loadLatest", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("compute/statistics.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "script",
            "statistics",
            "--id",
            "1001",
            "--load-latest",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(feature = "live-api")]
#[test]
fn live_script_lifecycle() {
    support::run_lifecycle(|cleanup| {
        let name = support::unique_name("hpsc");

        // 1. Create script
        let create = support::hoppy_live_json(&[
            "script",
            "create",
            "--name",
            &name,
            "--script-type",
            "cdn",
        ]);
        assert!(create.success, "create failed: {}", create.stderr);
        let id = create.json.as_ref().unwrap()["Id"]
            .as_i64()
            .expect("Id missing from create response");
        let id_str = id.to_string();

        // Register cleanup early
        cleanup.push(&[
            "script",
            "delete",
            "--id",
            &id_str,
            "--delete-linked-pull-zones",
        ]);

        // 2. Get by id
        let get = support::hoppy_live_json(&["script", "get", "--id", &id_str]);
        assert!(get.success, "get failed: {}", get.stderr);
        assert_eq!(
            get.json.as_ref().unwrap()["Id"].as_i64(),
            Some(id),
            "Id mismatch in get response"
        );
        assert!(
            get.json.as_ref().unwrap()["Name"].as_str().is_some(),
            "Name missing from get response"
        );

        // 3. List — verify script appears
        let list = support::hoppy_live_json(&["script", "list"]);
        assert!(list.success, "list failed: {}", list.stderr);
        let items = list.json.as_ref().unwrap()["Items"]
            .as_array()
            .expect("list response should have Items array");
        let found = items.iter().any(|s| s["Id"].as_i64() == Some(id));
        assert!(found, "created script {id} not found in list");

        // 4. Update name
        let updated_name = format!("{name}-updated");
        let update = support::hoppy_live_json(&[
            "script",
            "update",
            "--id",
            &id_str,
            "--name",
            &updated_name,
        ]);
        assert!(update.success, "update failed: {}", update.stderr);

        // 5. Update code
        let code = "export default { async fetch(req) { return new Response('hello'); } }";
        let code_update = support::hoppy_live_json(&[
            "script", "code", "update", "--id", &id_str, "--code", code,
        ]);
        assert!(
            code_update.success,
            "code update failed: {}",
            code_update.stderr
        );

        // 6. Get code — stdout is plain text
        let code_get = support::hoppy_live_raw(&["script", "code", "get", "--id", &id_str]);
        assert!(code_get.success, "code get failed: {}", code_get.stderr);
        assert!(
            code_get.stdout.contains("hello"),
            "expected 'hello' in code output, got: {}",
            code_get.stdout
        );

        // 7. Publish
        let publish = support::hoppy_live_json(&[
            "script",
            "publish",
            "--id",
            &id_str,
            "--note",
            "test release",
        ]);
        assert!(publish.success, "publish failed: {}", publish.stderr);

        // 8. List releases — verify non-empty
        let releases = support::hoppy_live_json(&["script", "release", "list", "--id", &id_str]);
        assert!(releases.success, "release list failed: {}", releases.stderr);
        let release_items = releases.json.as_ref().unwrap()["Items"]
            .as_array()
            .expect("release list response should have Items array");
        assert!(
            !release_items.is_empty(),
            "expected at least one release after publish"
        );

        // 9. Delete handled by cleanup
    });
}

#[cfg(feature = "live-api")]
#[test]
fn live_script_variable_lifecycle() {
    support::run_lifecycle(|cleanup| {
        let name = support::unique_name("hpscv");

        // 1. Create script
        let create = support::hoppy_live_json(&[
            "script",
            "create",
            "--name",
            &name,
            "--script-type",
            "cdn",
        ]);
        assert!(create.success, "create failed: {}", create.stderr);
        let id = create.json.as_ref().unwrap()["Id"]
            .as_i64()
            .expect("Id missing from create response");
        let id_str = id.to_string();

        // Register cleanup early
        cleanup.push(&[
            "script",
            "delete",
            "--id",
            &id_str,
            "--delete-linked-pull-zones",
        ]);

        // 2. Add variable
        let add = support::hoppy_live_json(&[
            "script",
            "variable",
            "add",
            "--id",
            &id_str,
            "--name",
            "TEST_VAR",
            "--default-value",
            "hello",
        ]);
        assert!(add.success, "variable add failed: {}", add.stderr);

        // 3. List variables — verify TEST_VAR appears
        let list = support::hoppy_live_json(&["script", "variable", "list", "--id", &id_str]);
        assert!(list.success, "variable list failed: {}", list.stderr);
        let vars = list
            .json
            .as_ref()
            .unwrap()
            .as_array()
            .expect("variable list response should be an array");
        let found = vars.iter().any(|v| v["Name"].as_str() == Some("TEST_VAR"));
        assert!(found, "TEST_VAR not found in variable list");

        // 4. Extract variable id
        let vid = vars
            .iter()
            .find(|v| v["Name"].as_str() == Some("TEST_VAR"))
            .and_then(|v| v["Id"].as_i64())
            .expect("Id missing from TEST_VAR entry");
        let vid_str = vid.to_string();

        // 5. Update variable
        let update = support::hoppy_live_json(&[
            "script",
            "variable",
            "update",
            "--id",
            &id_str,
            "--variable-id",
            &vid_str,
            "--default-value",
            "world",
        ]);
        assert!(update.success, "variable update failed: {}", update.stderr);

        // 6. Upsert variable
        let upsert = support::hoppy_live_json(&[
            "script",
            "variable",
            "upsert",
            "--id",
            &id_str,
            "--name",
            "TEST_VAR2",
            "--default-value",
            "upserted",
        ]);
        assert!(upsert.success, "variable upsert failed: {}", upsert.stderr);

        // 7. Delete variable
        let delete = support::hoppy_live_json_yes(&[
            "script",
            "variable",
            "delete",
            "--id",
            &id_str,
            "--variable-id",
            &vid_str,
        ]);
        assert!(delete.success, "variable delete failed: {}", delete.stderr);

        // 8. Delete script handled by cleanup
    });
}

#[cfg(feature = "live-api")]
#[test]
fn live_script_secret_lifecycle() {
    support::run_lifecycle(|cleanup| {
        let name = support::unique_name("hpscs");

        // 1. Create script
        let create = support::hoppy_live_json(&[
            "script",
            "create",
            "--name",
            &name,
            "--script-type",
            "cdn",
        ]);
        assert!(create.success, "create failed: {}", create.stderr);
        let id = create.json.as_ref().unwrap()["Id"]
            .as_i64()
            .expect("Id missing from create response");
        let id_str = id.to_string();

        // Register cleanup early
        cleanup.push(&[
            "script",
            "delete",
            "--id",
            &id_str,
            "--delete-linked-pull-zones",
        ]);

        // 2. Add secret
        let add = support::hoppy_live_json(&[
            "script",
            "secret",
            "add",
            "--id",
            &id_str,
            "--name",
            "MY_SECRET",
            "--value",
            "s3cret",
        ]);
        assert!(add.success, "secret add failed: {}", add.stderr);

        // 3. List secrets — verify MY_SECRET appears
        let list = support::hoppy_live_json(&["script", "secret", "list", "--id", &id_str]);
        assert!(list.success, "secret list failed: {}", list.stderr);
        let secrets = list
            .json
            .as_ref()
            .unwrap()
            .as_array()
            .expect("secret list response should be an array");
        let found = secrets
            .iter()
            .any(|s| s["Name"].as_str() == Some("MY_SECRET"));
        assert!(found, "MY_SECRET not found in secret list");

        // 4. Extract secret id
        let sid = secrets
            .iter()
            .find(|s| s["Name"].as_str() == Some("MY_SECRET"))
            .and_then(|s| s["Id"].as_i64())
            .expect("Id missing from MY_SECRET entry");
        let sid_str = sid.to_string();

        // 5. Update secret
        let update = support::hoppy_live_json(&[
            "script",
            "secret",
            "update",
            "--id",
            &id_str,
            "--secret-id",
            &sid_str,
            "--value",
            "new-s3cret",
        ]);
        assert!(update.success, "secret update failed: {}", update.stderr);

        // 6. Upsert secret
        let upsert = support::hoppy_live_json(&[
            "script",
            "secret",
            "upsert",
            "--id",
            &id_str,
            "--name",
            "MY_SECRET2",
            "--value",
            "upserted",
        ]);
        assert!(upsert.success, "secret upsert failed: {}", upsert.stderr);

        // 7. Delete secret
        let delete = support::hoppy_live_json_yes(&[
            "script",
            "secret",
            "delete",
            "--id",
            &id_str,
            "--secret-id",
            &sid_str,
        ]);
        assert!(delete.success, "secret delete failed: {}", delete.stderr);

        // 8. Delete script handled by cleanup
    });
}
