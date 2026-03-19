mod e2e_support;

use e2e_support::{cmd, server, skip_in_live_mode};
use predicates::prelude::*;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

const FIXTURE_LIBRARY_LIST: &str = include_str!("../crates/bunny-api-core/tests/fixtures/videolibrary_list_paginated.json");
const FIXTURE_LIBRARY_GET: &str = include_str!("../crates/bunny-api-core/tests/fixtures/videolibrary_get.json");
const FIXTURE_VIDEO_LIST: &str = include_str!("../crates/bunny-api-stream/tests/fixtures/video_list_paginated.json");
const FIXTURE_VIDEO_GET: &str = include_str!("../crates/bunny-api-stream/tests/fixtures/video_get.json");
const FIXTURE_COLLECTION_LIST: &str =
    include_str!("../crates/bunny-api-stream/tests/fixtures/collection_list_paginated.json");
const FIXTURE_COLLECTION_GET: &str = include_str!("../crates/bunny-api-stream/tests/fixtures/collection_get.json");

// ---------------------------------------------------------------------------
// Library — List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stream_library_list_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/videolibrary"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIBRARY_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["stream", "library", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("main-library"))
        .stdout(predicate::str::contains("archive-library"));
}

#[tokio::test]
async fn stream_library_list_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/videolibrary"))
        .and(header("AccessKey", "test-api-key"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1000"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIBRARY_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["--format", "json", "stream", "library", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"CurrentPage\""))
        .stdout(predicate::str::contains("\"TotalItems\""));
}

// ---------------------------------------------------------------------------
// Library — Get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stream_library_get_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/videolibrary/10001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIBRARY_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["stream", "library", "get", "--id", "10001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("main-library"))
        .stdout(predicate::str::contains("5001"));
}

#[tokio::test]
async fn stream_library_get_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/videolibrary/10001"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIBRARY_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format", "json", "stream", "library", "get", "--id", "10001",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"Id\": 10001"));
}

#[tokio::test]
async fn stream_library_get_not_found() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/videolibrary/99999"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(404).set_body_raw(
            include_str!("../crates/bunny-api-core/tests/fixtures/error_not_found_videolibrary.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["stream", "library", "get", "--id", "99999"])
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// Video — List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stream_video_list_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/library/10001/videos"))
        .and(header("AccessKey", "test-stream-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["stream", "video", "list", "--library-id", "10001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Introduction Video"))
        .stdout(predicate::str::contains("Tutorial Part 1"));
}

#[tokio::test]
async fn stream_video_list_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/library/10001/videos"))
        .and(header("AccessKey", "test-stream-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format",
            "json",
            "stream",
            "video",
            "list",
            "--library-id",
            "10001",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"CurrentPage\""))
        .stdout(predicate::str::contains("\"TotalItems\""));
}

// ---------------------------------------------------------------------------
// Video — Get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stream_video_get_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee",
        ))
        .and(header("AccessKey", "test-stream-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "stream",
            "video",
            "get",
            "--library-id",
            "10001",
            "--video-id",
            "aaaabbbb-1111-2222-3333-ccccddddeeee",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Introduction Video"))
        .stdout(predicate::str::contains(
            "aaaabbbb-1111-2222-3333-ccccddddeeee",
        ));
}

#[tokio::test]
async fn stream_video_get_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/library/10001/videos/aaaabbbb-1111-2222-3333-ccccddddeeee",
        ))
        .and(header("AccessKey", "test-stream-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_VIDEO_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format",
            "json",
            "stream",
            "video",
            "get",
            "--library-id",
            "10001",
            "--video-id",
            "aaaabbbb-1111-2222-3333-ccccddddeeee",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"aaaabbbb-1111-2222-3333-ccccddddeeee\"",
        ));
}

#[tokio::test]
async fn stream_video_get_not_found() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/library/10001/videos/no-such-guid"))
        .and(header("AccessKey", "test-stream-key"))
        .respond_with(ResponseTemplate::new(404).set_body_raw(
            include_str!("../crates/bunny-api-stream/tests/fixtures/error_not_found_video.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "stream",
            "video",
            "get",
            "--library-id",
            "10001",
            "--video-id",
            "no-such-guid",
        ])
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// Collection — List
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stream_collection_list_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/library/10001/collections"))
        .and(header("AccessKey", "test-stream-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_COLLECTION_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["stream", "collection", "list", "--library-id", "10001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Tutorials"))
        .stdout(predicate::str::contains("Demos"));
}

#[tokio::test]
async fn stream_collection_list_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/library/10001/collections"))
        .and(header("AccessKey", "test-stream-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_COLLECTION_LIST, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format",
            "json",
            "stream",
            "collection",
            "list",
            "--library-id",
            "10001",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"CurrentPage\""))
        .stdout(predicate::str::contains("\"TotalItems\""));
}

// ---------------------------------------------------------------------------
// Collection — Get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stream_collection_get_table_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/library/10001/collections/col-guid-0001"))
        .and(header("AccessKey", "test-stream-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_COLLECTION_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "stream",
            "collection",
            "get",
            "--library-id",
            "10001",
            "--collection-id",
            "col-guid-0001",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Tutorials"))
        .stdout(predicate::str::contains("col-guid-0001"));
}

#[tokio::test]
async fn stream_collection_get_json_output() {
    skip_in_live_mode!();
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/library/10001/collections/col-guid-0001"))
        .and(header("AccessKey", "test-stream-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_COLLECTION_GET, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
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
        .assert()
        .success()
        .stdout(predicate::str::contains("\"col-guid-0001\""));
}
