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
}
