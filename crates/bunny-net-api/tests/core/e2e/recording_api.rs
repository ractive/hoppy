use bunny_net_api::core::CoreClient;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_GET: &str = include_str!("../../../../../fixtures/core/billing_get.json");

#[tokio::test]
async fn with_record_writes_under_domain_subdir_and_is_idempotent() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/billing"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .mount(&server)
        .await;

    let tmp = tempfile::tempdir().unwrap();
    let client = CoreClient::with_base_url("test-api-key", server.uri()).with_record(tmp.path());

    client.get_billing().await.unwrap();

    let recorded = tmp.path().join("core").join("GET_billing.json");
    assert!(
        recorded.exists(),
        "fixture not written at {}",
        recorded.display()
    );

    let mtime1 = std::fs::metadata(&recorded).unwrap().modified().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(25));

    // Second identical call must not rewrite the file.
    client.get_billing().await.unwrap();
    let mtime2 = std::fs::metadata(&recorded).unwrap().modified().unwrap();
    assert_eq!(
        mtime1, mtime2,
        "identical response rewrote the fixture (idempotency broken)"
    );
}
