use super::support;

use wiremock::matchers::{body_json, header, method, path, query_param};
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
        .args(["--format", "json", "db", "config", "show"])
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

// ---------------------------------------------------------------------------
// iter-51: --format parity for db v2-style commands
// ---------------------------------------------------------------------------

#[tokio::test]
async fn db_active_usage_format_json_pascal_case() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v2/databases/active_usage"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("database/active_usage.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "db", "active-usage"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // PascalCase keys, not snake_case from the upstream API.
    assert!(stdout.contains("\"ActiveDb\""), "got: {stdout}");
    assert!(stdout.contains("\"TotalDb\""), "got: {stdout}");
    assert!(stdout.contains("\"TotalDbSize\""), "got: {stdout}");
    assert!(!stdout.contains("active_db"));
}

// ---------------------------------------------------------------------------
// iter-56: db config + db v2 list --format parity
// ---------------------------------------------------------------------------

async fn mount_config(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/v1/config"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(support::fixture("database/config.json"), "application/json"),
        )
        .mount(server)
        .await;
}

async fn mount_limits(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/v1/config/limits"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("database/config_limits.json"),
            "application/json",
        ))
        .mount(server)
        .await;
}

#[tokio::test]
async fn db_config_show_format_table() {
    let server = MockServer::start().await;
    mount_config(&server).await;
    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "db", "config", "show"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.trim_start().starts_with('{'), "got: {stdout}");
    assert!(stdout.contains("eu-west-1"), "got: {stdout}");
    assert!(stdout.contains("Frankfurt"), "got: {stdout}");
    assert!(stdout.contains("London"), "got: {stdout}");
    assert!(stderr.contains("Storage regions"), "stderr: {stderr}");
    assert!(stderr.contains("Primary regions"), "stderr: {stderr}");
    assert!(stderr.contains("Replica regions"), "stderr: {stderr}");
}

#[tokio::test]
async fn db_config_show_format_text() {
    let server = MockServer::start().await;
    mount_config(&server).await;
    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["--format", "text", "db", "config", "show"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("storage\teu-west-1"), "got: {stdout}");
    assert!(stdout.contains("primary\tDE\tFrankfurt"), "got: {stdout}");
    assert!(stdout.contains("replica\tUK\tLondon"), "got: {stdout}");
}

#[tokio::test]
async fn db_config_show_format_json() {
    let server = MockServer::start().await;
    mount_config(&server).await;
    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "db", "config", "show"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim_start().starts_with('{'), "got: {stdout}");
    assert!(stdout.contains("storage_region_available"));
}

#[tokio::test]
async fn db_config_limits_format_table() {
    let server = MockServer::start().await;
    mount_limits(&server).await;
    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "db", "config", "limits"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim_start().starts_with('{'), "got: {stdout}");
    assert!(stdout.contains("Current Databases"), "got: {stdout}");
    assert!(stdout.contains("Max Databases"), "got: {stdout}");
    assert!(stdout.contains("50"), "got: {stdout}");
}

#[tokio::test]
async fn db_config_limits_format_text() {
    let server = MockServer::start().await;
    mount_limits(&server).await;
    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["--format", "text", "db", "config", "limits"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("current_databases\t3"), "got: {stdout}");
    assert!(stdout.contains("max_databases\t50"), "got: {stdout}");
}

#[tokio::test]
async fn db_config_limits_format_json() {
    let server = MockServer::start().await;
    mount_limits(&server).await;
    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "db", "config", "limits"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim_start().starts_with('{'));
    assert!(stdout.contains("current_databases"));
}

async fn mount_v2_list(server: &MockServer, fixture: &str) {
    Mock::given(method("GET"))
        .and(path("/v2/databases"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(support::fixture(fixture), "application/json"),
        )
        .mount(server)
        .await;
}

#[tokio::test]
async fn db_v2_list_table_empty() {
    let server = MockServer::start().await;
    mount_v2_list(&server, "database/database_list_v2_empty.json").await;
    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "db", "v2", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No results."), "stderr: {stderr}");
    assert!(stderr.contains("0 total"), "stderr: {stderr}");
}

#[tokio::test]
async fn db_v2_list_table_one() {
    let server = MockServer::start().await;
    mount_v2_list(&server, "database/database_list_v2_one.json").await;
    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "db", "v2", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("alpha"), "got: {stdout}");
    assert!(stdout.contains("eu-west-1"), "got: {stdout}");
    assert!(stdout.contains("ID"), "got: {stdout}");
    assert!(stdout.contains("Name"), "got: {stdout}");
    assert!(!stdout.contains("<empty list>"), "got: {stdout}");
    assert!(!stdout.contains("<object:"), "got: {stdout}");
    assert!(stderr.contains("1 total"), "stderr: {stderr}");
}

#[tokio::test]
async fn db_v2_list_table_many() {
    let server = MockServer::start().await;
    mount_v2_list(&server, "database/database_list_v2_many.json").await;
    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "db", "v2", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("alpha"), "got: {stdout}");
    assert!(stdout.contains("beta"), "got: {stdout}");
    assert!(stdout.contains("gamma"), "got: {stdout}");
    // Row count: header + 3 data rows + framing — count occurrences of unique IDs.
    let row_count = stdout.matches("db_01HX").count();
    assert_eq!(row_count, 3, "expected 3 db rows, got: {stdout}");
}

#[tokio::test]
async fn db_v2_list_json_unchanged() {
    let server = MockServer::start().await;
    mount_v2_list(&server, "database/database_list_v2_many.json").await;
    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "db", "v2", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // JSON envelope still PascalCase with Databases + PageInfo.
    assert!(stdout.contains("\"Databases\""), "got: {stdout}");
    assert!(stdout.contains("\"PageInfo\""), "got: {stdout}");
    assert!(stdout.contains("\"HasMoreItems\""), "got: {stdout}");
}

#[tokio::test]
async fn db_active_usage_format_table_not_raw_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v2/databases/active_usage"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("database/active_usage.json"),
            "application/json",
        ))
        .mount(&server)
        .await;

    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "db", "active-usage"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Table should not be raw JSON.
    assert!(!stdout.trim_start().starts_with('{'), "got: {stdout}");
    assert!(stdout.contains("ActiveDb"), "got: {stdout}");
}

#[tokio::test]
async fn db_active_usage_format_text_pascal_case_keys() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v2/databases/active_usage"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("database/active_usage.json"),
            "application/json",
        ))
        .mount(&server)
        .await;

    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["--format", "text", "db", "active-usage"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // text uses tab-separated KEY<TAB>value lines, with PascalCase keys.
    assert!(stdout.contains("ActiveDb\t"), "got: {stdout}");
    assert!(stdout.contains("TotalDb\t"), "got: {stdout}");
    assert!(!stdout.contains("active_db"));
}

// iter-66: db fork now sends {slug, date}; optimal endpoints require a token

#[tokio::test]
async fn db_fork_sends_slug_and_date() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/databases/db_01/fork"))
        .and(body_json(serde_json::json!({
            "slug": "my-fork",
            "date": "2026-07-10T12:00:00Z",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("database/database_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args([
            "db",
            "fork",
            "--id",
            "db_01",
            "--target",
            "my-fork",
            "--date",
            "2026-07-10T12:00:00Z",
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
async fn db_fork_requires_date() {
    // Missing --date must be a clap usage error (exit 2), no HTTP call.
    let output = hoppy_db_cmd("test-api-key", "http://127.0.0.1:1")
        .args(["db", "fork", "--id", "db_01", "--target", "my-fork"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--date"), "got: {stderr}");
}

#[tokio::test]
async fn db_config_optimal_sends_cdn_server_token() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/config/optimal"))
        .and(query_param("cdn_server_token", "tok-abc"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"storage_region":{"id":"eu-west-1","name":"Europe West","group":"EU"},"primary_regions":[],"replica_regions":[]}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args(["db", "config", "optimal", "--cdn-server-token", "tok-abc"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn db_config_optimal_single_sends_cdn_server_token() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/config/optimal_single"))
        .and(query_param("cdn_server_token", "tok-xyz"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"storage_region":{"id":"eu-west-1","name":"Europe West","group":"EU"},"region":null}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = hoppy_db_cmd("test-api-key", &server.uri())
        .args([
            "db",
            "config",
            "optimal-single",
            "--cdn-server-token",
            "tok-xyz",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
