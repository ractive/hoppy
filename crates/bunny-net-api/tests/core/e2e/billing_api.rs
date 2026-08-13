use bunny_net_api::core::CoreClient;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_GET: &str = include_str!("../../../../../fixtures/core/billing_get.json");
const FIXTURE_UNAUTHORIZED: &str =
    include_str!("../../../../../fixtures/core/error_unauthorized.json");

fn test_client(uri: &str) -> CoreClient {
    CoreClient::with_base_url("test-api-key", uri)
}

#[tokio::test]
async fn get_billing_returns_balance_and_charges() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/billing"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let billing = test_client(&server.uri()).get_billing().await.unwrap();

    // Shape-first: verify the values parsed to reasonable types/ranges,
    // not hand-authored fixture values that drift with live recordings.
    assert!(billing.balance.is_finite());
    assert!(billing.balance >= 0.0);
    assert!(billing.this_month_charges.is_finite());
    assert!(billing.this_month_charges >= 0.0);

    // JSON-key presence: confirm the fields exist in the raw response so a
    // renamed key doesn't silently pass via serde's #[serde(default)] = 0.0.
    let json: serde_json::Value = serde_json::from_str(FIXTURE_GET).unwrap();
    assert!(
        json["Balance"].is_number(),
        "Balance key missing or not a number"
    );
    assert!(
        json["ThisMonthCharges"].is_number(),
        "ThisMonthCharges key missing or not a number"
    );
    assert!(
        json["BillingEnabled"].is_boolean(),
        "BillingEnabled key missing or not a bool"
    );
    assert!(
        json["AutomaticRechargeEnabled"].is_boolean(),
        "AutomaticRechargeEnabled key missing or not a bool"
    );

    // Card type and identifier are optional strings; presence is enough.
    assert!(billing.automatic_payment_card_type.is_some());
    assert!(billing.automatic_payment_identifier.is_some());

    // Billing records (iter-83): the fixture carries 3 entries (2x Type 3,
    // 1x Type 2) that BillingDetails must not silently drop.
    assert_eq!(billing.billing_records.len(), 3);
    for record in &billing.billing_records {
        assert!(record.id > 0, "billing record id must be positive");
    }
    assert_eq!(billing.billing_records[0].record_type, 3);
    assert_eq!(billing.billing_records[2].record_type, 2);
    assert!(!billing.billing_records[2].invoice_available);

    assert!(
        json["BillingRecords"].is_array(),
        "BillingRecords key missing or not an array"
    );
    assert_eq!(
        json["BillingRecords"][0]["Type"], 3,
        "raw Type key drifted from expected fixture value"
    );
}

#[tokio::test]
async fn get_billing_invalid_api_key_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/billing"))
        .respond_with(
            ResponseTemplate::new(401).set_body_raw(FIXTURE_UNAUTHORIZED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri()).get_billing().await.unwrap_err();
    let api_err = err.downcast_ref::<bunny_net_api::core::ApiError>().unwrap();
    assert_eq!(api_err.status_code, 401);
}

#[tokio::test]
async fn get_billing_partial_response_uses_defaults() {
    let server = MockServer::start().await;

    // Only Balance is present — all other fields should default.
    Mock::given(method("GET"))
        .and(path("/billing"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(r#"{"Balance": 100.0}"#, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let billing = test_client(&server.uri()).get_billing().await.unwrap();

    assert!((billing.balance - 100.0).abs() < f64::EPSILON);
    assert!((billing.this_month_charges - 0.0).abs() < f64::EPSILON);
    assert!(!billing.billing_enabled);
    assert!(!billing.automatic_recharge_enabled);
    assert!(billing.automatic_payment_card_type.is_none());
    assert!(billing.automatic_payment_identifier.is_none());
    assert_eq!(billing.monthly_bandwidth_used, 0);
    // No `BillingRecords` key in the response — `#[serde(default)]` must
    // yield an empty list rather than a deserialization error.
    assert!(billing.billing_records.is_empty());
}

// ---------------------------------------------------------------------------
// Billing-record document download (iter-83)
// ---------------------------------------------------------------------------

/// The pre-signed `DocumentDownloadUrl` lives on a different host
/// (`billing.b-cdn.net`) than the rest of the Core API. The download must
/// stream the body to the writer and, critically, must NOT send the
/// `AccessKey` header — sending it would leak the API key to that host.
#[tokio::test]
async fn download_billing_record_document_streams_without_access_key_header() {
    let server = MockServer::start().await;
    let pdf: &[u8] = b"%PDF-1.4\n%mock receipt\n%%EOF\n";

    Mock::given(method("GET"))
        .and(path("/invoice/4291069"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(pdf.to_vec(), "application/pdf"))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client("https://unused.invalid");
    let url = format!(
        "{}/invoice/4291069?token=tok_secret&expires=9999",
        server.uri()
    );
    let mut buf: Vec<u8> = Vec::new();
    let n = client
        .download_billing_record_document(&url, &mut buf)
        .await
        .unwrap();

    assert_eq!(n, pdf.len() as u64);
    assert_eq!(buf, pdf);

    let received = server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1);
    assert!(
        received[0].headers.get("AccessKey").is_none(),
        "AccessKey header must never be sent to the document-download host"
    );
}

/// A failing document download (e.g. an expired signed URL) must still
/// surface a structured/plain error without a panic, and must not carry the
/// request URL (and therefore its `token`) in the error text.
#[tokio::test]
async fn download_billing_record_document_error_status_is_surfaced() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/invoice/expired"))
        .respond_with(ResponseTemplate::new(403).set_body_raw(b"Forbidden".to_vec(), "text/plain"))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client("https://unused.invalid");
    let url = format!("{}/invoice/expired?token=tok_secret", server.uri());
    let mut buf: Vec<u8> = Vec::new();
    let err = client
        .download_billing_record_document(&url, &mut buf)
        .await
        .unwrap_err();

    assert!(
        !err.to_string().contains("tok_secret"),
        "signed URL token leaked into error message: {err}"
    );
}
