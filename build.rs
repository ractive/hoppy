//! Embed build provenance (git SHA + bunny API spec date) into the binary so
//! `hoppy --version` is reproducibility-friendly. See iter-19 for context.
//!
//! Inputs are best-effort: if `git` is unavailable or the working tree isn't a
//! git checkout (e.g. `cargo install` from crates.io), the SHA falls back to
//! `unknown` rather than failing the build.

use std::path::Path;
use std::process::Command;

fn main() {
    let sha = git_sha().unwrap_or_else(|| "unknown".to_owned());
    println!("cargo:rustc-env=HOPPY_BUILD_SHA={sha}");

    let spec_date = newest_spec_mtime().unwrap_or_else(|| "unknown".to_owned());
    println!("cargo:rustc-env=HOPPY_BUNNY_API_SPEC_DATE={spec_date}");

    // Re-run when HEAD moves or the openapi specs change.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=specs");
}

fn git_sha() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
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

fn newest_spec_mtime() -> Option<String> {
    let specs = Path::new("specs");
    if !specs.is_dir() {
        return None;
    }
    let mut newest: Option<std::time::SystemTime> = None;
    for entry in std::fs::read_dir(specs).ok()?.flatten() {
        let meta = entry.metadata().ok()?;
        if let Ok(modified) = meta.modified() {
            newest = Some(newest.map_or(modified, |n| n.max(modified)));
        }
    }
    let modified = newest?;
    let secs = modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    // Format as YYYY-MM-DD without pulling in a date crate.
    Some(format_yyyymmdd(secs))
}

fn format_yyyymmdd(secs: i64) -> String {
    // Civil-from-days algorithm (Howard Hinnant, public domain).
    let days = secs.div_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}
