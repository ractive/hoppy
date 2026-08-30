//! Parser for `.cargo_vcs_info.json`, shared between `build.rs` (via
//! `#[path]`) and the e2e test module so the hand-rolled scan is covered by
//! `cargo test` even though build scripts themselves never are.
//!
//! Observed shape (cargo pretty-prints it):
//!
//! ```json
//! {
//!   "git": {
//!     "sha1": "3d32c922ab922f3820e94644544d8c53e301e8f6",
//!     "dirty": true
//!   },
//!   "path_in_vcs": "crates/hoppy-cli"
//! }
//! ```
//!
//! `dirty` is only emitted by newer cargo when the tree had uncommitted
//! changes (`--allow-dirty`). Plain string scan keeps the build script
//! dependency-free; anything unexpected yields `None`.

/// Extract the 12-char short SHA (with `+dirty` when flagged) from the
/// contents of `.cargo_vcs_info.json`.
pub fn parse_vcs_info_sha(contents: &str) -> Option<String> {
    let after_key = &contents[contents.find("\"sha1\"")? + "\"sha1\"".len()..];
    let after_colon = &after_key[after_key.find(':')? + 1..];
    let start = after_colon.find('"')? + 1;
    let end = start + after_colon[start..].find('"')?;
    let sha = &after_colon[start..end];
    if sha.len() < 12 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let short = &sha[..12];
    // Scope the dirty scan to what follows the sha, i.e. the rest of the
    // `git` object, so a coincidental substring elsewhere can't flag it.
    let rest = &after_colon[end..];
    let rest = &rest[..rest.find('}').unwrap_or(rest.len())];
    let dirty = rest
        .split_once("\"dirty\"")
        .and_then(|(_, v)| v.split_once(':'))
        .is_some_and(|(_, v)| v.trim_start().starts_with("true"));
    if dirty {
        Some(format!("{short}+dirty"))
    } else {
        Some(short.to_owned())
    }
}
