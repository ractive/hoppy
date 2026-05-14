use super::support;

use wiremock::matchers::{header, method, path};
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
    let data_row_re = regex::Regex::new(r"^\|\s*\S").unwrap();
    let data_rows = stdout.lines().filter(|l| data_row_re.is_match(l)).count();
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

        // 2. Create collection
        let col_create = support::hoppy_live_json(&[
            "stream",
            "collection",
            "create",
            "--library-id",
            &lib_id_str,
            "--name",
            &col_name,
        ]);
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
