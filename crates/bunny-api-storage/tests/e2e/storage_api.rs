use bunny_api_storage::StorageClient;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_LIST_FILES: &str =
    include_str!("../../../../fixtures/storage/storage_list_files.json");
const FIXTURE_UPLOAD_SUCCESS: &str =
    include_str!("../../../../fixtures/storage/storage_upload_success.json");
const FIXTURE_DELETE_SUCCESS: &str =
    include_str!("../../../../fixtures/storage/storage_delete_success.json");
const FIXTURE_UNAUTHORIZED: &str =
    include_str!("../../../../fixtures/storage/storage_unauthorized.json");
const FIXTURE_NOT_FOUND: &str = include_str!("../../../../fixtures/storage/storage_not_found.json");

fn test_client(uri: &str) -> StorageClient {
    StorageClient::with_base_url("test-access-key", uri)
}

#[tokio::test]
async fn list_files_returns_objects() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/hoppy-test-zone/test-dir/"))
        .and(header("AccessKey", "test-access-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_FILES, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let objects = test_client(&server.uri())
        .list_files("hoppy-test-zone", "test-dir")
        .await
        .unwrap();

    assert_eq!(objects.len(), 1);
    let obj = &objects[0];
    assert_eq!(obj.object_name, "hello.txt");
    assert_eq!(obj.length, 23);
    assert!(!obj.is_directory);
    assert_eq!(
        obj.checksum.as_deref(),
        Some("1E2EDD988B5BB04D20B846D94BB2C0ABDBA6A573C4EB10BB7754D8ADFEFC9138")
    );
}

#[tokio::test]
async fn list_files_empty_path_lists_root() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/hoppy-test-zone/"))
        .and(header("AccessKey", "test-access-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_FILES, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let objects = test_client(&server.uri())
        .list_files("hoppy-test-zone", "")
        .await
        .unwrap();

    assert_eq!(objects.len(), 1);
}

#[tokio::test]
async fn upload_file_success() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/hoppy-test-zone/test-dir/hello.txt"))
        .and(header("AccessKey", "test-access-key"))
        .respond_with(
            ResponseTemplate::new(201).set_body_raw(FIXTURE_UPLOAD_SUCCESS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .upload_file(
            "hoppy-test-zone",
            "test-dir",
            "hello.txt",
            b"hello world".to_vec(),
            None,
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn upload_file_with_checksum() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/hoppy-test-zone/test-dir/hello.txt"))
        .and(header("AccessKey", "test-access-key"))
        .and(header(
            "Checksum",
            "1E2EDD988B5BB04D20B846D94BB2C0ABDBA6A573C4EB10BB7754D8ADFEFC9138",
        ))
        .respond_with(
            ResponseTemplate::new(201).set_body_raw(FIXTURE_UPLOAD_SUCCESS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .upload_file(
            "hoppy-test-zone",
            "test-dir",
            "hello.txt",
            b"hello world".to_vec(),
            Some("1E2EDD988B5BB04D20B846D94BB2C0ABDBA6A573C4EB10BB7754D8ADFEFC9138"),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn download_file_returns_bytes() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/hoppy-test-zone/test-dir/hello.txt"))
        .and(header("AccessKey", "test-access-key"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"hello world".to_vec()))
        .expect(1)
        .mount(&server)
        .await;

    let bytes = test_client(&server.uri())
        .download_file("hoppy-test-zone", "test-dir", "hello.txt")
        .await
        .unwrap();

    assert_eq!(bytes.as_ref(), b"hello world");
}

#[tokio::test]
async fn delete_file_success() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/hoppy-test-zone/test-dir/hello.txt"))
        .and(header("AccessKey", "test-access-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_DELETE_SUCCESS, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    test_client(&server.uri())
        .delete_file("hoppy-test-zone", "test-dir", "hello.txt")
        .await
        .unwrap();
}

#[tokio::test]
async fn unauthorized_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/hoppy-test-zone/"))
        .respond_with(
            ResponseTemplate::new(401).set_body_raw(FIXTURE_UNAUTHORIZED, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .list_files("hoppy-test-zone", "")
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("Unauthorized") || err.to_string().contains("401"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn not_found_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/hoppy-test-zone/missing/"))
        .respond_with(
            ResponseTemplate::new(404).set_body_raw(FIXTURE_NOT_FOUND, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = test_client(&server.uri())
        .list_files("hoppy-test-zone", "missing")
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("Not Found") || err.to_string().contains("404"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn debug_mode_does_not_panic() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/hoppy-test-zone/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_FILES, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    // Just verify debug mode builds and runs without error.
    let client = StorageClient::with_base_url("test-access-key", server.uri()).with_debug(true);
    let objects = client.list_files("hoppy-test-zone", "").await.unwrap();
    assert_eq!(objects.len(), 1);
}
