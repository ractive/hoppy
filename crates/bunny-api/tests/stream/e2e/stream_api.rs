use bunny_api::stream::types::{CreateCollection, UpdateCollection};
use bunny_api::stream::{
    CreateVideo, SmartGenerateSettings, StreamCleanupResolutions, StreamClient, TranscribeSettings,
};
use wiremock::matchers::{body_bytes, body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_VIDEO_LIST: &str =
    include_str!("../../../../../fixtures/stream/video_list_paginated.json");
const FIXTURE_VIDEO_GET: &str = include_str!("../../../../../fixtures/stream/video_get.json");
const FIXTURE_VIDEO_CREATE: &str = include_str!("../../../../../fixtures/stream/video_create.json");
const FIXTURE_VIDEO_UPLOAD_STATUS: &str =
    include_str!("../../../../../fixtures/stream/video_upload_status.json");
const FIXTURE_VIDEO_DELETE_STATUS: &str =
    include_str!("../../../../../fixtures/stream/video_delete_status.json");
const FIXTURE_COLLECTION_LIST: &str =
    include_str!("../../../../../fixtures/stream/collection_list_paginated.json");
const FIXTURE_COLLECTION_GET: &str =
    include_str!("../../../../../fixtures/stream/collection_get.json");
const FIXTURE_COLLECTION_CREATE: &str =
    include_str!("../../../../../fixtures/stream/collection_create.json");
const FIXTURE_NOT_FOUND: &str =
    include_str!("../../../../../fixtures/stream/error_not_found_video.json");
const FIXTURE_CAPTION_ADD: &str =
    include_str!("../../../../../fixtures/stream/video_caption_add.json");
const FIXTURE_CAPTION_DELETE: &str =
    include_str!("../../../../../fixtures/stream/video_caption_delete.json");
const FIXTURE_LIBRARY_STATS: &str =
    include_str!("../../../../../fixtures/stream/library_statistics.json");
const FIXTURE_VIDEO_TRANSCRIBE_STATUS: &str =
    include_str!("../../../../../fixtures/stream/video_transcribe_status.json");
const FIXTURE_VIDEO_HEATMAP: &str =
    include_str!("../../../../../fixtures/stream/video_heatmap.json");
const FIXTURE_VIDEO_REENCODE: &str =
    include_str!("../../../../../fixtures/stream/video_reencode.json");
const FIXTURE_VIDEO_REPACKAGE: &str =
    include_str!("../../../../../fixtures/stream/video_repackage.json");
const FIXTURE_VIDEO_SMART_STATUS: &str =
    include_str!("../../../../../fixtures/stream/video_smart_status.json");
const FIXTURE_VIDEO_THUMBNAIL_STATUS: &str =
    include_str!("../../../../../fixtures/stream/video_thumbnail_status.json");
const FIXTURE_VIDEO_RESOLUTIONS: &str =
    include_str!("../../../../../fixtures/stream/video_resolutions.json");
const FIXTURE_VIDEO_STORAGE: &str =
    include_str!("../../../../../fixtures/stream/video_storage.json");
const FIXTURE_VIDEO_RESOLUTIONS_CLEANUP_STATUS: &str =
    include_str!("../../../../../fixtures/stream/video_resolutions_cleanup_status.json");

fn test_client(uri: &str) -> StreamClient {
    StreamClient::new("stream-test-key").with_base_url(uri)
}

// ---------------------------------------------------------------------------
// Video — list
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_videos_returns_paginated_items() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/library/10001/videos"))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_videos(10001, None, None, None, None, None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
    assert_eq!(result.total_items, 2);
    assert_eq!(result.current_page, 1);

    let first = &result.items[0];
    assert_eq!(first.guid, "aaaabbbb-1111-2222-3333-ccccddddeeee");
    assert_eq!(first.title, "Introduction Video");
    assert_eq!(first.video_library_id, 10001);
}

#[tokio::test]
async fn list_videos_forwards_pagination_params() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/library/10001/videos"))
        .and(query_param("page", "2"))
        .and(query_param("itemsPerPage", "25"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_videos(10001, Some(2), Some(25), None, None, None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
}

#[tokio::test]
async fn list_videos_with_search() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/library/10001/videos"))
        .and(query_param("search", "Tutorial"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_videos(10001, None, None, Some("Tutorial"), None, None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
}

#[tokio::test]
async fn list_videos_with_collection_filter() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/library/10001/videos"))
        .and(query_param("collection", "col-guid-0001"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_videos(10001, None, None, None, Some("col-guid-0001"), None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
}

// ---------------------------------------------------------------------------
// Video — get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_video_by_id() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee",
        ))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let video = test_client(&server.uri())
        .get_video(10001, "aaaabbbb-1111-2222-3333-ccccddddeeee")
        .await
        .unwrap();

    assert_eq!(video.guid, "aaaabbbb-1111-2222-3333-ccccddddeeee");
    assert_eq!(video.title, "Introduction Video");
    assert_eq!(video.width, 1920);
    assert_eq!(video.captions.len(), 1);
    assert_eq!(video.captions[0].srclang, "en");
}

// ---------------------------------------------------------------------------
// Video — create
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_video_sends_correct_body() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "Title": "My New Video"
    });

    Mock::given(method("POST"))
        .and(path("/library/10001/videos"))
        .and(header("AccessKey", "stream-test-key"))
        .and(body_json(expected_body))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_CREATE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = CreateVideo::new("My New Video");
    let video = test_client(&server.uri())
        .create_video(10001, &body)
        .await
        .unwrap();

    assert_eq!(video.guid, "newvideo-1111-2222-3333-444455556666");
    assert_eq!(video.title, "My New Video");
}

#[tokio::test]
async fn create_video_with_optional_fields() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "Title": "My New Video",
        "CollectionId": "col-guid-0001",
        "ThumbnailTime": 3000
    });

    Mock::given(method("POST"))
        .and(path("/library/10001/videos"))
        .and(body_json(expected_body))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_CREATE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = CreateVideo::new("My New Video")
        .collection_id("col-guid-0001")
        .thumbnail_time(3000);
    let video = test_client(&server.uri())
        .create_video(10001, &body)
        .await
        .unwrap();

    assert_eq!(video.guid, "newvideo-1111-2222-3333-444455556666");
}

// ---------------------------------------------------------------------------
// Video — upload
// ---------------------------------------------------------------------------

#[tokio::test]
async fn upload_video_sends_binary_body() {
    let server = MockServer::start().await;

    let video_bytes: Vec<u8> = b"fake-video-binary-content".to_vec();

    Mock::given(method("PUT"))
        .and(path(
            "/library/10001/videos/newvideo-1111-2222-3333-444455556666",
        ))
        .and(header("AccessKey", "stream-test-key"))
        .and(header("Content-Type", "application/octet-stream"))
        .and(body_bytes(video_bytes.clone()))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_VIDEO_UPLOAD_STATUS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let status = test_client(&server.uri())
        .upload_video(10001, "newvideo-1111-2222-3333-444455556666", video_bytes)
        .await
        .unwrap();

    assert!(status.success);
    assert_eq!(status.status_code, 200);
}

// ---------------------------------------------------------------------------
// Video — update
// ---------------------------------------------------------------------------

#[tokio::test]
async fn update_video_sends_correct_body() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "Title": "Renamed Video"
    });

    Mock::given(method("POST"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee",
        ))
        .and(header("AccessKey", "stream-test-key"))
        .and(body_json(expected_body))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_VIDEO_UPLOAD_STATUS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    use bunny_api::stream::types::UpdateVideo;
    let body = UpdateVideo::new().title("Renamed Video");
    let status = test_client(&server.uri())
        .update_video(10001, "aaaabbbb-1111-2222-3333-ccccddddeeee", &body)
        .await
        .unwrap();

    assert!(status.success);
}

// ---------------------------------------------------------------------------
// Video — delete
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_video_success() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee",
        ))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_VIDEO_DELETE_STATUS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let status = test_client(&server.uri())
        .delete_video(10001, "aaaabbbb-1111-2222-3333-ccccddddeeee")
        .await
        .unwrap();

    assert!(status.success);
}

// ---------------------------------------------------------------------------
// Video — error handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_nonexistent_video_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/library/10001/videos/no-such-guid"))
        .respond_with(
            ResponseTemplate::new(404).set_body_raw(FIXTURE_NOT_FOUND, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .get_video(10001, "no-such-guid")
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("404"),
        "expected 404 in error: {err}"
    );
}

#[tokio::test]
async fn invalid_api_key_returns_unauthorized_for_videos() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/library/10001/videos"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .list_videos(10001, None, None, None, None, None)
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("401"),
        "expected 401 in error: {err}"
    );
}

// ---------------------------------------------------------------------------
// Debug mode
// ---------------------------------------------------------------------------

#[tokio::test]
async fn debug_mode_works_for_stream_client() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee",
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_GET, "application/json"),
        )
        .mount(&server)
        .await;

    let client = StreamClient::new("stream-test-key")
        .with_base_url(server.uri())
        .with_debug(true);
    let video = client
        .get_video(10001, "aaaabbbb-1111-2222-3333-ccccddddeeee")
        .await
        .unwrap();
    assert_eq!(video.guid, "aaaabbbb-1111-2222-3333-ccccddddeeee");
}

// ---------------------------------------------------------------------------
// Collection — list
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_collections_returns_paginated_items() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/library/10001/collections"))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_COLLECTION_LIST, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .list_collections(10001, None, None, None, None)
        .await
        .unwrap();

    assert_eq!(result.items.len(), 2);
    assert_eq!(result.total_items, 2);

    let first = &result.items[0];
    assert_eq!(first.guid.as_deref(), Some("col-guid-0001"));
    assert_eq!(first.name.as_deref(), Some("Tutorials"));
    assert_eq!(first.video_count, 5);
}

// ---------------------------------------------------------------------------
// Collection — get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_collection_by_id() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/library/10001/collections/col-guid-0001"))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_COLLECTION_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let col = test_client(&server.uri())
        .get_collection(10001, "col-guid-0001")
        .await
        .unwrap();

    assert_eq!(col.guid.as_deref(), Some("col-guid-0001"));
    assert_eq!(col.name.as_deref(), Some("Tutorials"));
    assert_eq!(col.video_count, 5);
}

// ---------------------------------------------------------------------------
// Collection — create
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_collection_sends_correct_body() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "Name": "New Collection"
    });

    Mock::given(method("POST"))
        .and(path("/library/10001/collections"))
        .and(header("AccessKey", "stream-test-key"))
        .and(body_json(expected_body))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_COLLECTION_CREATE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = CreateCollection::new("New Collection");
    let col = test_client(&server.uri())
        .create_collection(10001, &body)
        .await
        .unwrap();

    assert_eq!(col.guid.as_deref(), Some("col-guid-new1"));
    assert_eq!(col.name.as_deref(), Some("New Collection"));
}

// ---------------------------------------------------------------------------
// Collection — update
// ---------------------------------------------------------------------------

#[tokio::test]
async fn update_collection_sends_correct_body() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!({
        "Name": "Renamed Collection"
    });

    Mock::given(method("POST"))
        .and(path("/library/10001/collections/col-guid-0001"))
        .and(header("AccessKey", "stream-test-key"))
        .and(body_json(expected_body))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_COLLECTION_GET, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateCollection::new().name("Renamed Collection");
    let col = test_client(&server.uri())
        .update_collection(10001, "col-guid-0001", &body)
        .await
        .unwrap();

    assert_eq!(col.guid.as_deref(), Some("col-guid-0001"));
}

// ---------------------------------------------------------------------------
// Collection — delete
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_collection_success() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/library/10001/collections/col-guid-0001"))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_VIDEO_DELETE_STATUS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let status = test_client(&server.uri())
        .delete_collection(10001, "col-guid-0001")
        .await
        .unwrap();

    assert!(status.success);
}

// ---------------------------------------------------------------------------
// Captions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn add_caption_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/library/1001/videos/abc-123/captions/en"))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_CAPTION_ADD, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .add_caption(
            1001,
            "abc-123",
            "en",
            "1\n00:00:00,000 --> 00:00:05,000\nHello",
        )
        .await
        .unwrap();
    assert!(result.success);
}

#[tokio::test]
async fn delete_caption_sends_correct_request() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/library/1001/videos/abc-123/captions/en"))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_CAPTION_DELETE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .delete_caption(1001, "abc-123", "en")
        .await
        .unwrap();
    assert!(result.success);
}

#[tokio::test]
async fn get_library_statistics_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/library/12345/statistics"))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIBRARY_STATS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stats = test_client(&server.uri())
        .get_library_statistics(12345, None, None, false, None)
        .await
        .unwrap();

    assert_eq!(stats.engagement_score, 72);
    assert!(stats.views_chart.is_some());
    assert_eq!(stats.views_chart.unwrap().len(), 3);
    assert!(stats.country_view_counts.is_some());
    assert_eq!(stats.country_view_counts.unwrap().len(), 4);
}

// ---------------------------------------------------------------------------
// Video processing tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn transcribe_video_with_settings() {
    let server = MockServer::start().await;

    let settings = TranscribeSettings::new()
        .target_languages(["de", "fr"])
        .generate_title(true);

    Mock::given(method("POST"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/transcribe",
        ))
        .and(query_param("force", "true"))
        .and(header("AccessKey", "stream-test-key"))
        .and(body_json(serde_json::json!({
            "targetLanguages": ["de", "fr"],
            "generateTitle": true,
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_VIDEO_TRANSCRIBE_STATUS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .transcribe_video(
            10001,
            "aaaabbbb-1111-2222-3333-ccccddddeeee",
            true,
            Some(&settings),
        )
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.message.as_deref(), Some("Transcription was queued"));
}

#[tokio::test]
async fn transcribe_video_minimal() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/transcribe",
        ))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_VIDEO_TRANSCRIBE_STATUS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .transcribe_video(10001, "aaaabbbb-1111-2222-3333-ccccddddeeee", false, None)
        .await
        .unwrap();

    assert!(result.success);
}

#[tokio::test]
async fn get_video_heatmap_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/heatmap",
        ))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_HEATMAP, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .get_video_heatmap(10001, "aaaabbbb-1111-2222-3333-ccccddddeeee")
        .await
        .unwrap();

    let map = result.heatmap.expect("heatmap should be present");
    assert_eq!(map.get("0"), Some(&80));
    assert_eq!(map.get("1"), Some(&100));
}

#[tokio::test]
async fn reencode_video_returns_video() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/reencode",
        ))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_REENCODE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let video = test_client(&server.uri())
        .reencode_video(10001, "aaaabbbb-1111-2222-3333-ccccddddeeee")
        .await
        .unwrap();

    assert_eq!(video.guid, "aaaabbbb-1111-2222-3333-ccccddddeeee");
}

#[tokio::test]
async fn reencode_video_using_codec_uses_correct_path() {
    use bunny_api::stream::types::EncoderOutputCodec;

    let server = MockServer::start().await;

    // hevc = codec id 2
    Mock::given(method("PUT"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/outputs/2",
        ))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_REENCODE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let video = test_client(&server.uri())
        .reencode_video_using_codec(
            10001,
            "aaaabbbb-1111-2222-3333-ccccddddeeee",
            EncoderOutputCodec::Hevc,
        )
        .await
        .unwrap();

    assert_eq!(video.guid, "aaaabbbb-1111-2222-3333-ccccddddeeee");
}

#[tokio::test]
async fn repackage_video_with_keep_original_default() {
    let server = MockServer::start().await;

    // Default: keepOriginalFiles=true — param should NOT be sent
    Mock::given(method("POST"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/repackage",
        ))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_REPACKAGE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let video = test_client(&server.uri())
        .repackage_video(10001, "aaaabbbb-1111-2222-3333-ccccddddeeee", true)
        .await
        .unwrap();

    assert_eq!(video.guid, "aaaabbbb-1111-2222-3333-ccccddddeeee");

    // Verify no query params were sent (the default path should be parameterless).
    let received = server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].url.query(), None);
}

#[tokio::test]
async fn repackage_video_discarding_originals() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/repackage",
        ))
        .and(query_param("keepOriginalFiles", "false"))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_REPACKAGE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let video = test_client(&server.uri())
        .repackage_video(10001, "aaaabbbb-1111-2222-3333-ccccddddeeee", false)
        .await
        .unwrap();

    assert_eq!(video.guid, "aaaabbbb-1111-2222-3333-ccccddddeeee");
}

#[tokio::test]
async fn smart_generate_sends_body() {
    let server = MockServer::start().await;

    let settings = SmartGenerateSettings::new()
        .generate_title(true)
        .generate_description(true);

    Mock::given(method("POST"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/smart",
        ))
        .and(header("AccessKey", "stream-test-key"))
        .and(body_json(serde_json::json!({
            "generateTitle": true,
            "generateDescription": true,
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_SMART_STATUS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .smart_generate(10001, "aaaabbbb-1111-2222-3333-ccccddddeeee", &settings)
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.message.as_deref(), Some("Smart action queued"));
}

#[tokio::test]
async fn set_video_thumbnail_with_url() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/thumbnail",
        ))
        .and(query_param("thumbnailUrl", "https://example.com/thumb.jpg"))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_VIDEO_THUMBNAIL_STATUS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = test_client(&server.uri())
        .set_video_thumbnail(
            10001,
            "aaaabbbb-1111-2222-3333-ccccddddeeee",
            Some("https://example.com/thumb.jpg"),
        )
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.message.as_deref(), Some("Thumbnail set"));
}

#[tokio::test]
async fn get_video_resolutions_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/resolutions",
        ))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_RESOLUTIONS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let envelope = test_client(&server.uri())
        .get_video_resolutions(10001, "aaaabbbb-1111-2222-3333-ccccddddeeee")
        .await
        .unwrap();

    assert!(envelope.success);
    let data = envelope.data.expect("data should be present");
    assert_eq!(
        data.available_resolutions,
        vec!["1080p", "720p", "480p", "360p", "240p"]
    );
    assert_eq!(data.configured_resolutions, vec!["720p", "480p"]);
    assert!(data.has_original);
}

#[tokio::test]
async fn cleanup_resolutions_dry_run() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/resolutions/cleanup",
        ))
        .and(query_param("dryRun", "true"))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_VIDEO_RESOLUTIONS_CLEANUP_STATUS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let opts = StreamCleanupResolutions {
        dry_run: true,
        ..Default::default()
    };

    let result = test_client(&server.uri())
        .cleanup_video_resolutions(10001, "aaaabbbb-1111-2222-3333-ccccddddeeee", &opts)
        .await
        .unwrap();

    assert!(result.success);
}

#[tokio::test]
async fn cleanup_resolutions_with_explicit_list() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/resolutions/cleanup",
        ))
        .and(query_param("resolutionsToDelete", "720p,480p"))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_VIDEO_RESOLUTIONS_CLEANUP_STATUS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let opts = StreamCleanupResolutions {
        resolutions_to_delete: Some("720p,480p"),
        ..Default::default()
    };

    let result = test_client(&server.uri())
        .cleanup_video_resolutions(10001, "aaaabbbb-1111-2222-3333-ccccddddeeee", &opts)
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.message.as_deref(), Some("Resolutions cleaned"));
}

#[tokio::test]
async fn get_video_storage_size_returns_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee/storage",
        ))
        .and(header("AccessKey", "stream-test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_STORAGE, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let envelope = test_client(&server.uri())
        .get_video_storage_size(10001, "aaaabbbb-1111-2222-3333-ccccddddeeee")
        .await
        .unwrap();

    assert!(envelope.success);
    let data = envelope.data.expect("data should be present");
    assert_eq!(data.originals, 52428800);
    assert_eq!(data.thumbnails, 4096);
    assert_eq!(data.mp4_fallback, 1024);
    let encoded = data.encoded.expect("encoded should be present");
    let rendition = encoded
        .get("x264-720p")
        .expect("x264-720p should be present");
    assert_eq!(rendition.size, 12345678);
}
