//! Embed build provenance (short git SHA + commit date) into the binary so
//! `hoppy -V` is reproducibility-friendly. Inputs are best-effort:
//!
//! * `CARGO_HOPPY_FORCE_NO_GIT=1` forces empty values (used in repro tests).
//! * `GIT_COMMIT` / `GIT_COMMIT_DATE` override the values when set (used by
//!   CI / tarball builds that don't ship a `.git` tree).
//! * A published crate (`cargo install hoppy-cli` from crates.io) carries no
//!   `.git` tree but does carry `.cargo_vcs_info.json`, written by
//!   `cargo publish` with the commit it was packaged from. When that file is
//!   present we take the SHA from it (no commit date is recorded there, so
//!   `-V` prints `hoppy 0.7.0 (3d32c922ab92)`).
//! * Otherwise we shell out to `git`. Missing-git or missing-checkout fall
//!   back to empty strings rather than failing the build.
//!
//! When the working tree has uncommitted changes the SHA is suffixed with
//! `+dirty` so dogfooders can spot ad-hoc builds.

use std::path::PathBuf;
use std::process::Command;

const ENV_FORCE_NO_GIT: &str = "CARGO_HOPPY_FORCE_NO_GIT";
const ENV_GIT_COMMIT: &str = "GIT_COMMIT";
const ENV_GIT_COMMIT_DATE: &str = "GIT_COMMIT_DATE";
const CARGO_VCS_INFO: &str = ".cargo_vcs_info.json";

fn main() {
    println!("cargo:rerun-if-env-changed={ENV_FORCE_NO_GIT}");
    println!("cargo:rerun-if-env-changed={ENV_GIT_COMMIT}");
    println!("cargo:rerun-if-env-changed={ENV_GIT_COMMIT_DATE}");

    let force_no_git = std::env::var_os(ENV_FORCE_NO_GIT).is_some_and(|v| v == "1");

    let (sha, date) = if force_no_git {
        (String::new(), String::new())
    } else if let Some(sha) = env_override(ENV_GIT_COMMIT) {
        // CI / hermetic path: caller-supplied SHA, date from env or git.
        let date = env_override(ENV_GIT_COMMIT_DATE)
            .or_else(git_commit_date)
            .unwrap_or_default();
        (sha, date)
    } else if let Some(sha) = vcs_info_sha() {
        // Published-tarball path: `.cargo_vcs_info.json` records the SHA but
        // no date, and there is no `.git` tree to ask, so only an explicit
        // GIT_COMMIT_DATE can fill the date in.
        (sha, env_override(ENV_GIT_COMMIT_DATE).unwrap_or_default())
    } else {
        let sha = git_short_sha()
            .map(|raw| {
                if git_is_dirty() {
                    format!("{raw}+dirty")
                } else {
                    raw
                }
            })
            .unwrap_or_default();
        let date = env_override(ENV_GIT_COMMIT_DATE)
            .or_else(git_commit_date)
            .unwrap_or_default();
        (sha, date)
    };

    println!("cargo:rustc-env=HOPPY_BUILD_VERSION_SHA={sha}");
    println!("cargo:rustc-env=HOPPY_BUILD_DATE={date}");

    // Rerun when HEAD or any ref moves. `git rev-parse --git-dir` handles
    // worktrees (where .git is a file pointing into the parent's gitdir).
    if let Some(git_dir) = git_dir() {
        let head = git_dir.join("HEAD");
        let refs = git_dir.join("refs");
        if head.exists() {
            println!("cargo:rerun-if-changed={}", head.display());
        }
        if refs.is_dir() {
            println!("cargo:rerun-if-changed={}", refs.display());
        }
    }
    // Also rerun when files in this package change so `+dirty` stays
    // accurate when the working tree flips between clean/dirty without
    // touching HEAD or refs.
    println!("cargo:rerun-if-changed=src");
}

fn env_override(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|s| !s.is_empty())
}

/// Read the packaging commit from `.cargo_vcs_info.json` next to `Cargo.toml`.
///
/// The file is only present inside a `cargo package`/`cargo publish` tarball
/// and looks like `{"git":{"sha1":"<40 hex>","dirty":true},"path_in_vcs":…}`
/// (`dirty` is emitted by newer cargo only when the tree had uncommitted
/// changes, e.g. `--allow-dirty`). Parsed with a plain string scan to keep
/// the build script dependency-free; anything unexpected yields `None` and
/// we fall through to `git`.
fn vcs_info_sha() -> Option<String> {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")?;
    let path = PathBuf::from(manifest_dir).join(CARGO_VCS_INFO);
    println!("cargo:rerun-if-changed={}", path.display());
    let contents = std::fs::read_to_string(path).ok()?;
    parse_vcs_info_sha(&contents)
}

fn parse_vcs_info_sha(contents: &str) -> Option<String> {
    let after_key = &contents[contents.find("\"sha1\"")? + "\"sha1\"".len()..];
    let after_colon = &after_key[after_key.find(':')? + 1..];
    let start = after_colon.find('"')? + 1;
    let end = start + after_colon[start..].find('"')?;
    let sha = &after_colon[start..end];
    if sha.len() < 12 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let short = &sha[..12];
    if contents.contains("\"dirty\":true") || contents.contains("\"dirty\": true") {
        Some(format!("{short}+dirty"))
    } else {
        Some(short.to_owned())
    }
}

fn git_dir() -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8(output.stdout).ok()?;
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

fn git_short_sha() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8(output.stdout).ok()?;
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

fn git_commit_date() -> Option<String> {
    let output = Command::new("git")
        .args(["show", "-s", "--format=%cs", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8(output.stdout).ok()?;
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

fn git_is_dirty() -> bool {
    let Ok(output) = Command::new("git").args(["status", "--porcelain"]).output() else {
        return false;
    };
    output.status.success() && !output.stdout.is_empty()
}
