//! Resumable-upload orchestration for `hoppy stream video upload --resumable`.
//!
//! Wraps [`bunny_net_api::stream::TusUploader`] with:
//! - on-disk **session persistence** so a re-run resumes an interrupted upload
//!   instead of starting over (state file location is
//!   [`std::path::PathBuf`]-based and works on Windows/Linux/macOS);
//! - **retry with exponential backoff** on transient chunk failures, re-probing
//!   the server offset between attempts;
//! - a **progress bar** consistent with the single-shot PUT upload path.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use bunny_net_api::stream::TusUploader;
use bunny_net_api::stream::types::VideoUploadOptions;
use tokio::io::AsyncSeekExt;

use crate::progress;

/// Maximum number of attempts (initial try + retries) for the chunk loop.
const MAX_ATTEMPTS: u32 = 5;

/// Base backoff between retry attempts. Doubled each attempt.
const BASE_BACKOFF: Duration = Duration::from_millis(500);

/// Persisted resume state for a single in-flight TUS upload.
///
/// Serialised as JSON next to the other state files in the state directory.
/// The `location` is the TUS session URL returned by the server on creation;
/// re-running with the same `library_id` + `file` finds this record and
/// resumes from the server's current offset.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TusSession {
    library_id: i64,
    video_id: String,
    /// Absolute path of the source file, as a string (for display + validation).
    file: String,
    /// Total size of the source file in bytes at creation time.
    length: u64,
    /// The TUS upload session URL (`Location`) to `PATCH` chunks against.
    location: String,
}

/// Compute the state directory: an explicit `--state-dir`, or a `hoppy-tus`
/// subdirectory of the OS temp directory. Cross-platform via
/// [`std::env::temp_dir`] (no Unix-only assumptions).
fn state_dir(explicit: Option<&Path>) -> PathBuf {
    match explicit {
        Some(dir) => dir.to_path_buf(),
        None => std::env::temp_dir().join("hoppy-tus"),
    }
}

/// Deterministic state-file name for a given upload target.
///
/// Keyed on `library_id` + absolute file path so repeated runs of the same
/// command resolve to the same session file. Uses a hex SHA-256 to keep the
/// name filesystem-safe on every platform.
fn session_filename(library_id: i64, file_abs: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(library_id.to_string().as_bytes());
    hasher.update([0u8]); // separator
    hasher.update(file_abs.as_bytes());
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    const H: &[u8; 16] = b"0123456789abcdef";
    for &b in digest.iter() {
        hex.push(H[(b >> 4) as usize] as char);
        hex.push(H[(b & 0x0f) as usize] as char);
    }
    format!("tus-{hex}.json")
}

/// Load a previously persisted session, if one exists and is readable.
fn load_session(path: &Path) -> Option<TusSession> {
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Persist a session to disk, creating the state directory if needed.
fn save_session(path: &Path, session: &TusSession) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating TUS state directory {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(session).context("serialising TUS session state")?;
    std::fs::write(path, json)
        .with_context(|| format!("writing TUS session state to {}", path.display()))?;
    Ok(())
}

/// Remove a persisted session file, ignoring "not found" errors.
fn clear_session(path: &Path) {
    let _ = std::fs::remove_file(path);
}

/// Parameters for [`run_resumable_upload`], grouped to keep the arg list sane.
pub struct ResumableUpload<'a> {
    pub uploader: TusUploader,
    pub library_id: i64,
    pub video_id: &'a str,
    pub title: &'a str,
    pub file: &'a str,
    pub options: &'a VideoUploadOptions,
    pub state_dir: Option<&'a Path>,
    pub quiet: bool,
}

/// Drive a resumable TUS upload to completion, resuming from a persisted
/// session and the server-reported offset when one is available.
///
/// Returns the number of bytes uploaded on success. The session file is
/// removed once the upload completes.
pub async fn run_resumable_upload(params: ResumableUpload<'_>) -> Result<u64> {
    let ResumableUpload {
        uploader,
        library_id,
        video_id,
        title,
        file,
        options,
        state_dir: state_dir_override,
        quiet,
    } = params;

    // Resolve an absolute path for a stable session key; fall back to the raw
    // path if canonicalisation fails (e.g. symlink quirks) — the key just needs
    // to be stable per invocation.
    let file_abs = std::fs::canonicalize(file)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| file.to_owned());

    let dir = state_dir(state_dir_override);
    let session_path = dir.join(session_filename(library_id, &file_abs));

    // Open the source file and measure it.
    let mut fh = tokio::fs::File::open(file)
        .await
        .with_context(|| format!("opening file: {file}"))?;
    let length = fh
        .metadata()
        .await
        .with_context(|| format!("reading metadata for: {file}"))?
        .len();

    // Reuse a valid existing session, else create a fresh TUS upload.
    let mut resumed = false;
    let location = match load_session(&session_path) {
        Some(prev)
            if prev.video_id == video_id && prev.file == file_abs && prev.length == length =>
        {
            resumed = true;
            prev.location
        }
        _ => {
            let loc = uploader.create(length, title, options).await?;
            let session = TusSession {
                library_id,
                video_id: video_id.to_owned(),
                file: file_abs.clone(),
                length,
                location: loc.clone(),
            };
            save_session(&session_path, &session)?;
            loc
        }
    };

    // Probe the current server offset. If the session is stale/gone, restart
    // once from scratch rather than failing.
    let mut start_offset = match uploader.offset(&location).await {
        Ok(off) => off,
        Err(_) if resumed => {
            // Session expired server-side — recreate and start over.
            clear_session(&session_path);
            let loc = uploader.create(length, title, options).await?;
            let session = TusSession {
                library_id,
                video_id: video_id.to_owned(),
                file: file_abs.clone(),
                length,
                location: loc.clone(),
            };
            save_session(&session_path, &session)?;
            return finish_upload(
                &uploader,
                &loc,
                &mut fh,
                0,
                length,
                &session_path,
                quiet,
                video_id,
            )
            .await;
        }
        Err(e) => return Err(e),
    };

    // Guard against a server offset beyond the file (should not happen).
    if start_offset > length {
        start_offset = length;
    }

    // We reuse `location` from here on.
    finish_upload(
        &uploader,
        &location,
        &mut fh,
        start_offset,
        length,
        &session_path,
        quiet,
        video_id,
    )
    .await
}

/// Seek to `start_offset`, then run the chunk loop with retry/backoff and a
/// progress bar. Clears the session on success.
#[allow(clippy::too_many_arguments)]
async fn finish_upload(
    uploader: &TusUploader,
    location: &str,
    fh: &mut tokio::fs::File,
    start_offset: u64,
    length: u64,
    session_path: &Path,
    quiet: bool,
    video_id: &str,
) -> Result<u64> {
    let pb = progress::file_progress(length, quiet);
    if let Some(bar) = &pb {
        bar.set_position(start_offset);
    }

    let mut offset = start_offset;
    let mut attempt: u32 = 0;

    loop {
        if offset >= length {
            break;
        }
        // Position the reader at the current offset for this attempt.
        fh.seek(std::io::SeekFrom::Start(offset))
            .await
            .with_context(|| format!("seeking source file to offset {offset}"))?;

        let pb_ref = pb.clone();
        let result = uploader
            .upload_reader(location, fh, offset, length, move |o| {
                if let Some(bar) = &pb_ref {
                    bar.set_position(o);
                }
            })
            .await;

        match result {
            Ok(res) => {
                offset = res.uploaded;
                if res.complete {
                    break;
                }
            }
            Err(e) => {
                attempt += 1;
                if attempt >= MAX_ATTEMPTS {
                    return Err(e).with_context(|| {
                        format!(
                            "TUS upload failed after {MAX_ATTEMPTS} attempts; \
                             re-run the same command to resume from offset {offset}"
                        )
                    });
                }
                // Back off, then re-probe the server's authoritative offset so
                // we resume from exactly where it got to.
                let backoff = BASE_BACKOFF * 2u32.pow(attempt - 1);
                tokio::time::sleep(backoff).await;
                match uploader.offset(location).await {
                    Ok(server_offset) => {
                        offset = server_offset.min(length);
                        if let Some(bar) = &pb {
                            bar.set_position(offset);
                        }
                    }
                    Err(probe_err) => {
                        // Can't even probe — surface the original error.
                        return Err(e).with_context(|| {
                            format!("and offset re-probe also failed: {probe_err}")
                        });
                    }
                }
            }
        }
    }

    if offset < length {
        bail!(
            "TUS upload ended at offset {offset} of {length} bytes; \
             re-run the same command to resume"
        );
    }

    // Done — the video bytes are all on the server.
    clear_session(session_path);
    progress::finish_with_message(pb.as_ref(), format!("Uploaded {video_id} (resumable)"));
    Ok(offset)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_filename_is_stable_and_keyed() {
        let a = session_filename(1, "/tmp/video.mp4");
        let b = session_filename(1, "/tmp/video.mp4");
        assert_eq!(a, b, "same inputs must yield same name");
        assert!(a.starts_with("tus-") && a.ends_with(".json"));
        // Different library or file changes the name.
        assert_ne!(a, session_filename(2, "/tmp/video.mp4"));
        assert_ne!(a, session_filename(1, "/tmp/other.mp4"));
    }

    #[test]
    fn state_dir_defaults_to_temp_subdir() {
        let dir = state_dir(None);
        assert!(dir.ends_with("hoppy-tus"));
        let explicit = std::path::Path::new("/custom/dir");
        assert_eq!(state_dir(Some(explicit)), explicit);
    }

    #[test]
    fn session_roundtrips_through_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("session.json");
        let session = TusSession {
            library_id: 42,
            video_id: "guid-1".to_string(),
            file: "/tmp/a.mp4".to_string(),
            length: 12345,
            location: "https://video.bunnycdn.com/tusupload/xyz".to_string(),
        };
        save_session(&path, &session).unwrap();
        let loaded = load_session(&path).expect("session should load");
        assert_eq!(loaded.library_id, 42);
        assert_eq!(loaded.video_id, "guid-1");
        assert_eq!(loaded.length, 12345);
        assert_eq!(loaded.location, session.location);

        clear_session(&path);
        assert!(load_session(&path).is_none());
    }

    #[test]
    fn load_session_returns_none_for_missing_or_garbage() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("nope.json");
        assert!(load_session(&missing).is_none());

        let garbage = tmp.path().join("garbage.json");
        std::fs::write(&garbage, "not json at all").unwrap();
        assert!(load_session(&garbage).is_none());
    }
}
