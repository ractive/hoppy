mod e2e_support;

use e2e_support::{cmd, server};
use predicates::prelude::*;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

const FIXTURE_SCRIPTS_LIST: &str = include_str!("fixtures/compute/scripts_list.json");
const FIXTURE_SCRIPT_GET: &str = include_str!("fixtures/compute/script_get.json");
const FIXTURE_SCRIPT_CREATE: &str = include_str!("fixtures/compute/script_create.json");
const FIXTURE_SCRIPT_CODE_GET: &str = include_str!("fixtures/compute/script_code_get.json");
const FIXTURE_RELEASES_LIST: &str = include_str!("fixtures/compute/releases_list.json");
const FIXTURE_SECRETS_LIST: &str = include_str!("fixtures/compute/secrets_list.json");

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn script_list_table_output() {
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCRIPTS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["script", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("my-cdn-script"))
        .stdout(predicate::str::contains("my-dns-script"));
}

#[tokio::test]
async fn script_list_json_output() {
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCRIPTS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--format", "json", "script", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"CurrentPage\""))
        .stdout(predicate::str::contains("\"TotalItems\""));
}

// ---------------------------------------------------------------------------
// Get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn script_get_table_output() {
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCRIPT_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["script", "get", "--id", "1001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("my-cdn-script"))
        .stdout(predicate::str::contains("my-cdn-script.b-cdn.net"));
}

#[tokio::test]
async fn script_get_json_output() {
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/1001"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCRIPT_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--format", "json", "script", "get", "--id", "1001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"Id\": 1001"));
}

#[tokio::test]
async fn script_get_not_found() {
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/99999"))
        .respond_with(ResponseTemplate::new(404).set_body_raw(
            include_str!("fixtures/compute/error_not_found.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["script", "get", "--id", "99999"])
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// Create
// ---------------------------------------------------------------------------

#[tokio::test]
async fn script_create() {
    let mock = server::start().await;

    Mock::given(method("POST"))
        .and(path("/compute/script"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCRIPT_CREATE, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "script",
            "create",
            "--name",
            "new-script",
            "--script-type",
            "1",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("new-script"));
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

#[tokio::test]
async fn script_delete_with_yes_flag() {
    let mock = server::start().await;

    Mock::given(method("DELETE"))
        .and(path("/compute/script/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--yes", "script", "delete", "--id", "1001"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Deleted script 1001"));
}

// ---------------------------------------------------------------------------
// Code
// ---------------------------------------------------------------------------

#[tokio::test]
async fn script_code_get_table_output() {
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/1001/code"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCRIPT_CODE_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["script", "code", "get", "--id", "1001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("export default"));
}

#[tokio::test]
async fn script_code_get_json_output() {
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/1001/code"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCRIPT_CODE_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--format", "json", "script", "code", "get", "--id", "1001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"Code\""));
}

// ---------------------------------------------------------------------------
// Variable
// ---------------------------------------------------------------------------

#[tokio::test]
async fn script_variable_list() {
    let mock = server::start().await;

    // variable list fetches the script and reads EdgeScriptVariables from it
    Mock::given(method("GET"))
        .and(path("/compute/script/1001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCRIPT_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["script", "variable", "list", "--id", "1001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("API_URL"));
}

#[tokio::test]
async fn script_variable_get_json_output() {
    let mock = server::start().await;

    // variable list with --format json also goes through get_script
    Mock::given(method("GET"))
        .and(path("/compute/script/1001"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SCRIPT_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format", "json", "script", "variable", "list", "--id", "1001",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"API_URL\""));
}

// ---------------------------------------------------------------------------
// Secret
// ---------------------------------------------------------------------------

#[tokio::test]
async fn script_secret_list_table_output() {
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/1001/secrets"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SECRETS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["script", "secret", "list", "--id", "1001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("API_SECRET_KEY"))
        .stdout(predicate::str::contains("DATABASE_PASSWORD"));
}

#[tokio::test]
async fn script_secret_list_json_output() {
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/1001/secrets"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_SECRETS_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format", "json", "script", "secret", "list", "--id", "1001",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"API_SECRET_KEY\""));
}

// ---------------------------------------------------------------------------
// Release
// ---------------------------------------------------------------------------

#[tokio::test]
async fn script_release_list_table_output() {
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/1001/releases"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_RELEASES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["script", "release", "list", "--id", "1001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initial release"))
        .stdout(predicate::str::contains("Bug fix release"));
}

#[tokio::test]
async fn script_release_list_json_output() {
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/compute/script/1001/releases"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_RELEASES_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format", "json", "script", "release", "list", "--id", "1001",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"CurrentPage\""))
        .stdout(predicate::str::contains("\"TotalItems\""));
}
