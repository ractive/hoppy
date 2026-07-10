use super::support;

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
        "--remote-path",
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
        "--remote-path",
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
async fn storage_upload_computes_checksum_locally() {
    // Bare `--checksum` (no value) must compute the SHA-256 locally and send it
    // uppercased in the `Checksum` header.
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/my-zone/test-dir/hello.txt"))
        .and(header("AccessKey", "mock-storage-key"))
        .and(header(
            "Checksum",
            "83DC13CAF98C33CBE4A90BDDCDE1BF519B6F26267C74E1298FD41688B16901EB",
        ))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
            support::fixture("storage/storage_upload_success.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let temp_path = std::env::temp_dir().join("hoppy-test-upload-checksum.txt");
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
        "--checksum",
    ])
    .output()
    .unwrap();

    let _ = std::fs::remove_file(&temp_path);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn storage_upload_accepts_supplied_checksum_uppercased() {
    // `--checksum <lowercase hex>` must be uppercased before being sent.
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/my-zone/test-dir/hello.txt"))
        .and(header("AccessKey", "mock-storage-key"))
        .and(header(
            "Checksum",
            "83DC13CAF98C33CBE4A90BDDCDE1BF519B6F26267C74E1298FD41688B16901EB",
        ))
        .respond_with(ResponseTemplate::new(201).set_body_raw(
            support::fixture("storage/storage_upload_success.json"),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let temp_path = std::env::temp_dir().join("hoppy-test-upload-supplied-checksum.txt");
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
        "--checksum",
        "83dc13caf98c33cbe4a90bddcde1bf519b6f26267c74e1298fd41688b16901eb",
    ])
    .output()
    .unwrap();

    let _ = std::fs::remove_file(&temp_path);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn storage_upload_rejects_invalid_checksum() {
    let temp_path = std::env::temp_dir().join("hoppy-test-upload-bad-checksum.txt");
    std::fs::write(&temp_path, b"hello world content").unwrap();

    // No server needed — validation fails before any request.
    let output = support::hoppy_mock_cmd_full(
        "test-api-key",
        "http://127.0.0.1:1",
        Some("http://127.0.0.1:1"),
        None,
        None,
    )
    .args([
        "storage",
        "upload",
        "--zone",
        "my-zone",
        "--remote-path",
        "/test-dir/hello.txt",
        "--file",
        temp_path.to_str().unwrap(),
        "--checksum",
        "not-a-valid-sha256",
    ])
    .output()
    .unwrap();

    let _ = std::fs::remove_file(&temp_path);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid --checksum"),
        "expected a checksum validation error, got: {stderr}"
    );
}

#[tokio::test]
async fn storage_rm_directory_recursive() {
    // A trailing slash targets the directory listing URL for a recursive delete.
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/my-zone/test-dir/"))
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
        "test-dir/",
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
        stdout.contains("storage-directory"),
        "expected a directory delete result, got: {stdout}"
    );
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
        "--file",
        temp_path.to_str().unwrap(),
    ])
    .output()
    .unwrap();

    assert!(output.status.success());
    let contents = std::fs::read(&temp_path).unwrap();
    let _ = std::fs::remove_file(&temp_path);
    assert_eq!(contents, b"hello world content");
}

#[cfg(feature = "live-api")]
#[test]
fn live_storage_file_ops() {
    support::run_lifecycle(|cleanup| {
        let raw_name = support::unique_name("hpst");
        let zone_name: String = raw_name.chars().take(20).collect();

        // 1. Create storage zone
        let create = support::hoppy_live_json(&[
            "storage-zone",
            "create",
            "--name",
            &zone_name,
            "--region",
            "DE",
        ]);
        assert!(create.success, "zone create failed: {}", create.stderr);
        let id = create.json.as_ref().unwrap()["Id"]
            .as_u64()
            .expect("Id missing from create response");
        let id_str = id.to_string();

        // Register zone cleanup early
        cleanup.push(&["storage-zone", "delete", "--id", &id_str]);

        // Wait for zone to propagate
        std::thread::sleep(std::time::Duration::from_secs(5));

        // 2. Create a temp file with known content
        let content = b"hoppy live test content";
        let upload_path = format!("/tmp/hoppy-test-{}.txt", zone_name);
        std::fs::write(&upload_path, content).expect("failed to write temp file");

        // 3. Upload
        let upload = support::hoppy_live_json(&[
            "storage",
            "upload",
            "--zone",
            &zone_name,
            "--remote-path",
            "test/hello.txt",
            "--file",
            &upload_path,
        ]);
        let _ = std::fs::remove_file(&upload_path);
        assert!(upload.success, "upload failed: {}", upload.stderr);

        // 4. List — verify hello.txt appears
        let list = support::hoppy_live_json(&[
            "storage",
            "ls",
            "--zone",
            &zone_name,
            "--remote-path",
            "test",
        ]);
        assert!(list.success, "ls failed: {}", list.stderr);
        let found = list.stdout.contains("hello.txt");
        assert!(found, "hello.txt not found in ls output");

        // 5. Download
        let download_path = format!("/tmp/hoppy-test-dl-{}.txt", zone_name);
        let download = support::hoppy_live_json(&[
            "storage",
            "download",
            "--zone",
            &zone_name,
            "--remote-path",
            "test/hello.txt",
            "--file",
            &download_path,
        ]);
        assert!(download.success, "download failed: {}", download.stderr);

        // 6. Verify downloaded content
        let downloaded = std::fs::read(&download_path).expect("failed to read downloaded file");
        let _ = std::fs::remove_file(&download_path);
        assert_eq!(downloaded, content, "downloaded content does not match");

        // 7. Remove file
        let rm = support::hoppy_live_json_yes(&[
            "storage",
            "rm",
            "--zone",
            &zone_name,
            "--remote-path",
            "test/hello.txt",
        ]);
        assert!(rm.success, "rm failed: {}", rm.stderr);

        // 8. Storage zone delete is handled by cleanup
    });
}
