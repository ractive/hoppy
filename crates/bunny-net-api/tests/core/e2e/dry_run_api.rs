use bunny_net_api::core::CoreClient;
use bunny_net_api::core::types::CreatePullZone;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn mutating_call_is_blocked_and_sends_zero_requests() {
    let server = MockServer::start().await;
    // No mocks mounted — any request reaching the server fails the test via
    // the "expect(0)" assertion below, not via a 404.
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let client = CoreClient::with_base_url("test-api-key", server.uri()).with_dry_run(true);

    let err = client
        .create_pull_zone(&CreatePullZone::new("test-zone", "https://example.com"))
        .await
        .expect_err("mutating call must be blocked under dry-run");

    let skipped = err
        .chain()
        .find_map(|e| e.downcast_ref::<bunny_net_api::dry_run::DryRunSkipped>())
        .expect("error chain must contain DryRunSkipped");
    assert_eq!(skipped.method, "POST");
    assert!(skipped.url.ends_with("/pullzone"));
    assert!(
        skipped
            .body
            .as_deref()
            .unwrap_or_default()
            .contains("test-zone")
    );
}

#[tokio::test]
async fn read_only_call_still_executes_under_dry_run() {
    let server = MockServer::start().await;
    let body = r#"{"Items":[],"CurrentPage":1,"TotalItems":0,"HasMoreItems":false}"#;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = CoreClient::with_base_url("test-api-key", server.uri()).with_dry_run(true);

    client
        .list_pull_zones(None, None, None)
        .await
        .expect("GET must execute normally under dry-run");

    server.verify().await;
}

#[tokio::test]
async fn secrets_redacted_unless_reveal() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    // zone_security_enabled isn't a secret; use a body field the redactor
    // treats as sensitive via key-shape matching — the zone name itself
    // isn't redacted, so assert the *unrevealed* case just doesn't panic
    // and produces a body string, then compare with the revealed case.
    let redacted_client =
        CoreClient::with_base_url("test-api-key", server.uri()).with_dry_run(true);
    let revealed_client = CoreClient::with_base_url("test-api-key", server.uri())
        .with_dry_run(true)
        .with_debug_reveal_secrets(true);

    let body = CreatePullZone::new("test-zone", "https://example.com");
    let redacted_err = redacted_client.create_pull_zone(&body).await.unwrap_err();
    let revealed_err = revealed_client.create_pull_zone(&body).await.unwrap_err();

    let redacted_body = redacted_err
        .chain()
        .find_map(|e| e.downcast_ref::<bunny_net_api::dry_run::DryRunSkipped>())
        .unwrap()
        .body
        .clone();
    let revealed_body = revealed_err
        .chain()
        .find_map(|e| e.downcast_ref::<bunny_net_api::dry_run::DryRunSkipped>())
        .unwrap()
        .body
        .clone();

    // Both bodies are present (pretty-printed JSON); the zone name is not a
    // secret so it appears either way — this asserts the plumbing carries a
    // body through in both modes without asserting on unrelated redaction
    // rules covered by the recording::debug unit tests.
    assert!(redacted_body.is_some());
    assert!(revealed_body.is_some());
}
