mod support;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn storage_ls_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/my-zone/test-dir/"))
        .and(header("AccessKey", "mock-storage-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("storage/storage_list_files.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        Some(&server.uri()),
        None,
        None,
    )
    .args([
        "--format",
        "json",
        "storage",
        "ls",
        "--zone",
        "my-zone",
        "--path",
        "/test-dir",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn storage_ls_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/my-zone/test-dir/"))
        .and(header("AccessKey", "mock-storage-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("storage/storage_list_files.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        Some(&server.uri()),
        None,
        None,
    )
    .args([
        "--format",
        "table",
        "storage",
        "ls",
        "--zone",
        "my-zone",
        "--path",
        "/test-dir",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[tokio::test]
async fn storage_upload() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/my-zone/test-dir/hello.txt"))
        .and(header("AccessKey", "mock-storage-key"))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
            support::fixture("storage/storage_upload_success.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let temp_path = std::env::temp_dir().join("hoppy-test-upload-hello.txt");
    std::fs::write(&temp_path, b"hello world content").unwrap();

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        Some(&server.uri()),
        None,
        None,
    )
    .args([
        "--format",
        "json",
        "storage",
        "upload",
        "--zone",
        "my-zone",
        "--remote-path",
        "/test-dir/hello.txt",
        "--file",
        temp_path.to_str().unwrap(),
    ])
    .output()
    .unwrap();

    let _ = std::fs::remove_file(&temp_path);
    assert!(output.status.success());
}

#[tokio::test]
async fn storage_rm() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/my-zone/test-dir/hello.txt"))
        .and(header("AccessKey", "mock-storage-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("storage/storage_delete_success.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        Some(&server.uri()),
        None,
        None,
    )
    .args([
        "--yes",
        "--format",
        "json",
        "storage",
        "rm",
        "--zone",
        "my-zone",
        "--remote-path",
        "/test-dir/hello.txt",
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn storage_download() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/my-zone/test-dir/hello.txt"))
        .and(header("AccessKey", "mock-storage-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw("hello world content", "application/octet-stream"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let temp_path = std::env::temp_dir().join("hoppy-test-download-hello.txt");

    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        &server.uri(),
        Some(&server.uri()),
        None,
        None,
    )
    .args([
        "--format",
        "json",
        "storage",
        "download",
        "--zone",
        "my-zone",
        "--remote-path",
        "/test-dir/hello.txt",
        "--output",
        temp_path.to_str().unwrap(),
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    let contents = std::fs::read(&temp_path).unwrap();
    let _ = std::fs::remove_file(&temp_path);
    assert_eq!(contents, b"hello world content");
}
