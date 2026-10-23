//! `<ID>` may be given positionally or as `--id <ID>` on single-resource
//! commands. One representative command per command group, both forms.

use super::support;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Mount a GET mock for `url_path` serving `fixture_name` (any number of hits).
async fn serve(server: &MockServer, url_path: &str, fixture_name: &str) {
    Mock::given(method("GET"))
        .and(path(url_path))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(support::fixture(fixture_name), "application/json"),
        )
        .mount(server)
        .await;
}

/// Run `hoppy --format json <words…> <id>` and `… --id <id>`; both must
/// succeed with identical stdout.
async fn assert_both_forms(server: &MockServer, words: &[&str], id: &str) {
    let run = |tail: &[&str]| {
        let mut args = vec!["--format", "json"];
        args.extend_from_slice(words);
        args.extend_from_slice(tail);
        support::hoppy_mock_cmd("test-api-key", &server.uri())
            .args(args)
            .output()
            .unwrap()
    };
    let positional = run(&[id]);
    let flag = run(&["--id", id]);
    assert!(
        positional.status.success(),
        "positional form failed: {}",
        String::from_utf8_lossy(&positional.stderr)
    );
    assert!(
        flag.status.success(),
        "flag form failed: {}",
        String::from_utf8_lossy(&flag.stderr)
    );
    assert_eq!(positional.stdout, flag.stdout);
    assert!(!positional.stdout.is_empty());
}

#[tokio::test]
async fn pull_zone_get_accepts_positional_and_flag() {
    let server = MockServer::start().await;
    serve(&server, "/pullzone/1001", "core/pullzone_get.json").await;
    assert_both_forms(&server, &["pull-zone", "get"], "1001").await;
}

#[tokio::test]
async fn storage_zone_get_accepts_positional_and_flag() {
    let server = MockServer::start().await;
    serve(&server, "/storagezone/9001", "core/storagezone_get.json").await;
    assert_both_forms(&server, &["storage-zone", "get"], "9001").await;
}

#[tokio::test]
async fn dns_zone_get_accepts_positional_and_flag() {
    let server = MockServer::start().await;
    serve(&server, "/dnszone/50001", "core/dnszone_get.json").await;
    assert_both_forms(&server, &["dns", "zone", "get"], "50001").await;
}

#[tokio::test]
async fn script_get_accepts_positional_and_flag() {
    let server = MockServer::start().await;
    serve(&server, "/compute/script/1001", "compute/script_get.json").await;
    assert_both_forms(&server, &["script", "get"], "1001").await;
}

#[tokio::test]
async fn container_app_get_accepts_positional_and_flag() {
    let server = MockServer::start().await;
    serve(&server, "/apps/l3faCv1fWRHAYNU", "containers/app_get.json").await;
    for words in [&["container", "get"][..], &["container", "app", "get"][..]] {
        let run = |tail: &[&str]| {
            let mut args = vec!["--format", "json"];
            args.extend_from_slice(words);
            args.extend_from_slice(tail);
            support::hoppy_mock_cmd_full(
                "test-api-key",
                &server.uri(),
                None,
                None,
                Some(&server.uri()),
            )
            .args(args)
            .output()
            .unwrap()
        };
        let positional = run(&["l3faCv1fWRHAYNU"]);
        let flag = run(&["--id", "l3faCv1fWRHAYNU"]);
        assert!(positional.status.success(), "{words:?}");
        assert_eq!(positional.stdout, flag.stdout);
    }
}

/// Non-`get` commands share the pattern: dry-run `delete` accepts both forms.
#[tokio::test]
async fn pull_zone_delete_accepts_positional_and_flag() {
    let server = MockServer::start().await;
    for tail in [&["1001"][..], &["--id", "1001"][..]] {
        let mut args = vec!["--dry-run", "pull-zone", "delete"];
        args.extend_from_slice(tail);
        let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{tail:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("/pullzone/1001"),
            "{tail:?}"
        );
    }
}

#[tokio::test]
async fn id_given_twice_is_a_usage_error() {
    let server = MockServer::start().await;
    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["pull-zone", "get", "1001", "--id", "1001"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be used with"));
}

#[tokio::test]
async fn missing_id_is_a_usage_error() {
    let server = MockServer::start().await;
    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["pull-zone", "get"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("<ID>"));
}
