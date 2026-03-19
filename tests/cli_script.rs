mod support;

use wiremock::matchers::{header, method, path};
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
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
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
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
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
            "1",
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
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
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
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
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
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
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
