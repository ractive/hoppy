use bunny_net_api::stream::{CreateVideo, StreamCleanupResolutions, StreamClient, TusUploader};
use wiremock::matchers::{method, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_VIDEO_RESOLUTIONS_CLEANUP_STATUS: &str =
    include_str!("../../../../../fixtures/stream/video_resolutions_cleanup_status.json");

#[tokio::test]
async fn create_video_is_blocked_under_dry_run() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let client = StreamClient::new("stream-test-key")
        .with_base_url(server.uri())
        .with_dry_run(true);

    let err = client
        .create_video(10001, &CreateVideo::new("my title"))
        .await
        .expect_err("mutating call must be blocked under dry-run");

    let skipped = err
        .chain()
        .find_map(|e| e.downcast_ref::<bunny_net_api::dry_run::DryRunSkipped>())
        .expect("error chain must contain DryRunSkipped");
    assert_eq!(skipped.method, "POST");
}

/// `stream video resolutions cleanup --dry-run` is the one documented
/// exception: the global `--dry-run` flag maps into
/// `StreamCleanupResolutions::dry_run`, which drives a real request carrying
/// `?dryRun=true` so the server can return its own preview. That request
/// must still be sent even when the client itself is in `--dry-run` mode.
#[tokio::test]
async fn cleanup_resolutions_exemption_still_sends_under_client_dry_run() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(query_param("dryRun", "true"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(FIXTURE_VIDEO_RESOLUTIONS_CLEANUP_STATUS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = StreamClient::new("stream-test-key")
        .with_base_url(server.uri())
        .with_dry_run(true);

    let opts = StreamCleanupResolutions {
        dry_run: true,
        ..Default::default()
    };
    let result = client
        .cleanup_video_resolutions(10001, "aaaabbbb-1111-2222-3333-ccccddddeeee", &opts)
        .await
        .expect("cleanup_video_resolutions must bypass the client-level dry-run block");

    assert!(result.success);
}

#[tokio::test]
async fn tus_create_is_blocked_under_dry_run() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(201))
        .expect(0)
        .mount(&server)
        .await;

    let uploader = TusUploader::new(10001, "stream-test-key", "video-id")
        .with_base_url(server.uri())
        .with_dry_run(true);

    let err = uploader
        .create(
            1024,
            "my title",
            &bunny_net_api::stream::types::VideoUploadOptions::default(),
        )
        .await
        .expect_err("TUS create must be blocked under dry-run");

    let skipped = err
        .chain()
        .find_map(|e| e.downcast_ref::<bunny_net_api::dry_run::DryRunSkipped>())
        .expect("error chain must contain DryRunSkipped");
    assert_eq!(skipped.method, "POST");
}
