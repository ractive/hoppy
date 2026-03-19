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
        let found = list
            .json
            .as_ref()
            .unwrap()
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

        // 6. Cleanup runs via CleanupStack on exit (delete with --yes)
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
        let guid = col_create.json.as_ref().unwrap()["guid"]
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
            get.json.as_ref().unwrap()["guid"].as_str(),
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
        let found = list
            .json
            .as_ref()
            .unwrap()
            .as_array()
            .map(|arr| {
                arr.iter()
                    .any(|c| c["guid"].as_str() == Some(guid.as_str()))
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
            get2.json.as_ref().unwrap()["name"].as_str(),
            Some(updated_col_name.as_str()),
            "collection name was not updated"
        );

        // 7 & 8. Cleanup runs via CleanupStack on exit
        //        (collection delete first, then library delete)
    });
}
