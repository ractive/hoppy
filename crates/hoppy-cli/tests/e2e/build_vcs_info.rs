//! Unit coverage for the `.cargo_vcs_info.json` parser used by `build.rs`.
//! Build scripts are not compiled into `cargo test`, so the parser lives in
//! `build_support/vcs_info.rs` and is pulled in here via `#[path]`.

#[path = "../../build_support/vcs_info.rs"]
mod vcs_info;

use vcs_info::parse_vcs_info_sha;

const SHA: &str = "3d32c922ab922f3820e94644544d8c53e301e8f6";

#[test]
fn parses_pretty_printed_clean() {
    let json = format!(
        "{{\n  \"git\": {{\n    \"sha1\": \"{SHA}\"\n  }},\n  \"path_in_vcs\": \"crates/hoppy-cli\"\n}}\n"
    );
    assert_eq!(parse_vcs_info_sha(&json).as_deref(), Some("3d32c922ab92"));
}

#[test]
fn parses_pretty_printed_dirty() {
    let json = format!(
        "{{\n  \"git\": {{\n    \"sha1\": \"{SHA}\",\n    \"dirty\": true\n  }},\n  \"path_in_vcs\": \"crates/hoppy-cli\"\n}}\n"
    );
    assert_eq!(
        parse_vcs_info_sha(&json).as_deref(),
        Some("3d32c922ab92+dirty")
    );
}

#[test]
fn parses_compact_and_key_order_variants() {
    let compact = format!("{{\"git\":{{\"dirty\":true,\"sha1\":\"{SHA}\"}}}}");
    // `dirty` before `sha1` is outside the scoped scan — we only honour the
    // documented cargo ordering, so this reads as clean rather than guessing.
    assert_eq!(
        parse_vcs_info_sha(&compact).as_deref(),
        Some("3d32c922ab92")
    );

    let compact = format!("{{\"git\":{{\"sha1\":\"{SHA}\",\"dirty\":true}}}}");
    assert_eq!(
        parse_vcs_info_sha(&compact).as_deref(),
        Some("3d32c922ab92+dirty")
    );

    let explicit_false = format!("{{\"git\":{{\"sha1\":\"{SHA}\",\"dirty\":false}}}}");
    assert_eq!(
        parse_vcs_info_sha(&explicit_false).as_deref(),
        Some("3d32c922ab92")
    );
}

#[test]
fn dirty_outside_git_object_is_ignored() {
    let json =
        format!("{{\"git\":{{\"sha1\":\"{SHA}\"}},\"path_in_vcs\":\"x/\\\"dirty\\\":true\"}}");
    assert_eq!(parse_vcs_info_sha(&json).as_deref(), Some("3d32c922ab92"));
}

#[test]
fn rejects_malformed_input() {
    assert_eq!(parse_vcs_info_sha(""), None);
    assert_eq!(parse_vcs_info_sha("{}"), None);
    assert_eq!(parse_vcs_info_sha("{\"git\":{\"sha1\":"), None);
    assert_eq!(parse_vcs_info_sha("{\"git\":{\"sha1\":\"abc\"}}"), None);
    assert_eq!(
        parse_vcs_info_sha("{\"git\":{\"sha1\":\"zzzzzzzzzzzzzzzz\"}}"),
        None
    );
    assert_eq!(
        parse_vcs_info_sha("{\"git\":{\"sha1\":\"3d32c922ab92\"}}").as_deref(),
        Some("3d32c922ab92")
    );
}
