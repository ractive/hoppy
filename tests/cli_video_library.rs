mod support;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn video_library_drm_statistics_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/videolibrary/42/drm/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/videolibrary_drm_statistics.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "video-library",
            "drm-statistics",
            "--id",
            "42",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let _json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
}

#[tokio::test]
async fn video_library_transcribing_statistics_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/videolibrary/42/transcribing/statistics"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/videolibrary_transcribing_statistics.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "video-library",
            "transcribing-statistics",
            "--id",
            "42",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let _json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
}
