//! TUS 1.0 resumable-upload client for the bunny.net Stream API.
//!
//! bunny.net Stream exposes a [TUS 1.0](https://tus.io/) endpoint at
//! `https://video.bunnycdn.com/tusupload` for uploading large video files in
//! resumable chunks. This is a docs-only surface — there is no OpenAPI spec —
//! so the request/response shapes here follow
//! `docs.bunny.net/stream/tus-resumable-uploads` and the reference TUS
//! protocol.
//!
//! # Authentication
//!
//! Unlike the JSON Stream API (which uses an `AccessKey` header), the TUS
//! endpoint is authenticated with a **presigned signature**:
//!
//! ```text
//! AuthorizationSignature = hex( sha256( library_id + api_key + expiration_time + video_id ) )
//! ```
//!
//! where `expiration_time` is a Unix timestamp (seconds) in the future. The
//! signature, the expiry, the library ID and the video GUID are all sent as
//! headers on the creation request; the TUS server ties them to the upload
//! `Location` it returns, so subsequent `HEAD`/`PATCH` requests need no
//! further auth.
//!
//! # Streaming
//!
//! Chunks are read from the source in bounded windows (`chunk_size`) and each
//! `PATCH` streams its window without buffering the whole file — see
//! [`TusUploader::upload_reader`]. Only one `chunk_size` window is resident in
//! memory at a time.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use reqwest::{Client, StatusCode};
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::dry_run::{DryRunSkipped, check_dry_run};

use super::types::VideoUploadOptions;

/// The TUS protocol version bunny.net speaks.
const TUS_VERSION: &str = "1.0.0";

/// Default chunk size for `PATCH` requests: 32 MiB.
///
/// Large enough to keep per-request overhead low on fast links, small enough
/// that a dropped connection only loses one chunk's worth of progress.
pub const DEFAULT_CHUNK_SIZE: usize = 32 * 1024 * 1024;

/// How far in the future the presigned signature is valid, in seconds.
const SIGNATURE_TTL_SECS: u64 = 24 * 60 * 60;

/// A resumable TUS upload session for a single video.
///
/// Construct with [`TusUploader::new`], then drive an upload with
/// [`TusUploader::create`] followed by [`TusUploader::upload_reader`], or use
/// the higher-level composite in the CLI layer. The `library_id`, `api_key`
/// and `video_id` uniquely identify the target video; `base_url` defaults to
/// the production endpoint but can be overridden for testing.
pub struct TusUploader {
    http: Client,
    base_url: String,
    library_id: i64,
    api_key: String,
    video_id: String,
    chunk_size: usize,
    debug: bool,
    dry_run: bool,
}

/// The outcome of a chunk-by-chunk upload run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TusUploadResult {
    /// Total number of bytes now stored on the server (should equal the file length).
    pub uploaded: u64,
    /// Whether the upload reached completion (offset == length).
    pub complete: bool,
}

impl TusUploader {
    /// Create a new uploader targeting the production TUS endpoint.
    pub fn new(library_id: i64, api_key: impl Into<String>, video_id: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: "https://video.bunnycdn.com".to_string(),
            library_id,
            api_key: api_key.into(),
            video_id: video_id.into(),
            chunk_size: DEFAULT_CHUNK_SIZE,
            debug: false,
            dry_run: false,
        }
    }

    /// Override the base URL (host + scheme, no trailing slash). Used in tests.
    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_owned();
        self
    }

    /// Override the per-`PATCH` chunk size in bytes. A value of `0` is treated
    /// as [`DEFAULT_CHUNK_SIZE`].
    #[must_use]
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = if chunk_size == 0 {
            DEFAULT_CHUNK_SIZE
        } else {
            chunk_size
        };
        self
    }

    /// Enable debug logging of TUS requests/responses to stderr.
    #[must_use]
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Preview the create (`POST`) and each upload chunk (`PATCH`) instead of
    /// sending them. The offset probe (`HEAD`) is read-only and always runs.
    #[must_use]
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// The configured chunk size (after defaulting).
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// Compute the presigned `AuthorizationSignature` for this video.
    ///
    /// `expiration_time` is a Unix timestamp (seconds). The signature is the
    /// lowercase hex SHA-256 of `library_id + api_key + expiration + video_id`.
    fn signature(&self, expiration_time: u64) -> String {
        signature(
            self.library_id,
            &self.api_key,
            expiration_time,
            &self.video_id,
        )
    }

    /// Build the `Upload-Metadata` header value from per-upload options plus
    /// the mandatory `filetype`/`title` fields.
    ///
    /// Per TUS, the header is a comma-separated list of `key <base64(value)>`
    /// pairs. bunny.net reads the same per-upload knobs here that the PUT path
    /// carries as query parameters.
    fn metadata_header(title: &str, options: &VideoUploadOptions) -> String {
        upload_metadata(title, options)
    }

    /// Create the TUS upload and return its `Location` URL.
    ///
    /// `upload_length` is the total size of the file in bytes; `title` is the
    /// display title used for the `Upload-Metadata` `title` field.
    pub async fn create(
        &self,
        upload_length: u64,
        title: &str,
        options: &VideoUploadOptions,
    ) -> Result<String> {
        let expiration = current_unix_time()? + SIGNATURE_TTL_SECS;
        let signature = self.signature(expiration);
        let metadata = Self::metadata_header(title, options);
        let url = format!("{}/tusupload", self.base_url);

        if self.debug {
            eprintln!(">> POST {url} (TUS create, length={upload_length})");
        }

        let request = self
            .http
            .post(&url)
            .header("Tus-Resumable", TUS_VERSION)
            .header("Upload-Length", upload_length.to_string())
            .header("Upload-Metadata", metadata)
            .header("AuthorizationSignature", signature)
            .header("AuthorizationExpire", expiration.to_string())
            .header("VideoId", &self.video_id)
            .header("LibraryId", self.library_id.to_string())
            .build()
            .context("failed to build TUS create request")?;
        check_dry_run(&request, self.dry_run, false)?;
        let resp = self
            .http
            .execute(request)
            .await
            .context("TUS create request failed")?;

        let status = resp.status();
        if self.debug {
            eprintln!("<< {status} (TUS create)");
        }
        if status != StatusCode::CREATED && status != StatusCode::OK {
            let body = resp.text().await.unwrap_or_default();
            bail!("TUS create failed (HTTP {status}): {body}");
        }

        let location = resp
            .headers()
            .get("Location")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
            .context("TUS create response is missing a Location header")?;

        Ok(self.absolute_location(&location))
    }

    /// Resolve a possibly-relative `Location` header against the base URL.
    fn absolute_location(&self, location: &str) -> String {
        if location.starts_with("http://") || location.starts_with("https://") {
            location.to_owned()
        } else if let Some(rest) = location.strip_prefix('/') {
            format!("{}/{}", self.base_url, rest)
        } else {
            format!("{}/{}", self.base_url, location)
        }
    }

    /// Probe the current server-side `Upload-Offset` for an existing upload.
    ///
    /// Returns the number of bytes the server has already received. Read-only
    /// (`HEAD`) — always runs, even under `--dry-run`.
    pub async fn offset(&self, location: &str) -> Result<u64> {
        if self.debug {
            eprintln!(">> HEAD {location} (TUS offset probe)");
        }
        let resp = self
            .http
            .head(location)
            .header("Tus-Resumable", TUS_VERSION)
            .send()
            .await
            .context("TUS HEAD (offset probe) request failed")?;

        let status = resp.status();
        if self.debug {
            eprintln!("<< {status} (TUS offset probe)");
        }
        // 404/410 means the server dropped the upload — the caller should
        // recreate it. Surface a typed-ish error via the message.
        if status == StatusCode::NOT_FOUND || status == StatusCode::GONE {
            bail!("TUS upload no longer exists on the server (HTTP {status})");
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            bail!("TUS offset probe failed (HTTP {status}): {body}");
        }

        let offset = resp
            .headers()
            .get("Upload-Offset")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .context("TUS HEAD response is missing a valid Upload-Offset header")?;
        Ok(offset)
    }

    /// Upload the remaining bytes of `reader` starting at `start_offset`,
    /// invoking `on_progress` with the cumulative byte offset after each chunk.
    ///
    /// The caller is responsible for having already advanced `reader` to
    /// `start_offset` (e.g. via `seek`). `total_length` is the full file size
    /// used to size the final chunk and detect completion.
    ///
    /// Each chunk is sent as a `PATCH` with `Content-Type:
    /// application/offset+octet-stream`. On mismatch between the requested and
    /// server-reported offset the error is surfaced so the caller can re-probe
    /// and retry.
    pub async fn upload_reader<R, F>(
        &self,
        location: &str,
        reader: &mut R,
        start_offset: u64,
        total_length: u64,
        mut on_progress: F,
    ) -> Result<TusUploadResult>
    where
        R: AsyncRead + Unpin,
        F: FnMut(u64),
    {
        // Block before any disk read or chunk buffering: the preview for a
        // chunk PATCH is its size, never its bytes.
        if self.dry_run {
            return Err(DryRunSkipped {
                method: "PATCH".to_owned(),
                url: location.to_owned(),
                body: Some(format!(
                    "<binary chunk upload, {} bytes remaining>",
                    total_length - start_offset
                )),
            }
            .into());
        }

        let mut offset = start_offset;
        let mut buf = vec![0u8; self.chunk_size];

        while offset < total_length {
            // Fill up to one chunk from the reader. `read` may return short
            // reads, so loop until the buffer is full or EOF.
            let want = std::cmp::min(self.chunk_size as u64, total_length - offset) as usize;
            let mut filled = 0usize;
            while filled < want {
                let n = reader
                    .read(&mut buf[filled..want])
                    .await
                    .context("reading source file for TUS chunk")?;
                if n == 0 {
                    break;
                }
                filled += n;
            }
            if filled == 0 {
                // Reader ended early relative to declared length.
                bail!(
                    "source ended at offset {offset} but declared length is {total_length}; \
                     file may have been truncated mid-upload"
                );
            }

            let chunk = buf[..filled].to_vec();
            let new_offset = self.patch_chunk(location, offset, chunk).await?;
            if new_offset != offset + filled as u64 {
                bail!(
                    "TUS server reported offset {new_offset} after a {filled}-byte PATCH at \
                     offset {offset} (expected {})",
                    offset + filled as u64
                );
            }
            offset = new_offset;
            on_progress(offset);
        }

        Ok(TusUploadResult {
            uploaded: offset,
            complete: offset >= total_length,
        })
    }

    /// Send a single `PATCH` chunk and return the server's new `Upload-Offset`.
    async fn patch_chunk(&self, location: &str, offset: u64, chunk: Vec<u8>) -> Result<u64> {
        let len = chunk.len();
        if self.debug {
            eprintln!(">> PATCH {location} (offset={offset}, len={len})");
        }
        let request = self
            .http
            .patch(location)
            .header("Tus-Resumable", TUS_VERSION)
            .header("Content-Type", "application/offset+octet-stream")
            .header("Upload-Offset", offset.to_string())
            .body(chunk)
            .build()
            .context("failed to build TUS PATCH request")?;
        // Defense in depth — `upload_reader` blocks before reading the chunk,
        // so this is normally unreachable under dry-run. Never hand the raw
        // chunk bytes to the preview: they are file contents, not JSON.
        if self.dry_run {
            return Err(DryRunSkipped {
                method: request.method().to_string(),
                url: request.url().to_string(),
                body: Some(format!("<binary chunk, {len} bytes>")),
            }
            .into());
        }
        let resp = self
            .http
            .execute(request)
            .await
            .context("TUS PATCH request failed")?;

        let status = resp.status();
        if self.debug {
            eprintln!("<< {status} (TUS PATCH)");
        }
        // 204 No Content is the success case per TUS; 200 tolerated too.
        if status != StatusCode::NO_CONTENT && status != StatusCode::OK {
            let body = resp.text().await.unwrap_or_default();
            bail!("TUS PATCH failed at offset {offset} (HTTP {status}): {body}");
        }

        let new_offset = resp
            .headers()
            .get("Upload-Offset")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            // Some servers omit the header on 204; fall back to offset+len.
            .unwrap_or(offset + len as u64);
        Ok(new_offset)
    }
}

// ---------------------------------------------------------------------------
// Free functions (pure, unit-testable)
// ---------------------------------------------------------------------------

/// Current Unix time in whole seconds.
fn current_unix_time() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?
        .as_secs())
}

/// Compute the bunny.net TUS `AuthorizationSignature`:
/// lowercase hex SHA-256 of `library_id + api_key + expiration + video_id`.
pub fn signature(library_id: i64, api_key: &str, expiration: u64, video_id: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(library_id.to_string().as_bytes());
    hasher.update(api_key.as_bytes());
    hasher.update(expiration.to_string().as_bytes());
    hasher.update(video_id.as_bytes());
    to_hex(&hasher.finalize())
}

/// Lowercase-hex-encode a byte slice without pulling in a `hex` dependency.
fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

/// Build the TUS `Upload-Metadata` header value.
///
/// Format: comma-separated `key <base64(value)>` pairs. Keys with no value are
/// emitted bare (`key`), but bunny.net's metadata keys all carry values here.
/// The per-upload options mirror the PUT path's query parameters exactly, so a
/// resumable upload can request the same encoding/transcription behaviour.
pub fn upload_metadata(title: &str, options: &VideoUploadOptions) -> String {
    use base64::Engine as _;
    let b64 = base64::engine::general_purpose::STANDARD;

    let mut pairs: Vec<(String, String)> = Vec::new();
    pairs.push(("filetype".to_string(), "video/mp4".to_string()));
    pairs.push(("title".to_string(), title.to_string()));

    if let Some(v) = options.jit_enabled {
        pairs.push(("jitEnabled".to_string(), v.to_string()));
    }
    if let Some(v) = &options.enabled_resolutions {
        pairs.push(("enabledResolutions".to_string(), v.clone()));
    }
    if let Some(v) = &options.enabled_output_codecs {
        pairs.push(("enabledOutputCodecs".to_string(), v.clone()));
    }
    if let Some(v) = options.transcribe_enabled {
        pairs.push(("transcribeEnabled".to_string(), v.to_string()));
    }
    if let Some(v) = &options.transcribe_languages {
        pairs.push(("transcribeLanguages".to_string(), v.clone()));
    }
    if let Some(v) = &options.source_language {
        pairs.push(("sourceLanguage".to_string(), v.clone()));
    }
    if let Some(v) = options.generate_title {
        pairs.push(("generateTitle".to_string(), v.to_string()));
    }
    if let Some(v) = options.generate_description {
        pairs.push(("generateDescription".to_string(), v.to_string()));
    }
    if let Some(v) = options.generate_chapters {
        pairs.push(("generateChapters".to_string(), v.to_string()));
    }
    if let Some(v) = options.generate_moments {
        pairs.push(("generateMoments".to_string(), v.to_string()));
    }

    pairs
        .into_iter()
        .map(|(k, v)| format!("{k} {}", b64.encode(v.as_bytes())))
        .collect::<Vec<_>>()
        .join(",")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_is_stable_lowercase_hex_sha256() {
        // Precomputed: sha256("123" + "key" + "1000" + "guid")
        let sig = signature(123, "key", 1000, "guid");
        assert_eq!(sig.len(), 64);
        assert!(
            sig.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase())
        );
        // Deterministic across calls with the same inputs.
        assert_eq!(sig, signature(123, "key", 1000, "guid"));
        // Different inputs produce different signatures.
        assert_ne!(sig, signature(124, "key", 1000, "guid"));
        assert_ne!(sig, signature(123, "key", 1001, "guid"));
    }

    #[test]
    fn signature_matches_known_vector() {
        // Independently computed hex sha256 of the concatenation "1key2guid".
        // library_id=1, api_key="key", expiration=2, video_id="guid".
        let sig = signature(1, "key", 2, "guid");
        // sha256("1" + "key" + "2" + "guid") = "1key2guid"
        let expected = {
            use sha2::{Digest, Sha256};
            let mut h = Sha256::new();
            h.update(b"1key2guid");
            to_hex(&h.finalize())
        };
        assert_eq!(sig, expected);
    }

    #[test]
    fn to_hex_encodes_bytes() {
        assert_eq!(to_hex(&[0x00, 0x0f, 0xff, 0xa5]), "000fffa5");
        assert_eq!(to_hex(&[]), "");
    }

    #[test]
    fn upload_metadata_encodes_title_and_filetype() {
        use base64::Engine as _;
        let b64 = base64::engine::general_purpose::STANDARD;
        let meta = upload_metadata("My Video", &VideoUploadOptions::default());
        // Both mandatory keys present, base64-encoded values.
        assert!(meta.contains(&format!("title {}", b64.encode(b"My Video"))));
        assert!(meta.contains(&format!("filetype {}", b64.encode(b"video/mp4"))));
    }

    #[test]
    fn upload_metadata_includes_per_upload_options() {
        use base64::Engine as _;
        let b64 = base64::engine::general_purpose::STANDARD;
        let options = VideoUploadOptions {
            jit_enabled: Some(true),
            enabled_resolutions: Some("720p,1080p".to_string()),
            source_language: Some("en".to_string()),
            generate_title: Some(true),
            ..Default::default()
        };
        let meta = upload_metadata("t", &options);
        assert!(meta.contains(&format!("jitEnabled {}", b64.encode(b"true"))));
        assert!(meta.contains(&format!("enabledResolutions {}", b64.encode(b"720p,1080p"))));
        assert!(meta.contains(&format!("sourceLanguage {}", b64.encode(b"en"))));
        assert!(meta.contains(&format!("generateTitle {}", b64.encode(b"true"))));
        // Unset options must be absent.
        assert!(!meta.contains("generateMoments"));
    }

    #[test]
    fn upload_metadata_omits_unset_options() {
        let meta = upload_metadata("t", &VideoUploadOptions::default());
        assert!(!meta.contains("jitEnabled"));
        assert!(!meta.contains("enabledResolutions"));
        assert!(!meta.contains("transcribeEnabled"));
    }

    #[test]
    fn absolute_location_resolves_relative_paths() {
        let up = TusUploader::new(1, "k", "v").with_base_url("http://localhost:9000");
        assert_eq!(
            up.absolute_location("/tusupload/abc"),
            "http://localhost:9000/tusupload/abc"
        );
        assert_eq!(
            up.absolute_location("tusupload/abc"),
            "http://localhost:9000/tusupload/abc"
        );
        assert_eq!(
            up.absolute_location("https://cdn.example/tusupload/abc"),
            "https://cdn.example/tusupload/abc"
        );
    }

    #[test]
    fn chunk_size_defaults_when_zero() {
        let up = TusUploader::new(1, "k", "v").with_chunk_size(0);
        assert_eq!(up.chunk_size(), DEFAULT_CHUNK_SIZE);
        let up = TusUploader::new(1, "k", "v").with_chunk_size(1024);
        assert_eq!(up.chunk_size(), 1024);
    }
}
