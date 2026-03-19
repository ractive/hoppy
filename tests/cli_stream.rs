mod support;

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
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
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
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
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
