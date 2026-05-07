mod support;

use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn hoppy_db_cmd(api_key: &str, db_url: &str) -> assert_cmd::Command {
    let mut cmd = support::hoppy_cmd();
    cmd.env("BUNNY_API_KEY", api_key);
    cmd.env("BUNNY_DATABASE_URL", db_url);
    cmd
}

#[tokio::test]
async fn db_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/databases"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("database/database_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "db", "list"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn db_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/databases/db_01HX0000000000000000000001"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("database/database_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "db",
            "get",
            "--id",
            "db_01HX0000000000000000000001",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn db_create_validates_slug_locally() {
    // No mock — local validation should reject before any HTTP.
    let output = hoppy_db_cmd("test-api-key", "http://127.0.0.1:1") // unreachable, but unused
        .args([
            "db",
            "create",
            "--slug",
            "wardrobe-assistants-admin", // 25 chars
            "--group",
            "group_01",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("slug too long"), "stderr was: {stderr}");
}

#[tokio::test]
async fn db_create_rejects_uppercase_slug() {
    let output = hoppy_db_cmd("test-api-key", "http://127.0.0.1:1")
        .args(["db", "create", "--slug", "MyApp", "--group", "group_01"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("lowercase"), "stderr was: {stderr}");
}

#[tokio::test]
async fn db_create_posts_payload() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/databases"))
        .and(body_json(
            serde_json::json!({"slug": "my-app", "group": "group_01"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("database/database_create.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args([
            "--format", "json", "db", "create", "--slug", "my-app", "--group", "group_01",
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
async fn db_token_mint_redacts_jwt_by_default() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/databases/db_01/auth/tokens"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("database/token_mint.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "db",
            "token",
            "mint",
            "--db-id",
            "db_01",
            "--authorization",
            "full-access",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("eyJhbGc"),
        "raw JWT leaked into default output: {stdout}"
    );
    assert!(
        stdout.contains("<set, length="),
        "expected redaction placeholder; got: {stdout}"
    );
    assert!(stdout.contains("full-access"));
}

#[tokio::test]
async fn db_token_mint_reveal_shows_jwt() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/databases/db_01/auth/tokens"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("database/token_mint.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args([
            "--reveal", "--format", "json", "db", "token", "mint", "--db-id", "db_01",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("eyJhbGc"),
        "--reveal should print the raw JWT, got: {stdout}"
    );
}

#[tokio::test]
async fn db_group_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("database/group_list.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "db", "group", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn db_config_show_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/config"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(support::fixture("database/config.json"), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["db", "config", "show"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("eu-west-1"));
    assert!(stdout.contains("\"DE\""));
}

#[tokio::test]
async fn db_ping_uses_database_url_and_mints_token() {
    let ping_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/pipeline"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("database/ping_ok.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&ping_server)
        .await;
    let ping_uri = format!("{}/", ping_server.uri());

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/databases/db_01"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!(
            r#"{{
                "database": {{
                    "name": "my-app",
                    "id": "db_01",
                    "url": "{ping_uri}",
                    "block_reads": false,
                    "block_writes": false,
                    "allow_attach": false,
                    "group_id": "g",
                    "group_name": "EU",
                    "is_schema": false,
                    "schema": null,
                    "version": "0.24.30",
                    "size_max": "0",
                    "current_size": "0"
                }}
            }}"#,
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/databases/db_01/auth/tokens"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("database/token_mint.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "db", "ping", "--id", "db_01"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"ok\": true"), "got: {stdout}");
}
