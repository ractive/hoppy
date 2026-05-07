use bunny_api_core::CoreClient;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_GET: &str = include_str!("../../../../fixtures/core/billing_get.json");
const FIXTURE_UNAUTHORIZED: &str =
    include_str!("../../../../fixtures/core/error_unauthorized.json");

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

    assert!((billing.balance - 42.50).abs() < f64::EPSILON);
    assert!((billing.this_month_charges - 7.12).abs() < f64::EPSILON);
    assert!(billing.billing_enabled);
    assert!(billing.automatic_recharge_enabled);
    assert_eq!(billing.automatic_payment_card_type.as_deref(), Some("Visa"));
    assert_eq!(
        billing.automatic_payment_identifier.as_deref(),
        Some("****1234")
    );
    assert_eq!(billing.monthly_bandwidth_used, 10_737_418_240);
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
    let api_err = err.downcast_ref::<bunny_api_core::ApiError>().unwrap();
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
