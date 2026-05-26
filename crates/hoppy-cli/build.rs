//! Embed build provenance (short git SHA + commit date) into the binary so
//! `hoppy -V` is reproducibility-friendly. Inputs are best-effort:
//!
//! * `CARGO_HOPPY_FORCE_NO_GIT=1` forces empty values (used in repro tests).
//! * `GIT_COMMIT` / `GIT_COMMIT_DATE` override the values when set (used by
//!   CI / tarball builds that don't ship a `.git` tree).
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

fn main() {
    println!("cargo:rerun-if-env-changed={ENV_FORCE_NO_GIT}");
    println!("cargo:rerun-if-env-changed={ENV_GIT_COMMIT}");
    println!("cargo:rerun-if-env-changed={ENV_GIT_COMMIT_DATE}");

    let force_no_git = std::env::var_os(ENV_FORCE_NO_GIT).is_some_and(|v| v == "1");

    let (sha, date) = if force_no_git {
        (String::new(), String::new())
    } else {
        let sha = std::env::var(ENV_GIT_COMMIT)
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                let raw = git_short_sha()?;
                if git_is_dirty() {
                    Some(format!("{raw}+dirty"))
                } else {
                    Some(raw)
                }
            })
            .unwrap_or_default();

        let date = std::env::var(ENV_GIT_COMMIT_DATE)
            .ok()
            .filter(|s| !s.is_empty())
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
