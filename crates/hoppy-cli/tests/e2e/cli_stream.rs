use super::support;

use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Library tests (core API, AccessKey = api key)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stream_library_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/videolibrary"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/videolibrary_list_paginated.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "stream", "library", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn stream_library_list_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/videolibrary"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/videolibrary_list_paginated.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "table", "stream", "library", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn stream_library_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/videolibrary/10001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/videolibrary_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format", "json", "stream", "library", "get", "--id", "10001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(json["Id"].is_number(), "expected Id to be a number");
    assert!(json["Name"].is_string(), "expected Name to be a string");
    assert!(
        json["HasWatermark"].is_boolean(),
        "expected HasWatermark to be a boolean"
    );
    assert!(
        json["EnableMP4Fallback"].is_boolean(),
        "expected EnableMP4Fallback to be a boolean"
    );
    assert!(
        json["EnabledResolutions"].is_string(),
        "expected EnabledResolutions to be a string"
    );
    assert!(
        json["ReplicationRegions"].is_array(),
        "expected ReplicationRegions to be an array"
    );
}

#[tokio::test]
async fn stream_library_get_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/videolibrary/10001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/videolibrary_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format", "table", "stream", "library", "get", "--id", "10001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Check column headers
    assert!(stdout.contains("ID"), "expected ID column");
    assert!(stdout.contains("Name"), "expected Name column");
    assert!(stdout.contains("Videos"), "expected Videos column");
    assert!(
        stdout.contains("MP4 Fallback"),
        "expected MP4 Fallback column"
    );
    assert!(
        stdout.contains("Resolutions"),
        "expected Resolutions column"
    );
    assert!(stdout.contains("Created"), "expected Created column");
    // At least one data row present beneath the header.
    let data_rows = stdout
        .lines()
        .filter(|l| support::DATA_ROW_RE.is_match(l))
        .count();
    assert!(
        data_rows >= 1,
        "expected at least one data row, got {data_rows} matching lines"
    );
}

#[tokio::test]
async fn stream_library_create_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/videolibrary"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
            support::fixture("core/videolibrary_create.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format", "json", "stream", "library", "create", "--name", "test-lib",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn stream_library_update() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/videolibrary/10001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/videolibrary_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "stream",
            "library",
            "update",
            "--id",
            "10001",
            "--name",
            "updated-lib",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn stream_library_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/videolibrary/10001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes", "--format", "json", "stream", "library", "delete", "--id", "10001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn stream_library_reset_api_key_refetches_and_redacts() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/videolibrary/10001/resetApiKey"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/videolibrary/10001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/videolibrary_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--yes",
            "--format",
            "json",
            "stream",
            "library",
            "reset-api-key",
            "--id",
            "10001",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("REDACTED-STREAM-API-KEY"),
        "raw api key leaked into default output: {stdout}"
    );
    assert!(
        stdout.contains("<set, length="),
        "expected redacted placeholder, got: {stdout}"
    );
}

#[tokio::test]
async fn stream_library_reset_read_only_api_key_reveals_when_requested() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/videolibrary/10001/resetReadOnlyApiKey"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/videolibrary/10001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/videolibrary_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--reveal",
            "--yes",
            "--format",
            "json",
            "stream",
            "library",
            "reset-read-only-api-key",
            "--id",
            "10001",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("REDACTED-STREAM-READONLY-KEY"),
        "--reveal should surface the raw read-only key, got: {stdout}"
    );
}

/// `stream library get` without `--reveal` redacts ApiKey / ReadOnlyApiKey
/// in JSON and never leaks the raw token to stdout.
#[tokio::test]
async fn stream_library_get_redacts_api_keys_by_default() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/videolibrary/10001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/videolibrary_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format", "json", "stream", "library", "get", "--id", "10001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("REDACTED-STREAM-API-KEY"),
        "raw ApiKey leaked without --reveal: {stdout}"
    );
    assert!(
        !stdout.contains("REDACTED-STREAM-READONLY-KEY"),
        "raw ReadOnlyApiKey leaked without --reveal: {stdout}"
    );
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    let api_key = json["ApiKey"].as_str().expect("ApiKey should be a string");
    assert!(
        api_key.starts_with("<set, length="),
        "expected redaction placeholder, got {api_key}"
    );
    let ro = json["ReadOnlyApiKey"]
        .as_str()
        .expect("ReadOnlyApiKey should be a string");
    assert!(
        ro.starts_with("<set, length="),
        "expected redaction placeholder, got {ro}"
    );
}

/// `stream library get --reveal` prints the raw ApiKey / ReadOnlyApiKey.
#[tokio::test]
async fn stream_library_get_reveals_api_keys_when_requested() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/videolibrary/10001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/videolibrary_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--reveal", "--format", "json", "stream", "library", "get", "--id", "10001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert_eq!(
        json["ApiKey"].as_str(),
        Some("REDACTED-STREAM-API-KEY"),
        "expected raw ApiKey from fixture under --reveal"
    );
    assert_eq!(
        json["ReadOnlyApiKey"].as_str(),
        Some("REDACTED-STREAM-READONLY-KEY"),
        "expected raw ReadOnlyApiKey from fixture under --reveal"
    );
}

/// `stream library get --format text` redacts API keys in the row-oriented
/// output too — the redact policy applies across formats.
#[tokio::test]
async fn stream_library_get_text_redacts_api_keys_by_default() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/videolibrary/10001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/videolibrary_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format", "text", "stream", "library", "get", "--id", "10001",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("REDACTED-STREAM-API-KEY"),
        "raw ApiKey leaked in text output without --reveal: {stdout}"
    );
    assert!(
        stdout.contains("ApiKey") || stdout.contains("API Key"),
        "expected ApiKey row in text output: {stdout}"
    );
    assert!(
        stdout.contains("<set, length="),
        "expected redaction placeholder in text output: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// Collection tests (stream API, AccessKey = stream key)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stream_collection_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/library/10001/collections"))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/collection_list_paginated.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "collection",
        "list",
        "--library-id",
        "10001",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn stream_collection_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/library/10001/collections/col-guid-0001"))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/collection_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "collection",
        "get",
        "--library-id",
        "10001",
        "--collection-id",
        "col-guid-0001",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn stream_collection_create_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/library/10001/collections"))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
            support::fixture("stream/collection_create.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "collection",
        "create",
        "--library-id",
        "10001",
        "--name",
        "New Collection",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn stream_collection_update() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/library/10001/collections/col-guid-0001"))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/collection_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "collection",
        "update",
        "--library-id",
        "10001",
        "--collection-id",
        "col-guid-0001",
        "--name",
        "updated",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn stream_collection_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/library/10001/collections/col-guid-0001"))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_delete_status.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--yes",
        "--format",
        "json",
        "stream",
        "collection",
        "delete",
        "--library-id",
        "10001",
        "--collection-id",
        "col-guid-0001",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
}

// ---------------------------------------------------------------------------
// Caption tests (stream API)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stream_video_caption_add_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/library/10001/videos/vid-guid-0001/captions/en"))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_caption_add.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    // Write a temporary SRT file using tempfile to avoid cross-test interference
    let mut srt_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut srt_file, b"1\n00:00:00,000 --> 00:00:05,000\nHello\n").unwrap();

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "video",
        "caption",
        "add",
        "--library-id",
        "10001",
        "--video-id",
        "vid-guid-0001",
        "--srclang",
        "en",
        "--file",
        srt_file.path().to_str().unwrap(),
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn stream_video_caption_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/library/10001/videos/vid-guid-0001/captions/en"))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_caption_delete.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "video",
        "caption",
        "delete",
        "--library-id",
        "10001",
        "--video-id",
        "vid-guid-0001",
        "--srclang",
        "en",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn stream_library_statistics_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/library/12345/statistics"))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/library_statistics.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "library",
        "statistics",
        "--id",
        "12345",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    let _json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
}

// ---------------------------------------------------------------------------
// Video processing tests (stream API)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stream_video_transcribe_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/transcribe",
        ))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_transcribe_status.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "video",
        "transcribe",
        "--library-id",
        "10001",
        "--video-id",
        "aaaabbbb-1111-2222-3333-ccccddddeeee",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn stream_video_heatmap_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/heatmap",
        ))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_heatmap.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "video",
        "heatmap",
        "--library-id",
        "10001",
        "--video-id",
        "aaaabbbb-1111-2222-3333-ccccddddeeee",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert_eq!(json["heatmap"]["0"], 80);
    assert_eq!(json["heatmap"]["1"], 100);
    assert_eq!(json["heatmap"]["4"], 20);
}

#[tokio::test]
async fn stream_video_reencode_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/reencode",
        ))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_reencode.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "video",
        "reencode",
        "--library-id",
        "10001",
        "--video-id",
        "aaaabbbb-1111-2222-3333-ccccddddeeee",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn stream_video_reencode_codec_json() {
    let server = MockServer::start().await;
    // hevc = codec id 2
    Mock::given(method("PUT"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/outputs/2",
        ))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_reencode.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "video",
        "reencode",
        "--library-id",
        "10001",
        "--video-id",
        "aaaabbbb-1111-2222-3333-ccccddddeeee",
        "--codec",
        "hevc",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn stream_video_repackage_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/repackage",
        ))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_repackage.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "video",
        "repackage",
        "--library-id",
        "10001",
        "--video-id",
        "aaaabbbb-1111-2222-3333-ccccddddeeee",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn stream_video_smart_generate_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/smart",
        ))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_smart_status.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "video",
        "smart-generate",
        "--library-id",
        "10001",
        "--video-id",
        "aaaabbbb-1111-2222-3333-ccccddddeeee",
        "--generate-title",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn stream_video_set_thumbnail_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/thumbnail",
        ))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_thumbnail_status.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "video",
        "set-thumbnail",
        "--library-id",
        "10001",
        "--video-id",
        "aaaabbbb-1111-2222-3333-ccccddddeeee",
        "--thumbnail-url",
        "https://example.com/thumb.jpg",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn stream_video_resolutions_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/resolutions",
        ))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_resolutions.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "video",
        "resolutions",
        "list",
        "--library-id",
        "10001",
        "--video-id",
        "aaaabbbb-1111-2222-3333-ccccddddeeee",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn stream_video_resolutions_cleanup_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/resolutions/cleanup",
        ))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_resolutions_cleanup_status.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--yes",
        "--format",
        "json",
        "stream",
        "video",
        "resolutions",
        "cleanup",
        "--library-id",
        "10001",
        "--video-id",
        "aaaabbbb-1111-2222-3333-ccccddddeeee",
        "--delete-original",
        "--dry-run",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn stream_video_storage_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/storage",
        ))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_storage.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "video",
        "storage",
        "--library-id",
        "10001",
        "--video-id",
        "aaaabbbb-1111-2222-3333-ccccddddeeee",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

// ---------------------------------------------------------------------------
// Library settings / referrer / watermark / languages (iter-73)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stream_library_update_full_settings_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/videolibrary/10001"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "EnabledResolutions": "240p,720p,1080p",
            "WebhookUrl": "https://example.com/hook",
            "EnableDRM": true,
        })))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/videolibrary_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "stream",
            "library",
            "update",
            "--id",
            "10001",
            "--enabled-resolutions",
            "240p,720p,1080p",
            "--webhook-url",
            "https://example.com/hook",
            "--enable-drm",
            "true",
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
async fn stream_library_update_requires_a_flag() {
    // No mock server call should happen; the CLI must bail before any request.
    let server = MockServer::start().await;
    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["stream", "library", "update", "--id", "10001"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("at least one update flag"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn stream_library_referrer_allow() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/videolibrary/10001/addAllowedReferrer"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({ "Hostname": "example.com" })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--format",
            "json",
            "stream",
            "library",
            "referrer",
            "allow",
            "--id",
            "10001",
            "--value",
            "example.com",
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
async fn stream_library_referrer_block() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/videolibrary/10001/addBlockedReferrer"))
        .and(header("AccessKey", "test-api-key"))
        .and(body_json(serde_json::json!({ "Hostname": "bad.example" })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "stream",
            "library",
            "referrer",
            "block",
            "--id",
            "10001",
            "--value",
            "bad.example",
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
async fn stream_library_watermark_set_streams_file() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/videolibrary/10001/watermark"))
        .and(header("AccessKey", "test-api-key"))
        .and(header("Content-Type", "application/octet-stream"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    // Write a small "image" file to a temp path.
    let dir = std::env::temp_dir();
    let path = dir.join(format!("hoppy-watermark-{}.png", std::process::id()));
    std::fs::write(&path, [0x89u8, 0x50, 0x4e, 0x47]).unwrap();

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args([
            "--quiet",
            "stream",
            "library",
            "watermark",
            "set",
            "--id",
            "10001",
            "--file",
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let _ = std::fs::remove_file(&path);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn stream_library_watermark_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/videolibrary/10001/watermark"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["stream", "library", "watermark", "delete", "--id", "10001"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn stream_library_languages_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/videolibrary/languages"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/videolibrary_languages.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "stream", "library", "languages"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(json.is_array(), "expected a JSON array of languages");
    assert_eq!(json[0]["ShortName"].as_str(), Some("en"));
}

// ---------------------------------------------------------------------------
// Video metadata update + player-facing endpoints (iter-73)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stream_video_update_config_json_chapters() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/library/10001/videos/vid-guid-0001"))
        .and(header("AccessKey", "mock-stream-key"))
        .and(body_json(serde_json::json!({
            "Chapters": [{ "title": "Intro", "start": 0, "end": 30 }],
            "MetaTags": [{ "property": "topic", "value": "demo" }],
        })))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_upload_status.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;
    // Follow-up GET to render the updated video.
    Mock::given(method("GET"))
        .and(path("/library/10001/videos/vid-guid-0001"))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_get.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let dir = std::env::temp_dir();
    let cfg_path = dir.join(format!("hoppy-vidmeta-{}.json", std::process::id()));
    std::fs::write(
        &cfg_path,
        r#"{"chapters":[{"title":"Intro","start":0,"end":30}],"metaTags":[{"property":"topic","value":"demo"}]}"#,
    )
    .unwrap();

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "video",
        "update",
        "--library-id",
        "10001",
        "--video-id",
        "vid-guid-0001",
        "--config-json",
        cfg_path.to_str().unwrap(),
    ])
    .output()
    .unwrap();

    let _ = std::fs::remove_file(&cfg_path);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn stream_video_oembed_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/OEmbed"))
        .and(header("AccessKey", "mock-stream-key"))
        .and(query_param(
            "url",
            "https://iframe.mediadelivery.net/play/10001/aaaabbbb",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_oembed.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "stream",
        "video",
        "oembed",
        "--id",
        "10001",
        "--url",
        "https://iframe.mediadelivery.net/play/10001/aaaabbbb",
    ])
    .output()
    .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert_eq!(json["type"].as_str(), Some("video"));
    assert!(json["html"].as_str().unwrap().contains("iframe"));
}

#[tokio::test]
async fn stream_video_play_data_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/play",
        ))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_play_data.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "stream",
        "video",
        "play-data",
        "--library-id",
        "10001",
        "--video-id",
        "aaaabbbb-1111-2222-3333-ccccddddeeee",
    ])
    .output()
    .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(json["videoPlaylistUrl"].is_string());
}

#[tokio::test]
async fn stream_video_play_heatmap_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/play/heatmap",
        ))
        .and(header("AccessKey", "mock-stream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("stream/video_heatmap.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        None,
        Some(&server.uri()),
        None,
    )
    .args([
        "--format",
        "json",
        "stream",
        "video",
        "play-heatmap",
        "--library-id",
        "10001",
        "--video-id",
        "aaaabbbb-1111-2222-3333-ccccddddeeee",
    ])
    .output()
    .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------------
// Live API lifecycle tests
// ---------------------------------------------------------------------------

#[cfg(feature = "live-api")]
#[test]
fn live_stream_library_lifecycle() {
    support::run_lifecycle(|cleanup| {
        let name = support::unique_name("hoppy-test-lib");

        // 1. Create library
        let create = support::hoppy_live_json(&["stream", "library", "create", "--name", &name]);
        assert!(create.success, "create failed — stderr: {}", create.stderr);
        let id = create.json.as_ref().unwrap()["Id"].as_i64().unwrap();
        let id_str = id.to_string();

        // Register cleanup early so it runs even on panic
        cleanup.push(&["stream", "library", "delete", "--id", &id_str]);

        // 2. Get by id
        let get = support::hoppy_live_json(&["stream", "library", "get", "--id", &id_str]);
        assert!(get.success, "get failed — stderr: {}", get.stderr);
        assert_eq!(
            get.json.as_ref().unwrap()["Id"].as_i64(),
            Some(id),
            "get returned wrong Id"
        );
        assert_eq!(
            get.json.as_ref().unwrap()["Name"].as_str(),
            Some(name.as_str()),
            "get returned wrong Name"
        );

        // 3. List and verify library appears
        let list = support::hoppy_live_json(&["stream", "library", "list"]);
        assert!(list.success, "list failed — stderr: {}", list.stderr);
        let found = list.json.as_ref().unwrap()["Items"]
            .as_array()
            .map(|arr| arr.iter().any(|lib| lib["Id"].as_i64() == Some(id)))
            .unwrap_or(false);
        assert!(found, "library {id} not found in list output");

        // 4. Update name
        let updated_name = format!("{name}-updated");
        let update = support::hoppy_live_json(&[
            "stream",
            "library",
            "update",
            "--id",
            &id_str,
            "--name",
            &updated_name,
        ]);
        assert!(update.success, "update failed — stderr: {}", update.stderr);

        // 5. Get and verify Name changed
        let get2 = support::hoppy_live_json(&["stream", "library", "get", "--id", &id_str]);
        assert!(get2.success, "second get failed — stderr: {}", get2.stderr);
        assert_eq!(
            get2.json.as_ref().unwrap()["Name"].as_str(),
            Some(updated_name.as_str()),
            "Name was not updated"
        );

        // 6. Stream library statistics
        let lib_stats =
            support::hoppy_live_json(&["stream", "library", "statistics", "--id", &id_str]);
        assert!(
            lib_stats.success,
            "stream library statistics failed — stderr: {}",
            lib_stats.stderr
        );

        // 7. Video library DRM statistics (via core API)
        let drm_stats =
            support::hoppy_live_json(&["video-library", "drm-statistics", "--id", &id_str]);
        assert!(
            drm_stats.success,
            "video library DRM statistics failed — stderr: {}",
            drm_stats.stderr
        );

        // 8. Video library transcribing statistics (via core API)
        let tx_stats = support::hoppy_live_json(&[
            "video-library",
            "transcribing-statistics",
            "--id",
            &id_str,
        ]);
        assert!(
            tx_stats.success,
            "video library transcribing statistics failed — stderr: {}",
            tx_stats.stderr
        );

        // 9. Cleanup runs via CleanupStack on exit (delete with --yes)
    });
}

/// Create a stream collection, retrying a handful of times on HTTP 401.
///
/// Immediately after `POST /videolibrary` returns, the new library's
/// per-library AccessKey is valid against bunny.net's Core API but hasn't
/// yet propagated to the Stream API (`video.bunnycdn.com`) — requests in
/// that window come back with a bare `401 Unauthorized`, not a 404 or a
/// feature-gate error. Verified empirically: 3/3 fresh libraries hit a 401
/// on the very first collection-create attempt, then succeeded 2-6 seconds
/// later. This is a real (short-lived) eventual-consistency window on
/// bunny.net's side, not a client bug — `resolve_stream_client` already
/// resolves and uses the correct per-library key (see
/// `hoppy-knowledgebase/backlog/live-stream-collection-401.md`).
#[cfg(feature = "live-api")]
#[allow(dead_code)]
fn create_collection_with_retry(lib_id: &str, name: &str) -> support::LiveResult {
    const MAX_ATTEMPTS: u32 = 5;
    let args = [
        "stream",
        "collection",
        "create",
        "--library-id",
        lib_id,
        "--name",
        name,
    ];
    let mut result = support::hoppy_live_json(&args);
    let mut attempt = 1;
    while !result.success && result.stderr.contains("401") && attempt < MAX_ATTEMPTS {
        std::thread::sleep(std::time::Duration::from_secs(u64::from(attempt) * 2));
        result = support::hoppy_live_json(&args);
        attempt += 1;
    }
    result
}

#[cfg(feature = "live-api")]
#[test]
fn live_stream_collection_lifecycle() {
    support::run_lifecycle(|cleanup| {
        let lib_name = support::unique_name("hoppy-test-lib");

        // 1. Create library
        let lib_create =
            support::hoppy_live_json(&["stream", "library", "create", "--name", &lib_name]);
        assert!(
            lib_create.success,
            "library create failed — stderr: {}",
            lib_create.stderr
        );
        let lib_id = lib_create.json.as_ref().unwrap()["Id"].as_i64().unwrap();
        let lib_id_str = lib_id.to_string();

        // Push library delete first — it runs last (stack is LIFO)
        cleanup.push(&["stream", "library", "delete", "--id", &lib_id_str]);

        let col_name = support::unique_name("hoppy-test-col");

        // 2. Create collection (retry on 401 — see create_collection_with_retry)
        let col_create = create_collection_with_retry(&lib_id_str, &col_name);
        assert!(
            col_create.success,
            "collection create failed — stderr: {}",
            col_create.stderr
        );
        let guid = col_create.json.as_ref().unwrap()["Guid"]
            .as_str()
            .unwrap()
            .to_string();

        // Push collection delete second — it runs first (before library delete)
        cleanup.push(&[
            "stream",
            "collection",
            "delete",
            "--library-id",
            &lib_id_str,
            "--collection-id",
            &guid,
        ]);

        // 3. Get collection
        let get = support::hoppy_live_json(&[
            "stream",
            "collection",
            "get",
            "--library-id",
            &lib_id_str,
            "--collection-id",
            &guid,
        ]);
        assert!(
            get.success,
            "collection get failed — stderr: {}",
            get.stderr
        );
        assert_eq!(
            get.json.as_ref().unwrap()["Guid"].as_str(),
            Some(guid.as_str()),
            "get returned wrong guid"
        );

        // 4. List collections and verify appears
        let list = support::hoppy_live_json(&[
            "stream",
            "collection",
            "list",
            "--library-id",
            &lib_id_str,
        ]);
        assert!(list.success, "list failed — stderr: {}", list.stderr);
        let found = list.json.as_ref().unwrap()["Items"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .any(|c| c["Guid"].as_str() == Some(guid.as_str()))
            })
            .unwrap_or(false);
        assert!(found, "collection {guid} not found in list output");

        // 5. Update collection name
        let updated_col_name = format!("{col_name}-updated");
        let update = support::hoppy_live_json(&[
            "stream",
            "collection",
            "update",
            "--library-id",
            &lib_id_str,
            "--collection-id",
            &guid,
            "--name",
            &updated_col_name,
        ]);
        assert!(
            update.success,
            "collection update failed — stderr: {}",
            update.stderr
        );

        // 6. Get and verify name changed
        let get2 = support::hoppy_live_json(&[
            "stream",
            "collection",
            "get",
            "--library-id",
            &lib_id_str,
            "--collection-id",
            &guid,
        ]);
        assert!(
            get2.success,
            "second collection get failed — stderr: {}",
            get2.stderr
        );
        assert_eq!(
            get2.json.as_ref().unwrap()["Name"].as_str(),
            Some(updated_col_name.as_str()),
            "collection name was not updated"
        );

        // 7 & 8. Cleanup runs via CleanupStack on exit
        //        (collection delete first, then library delete)
    });
}

#[cfg(feature = "live-api")]
#[test]
fn live_stream_video_processing_lifecycle() {
    // Requires: BUNNY_API_KEY env var and a pre-existing small video file.
    // If TEST_VIDEO_PATH is not set, we skip gracefully.
    let video_path = match std::env::var("TEST_VIDEO_PATH") {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Skipping live_stream_video_processing_lifecycle: TEST_VIDEO_PATH not set");
            return;
        }
    };

    support::run_lifecycle(|cleanup| {
        let lib_name = support::unique_name("hoppy-test-proc");

        // 1. Create library
        let create =
            support::hoppy_live_json(&["stream", "library", "create", "--name", &lib_name]);
        assert!(create.success, "create failed: {}", create.stderr);
        let lib_id = create.json.as_ref().unwrap()["Id"].as_i64().unwrap();
        let lib_id_str = lib_id.to_string();
        cleanup.push(&["stream", "library", "delete", "--id", &lib_id_str]);

        // 2. Upload video
        let upload = support::hoppy_live_json(&[
            "stream",
            "video",
            "upload",
            "--library-id",
            &lib_id_str,
            "--file",
            &video_path,
            "--title",
            "processing-test",
        ]);
        assert!(upload.success, "upload failed: {}", upload.stderr);
        let video_id = upload.json.as_ref().unwrap()["Guid"]
            .as_str()
            .unwrap()
            .to_owned();
        cleanup.push(&[
            "stream",
            "video",
            "delete",
            "--library-id",
            &lib_id_str,
            "--video-id",
            &video_id,
        ]);

        // 3. Poll until Finished (status=4) — give up after ~3 minutes
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
        loop {
            let get = support::hoppy_live_json(&[
                "stream",
                "video",
                "get",
                "--library-id",
                &lib_id_str,
                "--video-id",
                &video_id,
            ]);
            assert!(get.success, "get failed: {}", get.stderr);
            let status = get.json.as_ref().unwrap()["Status"].as_i64().unwrap_or(-1);
            if status == 4 {
                break;
            }
            if std::time::Instant::now() > deadline {
                panic!(
                    "Timed out waiting for video to reach Finished status (last status={status})"
                );
            }
            std::thread::sleep(std::time::Duration::from_secs(10));
        }

        // 4. Heatmap
        let heatmap = support::hoppy_live_json(&[
            "stream",
            "video",
            "heatmap",
            "--library-id",
            &lib_id_str,
            "--video-id",
            &video_id,
        ]);
        assert!(heatmap.success, "heatmap failed: {}", heatmap.stderr);

        // 5. Resolutions list
        let res = support::hoppy_live_json(&[
            "stream",
            "video",
            "resolutions",
            "list",
            "--library-id",
            &lib_id_str,
            "--video-id",
            &video_id,
        ]);
        assert!(res.success, "resolutions list failed: {}", res.stderr);

        // 6. Storage
        let storage = support::hoppy_live_json(&[
            "stream",
            "video",
            "storage",
            "--library-id",
            &lib_id_str,
            "--video-id",
            &video_id,
        ]);
        assert!(storage.success, "storage failed: {}", storage.stderr);

        // 7. Reencode
        let reencode = support::hoppy_live_json(&[
            "stream",
            "video",
            "reencode",
            "--library-id",
            &lib_id_str,
            "--video-id",
            &video_id,
        ]);
        assert!(reencode.success, "reencode failed: {}", reencode.stderr);

        // 8. Transcribe
        let transcribe = support::hoppy_live_json(&[
            "stream",
            "video",
            "transcribe",
            "--library-id",
            &lib_id_str,
            "--video-id",
            &video_id,
        ]);
        assert!(
            transcribe.success,
            "transcribe failed: {}",
            transcribe.stderr
        );
    });
}
