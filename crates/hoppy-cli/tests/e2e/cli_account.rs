//! CLI e2e coverage for the iter-75 account/admin command group:
//! `apikey list`, `billing {summary,payment-requests,invoice-pdf,
//! payment-request-pdf}`, `region list`, `country list`, `search`, and
//! `user audit`.

use super::support;

use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// apikey list
// ---------------------------------------------------------------------------

#[tokio::test]
async fn apikey_list_redacts_by_default_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/apikey"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/apikey_list.json"),
            "application/json",
        ))
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "apikey", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The raw key value must not appear; the placeholder must.
    assert!(
        !stdout.contains("a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6"),
        "raw key leaked into table output:\n{stdout}"
    );
    assert!(
        stdout.contains("<set, length="),
        "expected redaction placeholder, got:\n{stdout}"
    );
}

#[tokio::test]
async fn apikey_list_reveal_shows_raw_key() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/apikey"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/apikey_list.json"),
            "application/json",
        ))
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "apikey", "list", "--reveal"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json["Items"][0]["Key"], "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6",
        "expected raw key with --reveal"
    );
}

#[tokio::test]
async fn apikey_list_json_redacts_key_without_reveal() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/apikey"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/apikey_list.json"),
            "application/json",
        ))
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "apikey", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let key = json["Items"][0]["Key"].as_str().unwrap();
    assert!(
        key.starts_with("<set, length="),
        "expected redacted key in JSON without --reveal, got: {key}"
    );
}

// ---------------------------------------------------------------------------
// billing summary + payment requests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn billing_summary_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/billing/summary"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/billing_summary.json"),
            "application/json",
        ))
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "billing", "summary"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Pull Zone ID"));
    assert!(stdout.contains("500123"));
}

#[tokio::test]
async fn billing_payment_requests_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/billing/payment-requests"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/billing_payment_requests.json"),
            "application/json",
        ))
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "billing", "payment-requests"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.is_array());
    assert_eq!(json[0]["Id"], 90001);
    assert_eq!(json[1]["Paid"], false);
}

// ---------------------------------------------------------------------------
// invoice PDF downloads (streamed to file)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn billing_invoice_pdf_writes_file() {
    let server = MockServer::start().await;
    let pdf: &[u8] = b"%PDF-1.4\n%mock\n%%EOF\n";
    Mock::given(method("GET"))
        .and(path("/billing/summary/44001/pdf"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(pdf.to_vec(), "application/pdf"))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("invoice.pdf");
    let out_str = out_path.to_str().unwrap();

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "billing",
            "invoice-pdf",
            "--record-id",
            "44001",
            "--output",
            out_str,
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let written = std::fs::read(&out_path).unwrap();
    assert_eq!(written, pdf);
    assert!(written.starts_with(b"%PDF"));
}

#[tokio::test]
async fn billing_payment_request_pdf_writes_file() {
    let server = MockServer::start().await;
    let pdf: &[u8] = b"%PDF-1.5\n%mock pr\n%%EOF\n";
    Mock::given(method("GET"))
        .and(path("/billing/payment-request-invoice/90002/pdf"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(pdf.to_vec(), "application/pdf"))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("pr.pdf");
    let out_str = out_path.to_str().unwrap();

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "billing",
            "payment-request-pdf",
            "--id",
            "90002",
            "--output",
            out_str,
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let written = std::fs::read(&out_path).unwrap();
    assert_eq!(written, pdf);
}

// ---------------------------------------------------------------------------
// region + country reference data
// ---------------------------------------------------------------------------

#[tokio::test]
async fn region_list_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/region"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/region_list.json"),
            "application/json",
        ))
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "region", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Frankfurt"));
    assert!(stdout.contains("DE"));
}

#[tokio::test]
async fn country_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/country"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/country_list.json"),
            "application/json",
        ))
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "country", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.is_array());
    assert_eq!(json[0]["IsoCode"], "DE");
    assert_eq!(json[0]["IsEU"], true);
}

// ---------------------------------------------------------------------------
// global search
// ---------------------------------------------------------------------------

#[tokio::test]
async fn search_forwards_query_and_renders_results() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/search"))
        .and(query_param("search", "example"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/search_results.json"),
            "application/json",
        ))
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "search", "example"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PullZone"));
    assert!(stdout.contains("example-cdn"));
}

// ---------------------------------------------------------------------------
// user audit log
// ---------------------------------------------------------------------------

#[tokio::test]
async fn user_audit_forwards_filters_and_renders() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user/audit/2026-07-01"))
        .and(query_param("ResourceType", "PullZone"))
        .and(query_param("Order", "Descending"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(support::fixture("core/user_audit.json"), "application/json"),
        )
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "table",
            "user",
            "audit",
            "--date",
            "2026-07-01",
            "--resource-type",
            "PullZone",
            "--order",
            "descending",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Update"));
    assert!(stdout.contains("PullZone"));
}

#[tokio::test]
async fn user_audit_json_includes_pagination_fields() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user/audit/2026-07-01"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(support::fixture("core/user_audit.json"), "application/json"),
        )
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "user", "audit", "--date", "2026-07-01"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["HasMoreData"], true);
    assert!(json["Logs"].is_array());
    assert_eq!(json["ContinuationToken"], "next-page-token-abc");
}

// ---------------------------------------------------------------------------
// Live API smoke tests (feature-gated)
// ---------------------------------------------------------------------------

#[cfg(feature = "live-api")]
#[test]
fn live_apikey_list() {
    let result = support::hoppy_live_json(&["apikey", "list"]);
    assert!(
        result.success,
        "apikey list failed — stderr: {}",
        result.stderr
    );
    let json = result.json.as_ref().unwrap();
    assert!(json["Items"].is_array(), "expected an Items array");
}

#[cfg(feature = "live-api")]
#[test]
fn live_region_list() {
    let result = support::hoppy_live_json(&["region", "list"]);
    assert!(
        result.success,
        "region list failed — stderr: {}",
        result.stderr
    );
    assert!(
        result.json.as_ref().unwrap().is_array(),
        "expected a JSON array of regions"
    );
}

#[cfg(feature = "live-api")]
#[test]
fn live_country_list() {
    let result = support::hoppy_live_json(&["country", "list"]);
    assert!(
        result.success,
        "country list failed — stderr: {}",
        result.stderr
    );
    assert!(
        result.json.as_ref().unwrap().is_array(),
        "expected a JSON array of countries"
    );
}
