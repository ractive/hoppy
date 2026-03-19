mod e2e_support;

use e2e_support::{cmd, server};
use predicates::prelude::*;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, ResponseTemplate};

const FIXTURE_LIST_FILES: &str = include_str!("fixtures/storage/storage_list_files.json");
const FIXTURE_DELETE_SUCCESS: &str = include_str!("fixtures/storage/storage_delete_success.json");

// ---------------------------------------------------------------------------
// ls
// ---------------------------------------------------------------------------

#[tokio::test]
async fn storage_ls_root_table_output() {
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/hoppy-test-zone/"))
        .and(header("AccessKey", "test-storage-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_FILES, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args(["storage", "ls", "--zone", "hoppy-test-zone"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello.txt"));
}

#[tokio::test]
async fn storage_ls_json_output() {
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/hoppy-test-zone/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_FILES, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--format",
            "json",
            "storage",
            "ls",
            "--zone",
            "hoppy-test-zone",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ObjectName\""))
        .stdout(predicate::str::contains("hello.txt"));
}

#[tokio::test]
async fn storage_ls_subdir() {
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/hoppy-test-zone/test-dir/"))
        .and(header("AccessKey", "test-storage-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_LIST_FILES, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "storage",
            "ls",
            "--zone",
            "hoppy-test-zone",
            "--path",
            "test-dir",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello.txt"));
}

// ---------------------------------------------------------------------------
// rm
// ---------------------------------------------------------------------------

#[tokio::test]
async fn storage_rm_with_yes_flag() {
    let mock = server::start().await;

    Mock::given(method("DELETE"))
        .and(path("/hoppy-test-zone/test-dir/hello.txt"))
        .and(header("AccessKey", "test-storage-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(FIXTURE_DELETE_SUCCESS, "application/json"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "--yes",
            "storage",
            "rm",
            "--zone",
            "hoppy-test-zone",
            "--remote-path",
            "test-dir/hello.txt",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Deleted hoppy-test-zone/test-dir/hello.txt",
        ));
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn storage_ls_not_found() {
    let mock = server::start().await;

    Mock::given(method("GET"))
        .and(path("/hoppy-test-zone/missing/"))
        .respond_with(
            ResponseTemplate::new(404).set_body_string("{\"Message\":\"Object Not Found\"}"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    cmd::hoppy(&mock)
        .args([
            "storage",
            "ls",
            "--zone",
            "hoppy-test-zone",
            "--path",
            "missing",
        ])
        .assert()
        .failure();
}
