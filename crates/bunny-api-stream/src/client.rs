use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{Context, Result, bail};
use reqwest::{Client, RequestBuilder};

use bunny_api_recording::{capture_request, maybe_record_response};

use crate::types::{
    Collection, CreateCollection, CreateVideo, EncoderOutputCodec, FetchVideo, PaginatedList,
    SmartGenerateSettings, StatusEnvelope, StatusMessage, TranscribeSettings, UpdateCollection,
    UpdateVideo, Video, VideoHeatmap, VideoResolutionsInfo, VideoStatistics, VideoStorageSize,
};

const BASE_URL: &str = "https://video.bunnycdn.com";

/// Options for [`StreamClient::cleanup_video_resolutions`].
///
/// Defaults match the API defaults: nothing is deleted unless explicitly
/// requested. `dry_run` returns the work that would be done without
/// performing it.
#[derive(Debug, Clone, Default)]
pub struct StreamCleanupResolutions<'a> {
    /// Comma-separated list of explicit resolutions to delete (e.g. `"720p,480p"`).
    pub resolutions_to_delete: Option<&'a str>,
    /// Delete every rendition that is not in the library's configured set.
    pub delete_non_configured_resolutions: bool,
    /// Delete the original uploaded file.
    pub delete_original: bool,
    /// Delete the MP4 fallback files.
    pub delete_mp4_files: bool,
    /// Preview the cleanup without actually changing anything.
    pub dry_run: bool,
}

/// Client for the bunny.net Stream (Video) API.
///
/// Construct with [`StreamClient::new`] and then call any of the available
/// methods. All methods are `async` and return [`anyhow::Result`].
///
/// # Authentication
/// The client attaches the API key as an `AccessKey` header on every request,
/// matching the bunny.net Stream API security scheme.
pub struct StreamClient {
    http: Client,
    base_url: String,
    api_key: String,
    debug: bool,
    record_dir: Option<PathBuf>,
    last_request: Mutex<Option<(String, String)>>,
}

impl StreamClient {
    /// Create a new client using the provided API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: BASE_URL.to_string(),
            api_key: api_key.into(),
            debug: false,
            record_dir: None,
            last_request: Mutex::new(None),
        }
    }

    /// Override the base URL (useful for testing against a mock server).
    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_owned();
        self
    }

    /// Enable or disable debug logging of HTTP requests and responses to stderr.
    #[must_use]
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Enable recording API responses to files in the given directory.
    #[must_use]
    pub fn with_record(mut self, dir: impl Into<PathBuf>) -> Self {
        self.record_dir = Some(dir.into());
        self
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn auth(&self, rb: RequestBuilder) -> RequestBuilder {
        rb.header("AccessKey", &self.api_key)
    }

    /// URL-encode a single path segment (e.g. a video or collection GUID).
    fn encode(id: &str) -> String {
        urlencoding::encode(id).into_owned()
    }

    /// Execute a prepared request, optionally logging method and URL to stderr.
    async fn send(&self, rb: RequestBuilder) -> Result<reqwest::Response> {
        let request = rb.build().context("failed to build request")?;
        if self.debug {
            eprintln!(">> {} {}", request.method(), request.url());
        }
        capture_request(
            &self.last_request,
            request.method().as_ref(),
            request.url().path(),
        );
        self.http
            .execute(request)
            .await
            .context("HTTP request failed")
    }

    /// Read the response body, logging status and body when debug is enabled.
    async fn read_body(
        &self,
        resp: reqwest::Response,
    ) -> Result<(reqwest::StatusCode, bytes::Bytes)> {
        let status = resp.status();
        let bytes = resp.bytes().await.context("failed to read response body")?;
        if self.debug {
            eprintln!("<< {status}");
            eprintln!("<<< {}", String::from_utf8_lossy(&bytes));
        }
        maybe_record_response(
            self.record_dir.as_deref(),
            &self.last_request,
            status.is_success(),
            &bytes,
        );
        Ok((status, bytes))
    }

    /// Deserialise a successful JSON response or surface a meaningful error.
    async fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T> {
        let (status, bytes) = self.read_body(resp).await?;
        if status.is_success() {
            let body = String::from_utf8_lossy(&bytes);
            serde_json::from_slice(&bytes)
                .with_context(|| format!("deserialising response (HTTP {status}): {body}"))
        } else {
            bail!("HTTP {status}: {}", String::from_utf8_lossy(&bytes));
        }
    }

    // -----------------------------------------------------------------------
    // Video methods
    // -----------------------------------------------------------------------

    /// List videos in a library with optional filters and pagination.
    ///
    /// Pass `None` for any optional parameter to use the API's default.
    pub async fn list_videos(
        &self,
        library_id: i64,
        page: Option<u32>,
        items_per_page: Option<u32>,
        search: Option<&str>,
        collection: Option<&str>,
        order_by: Option<&str>,
    ) -> Result<PaginatedList<Video>> {
        let url = format!("{}/library/{library_id}/videos", self.base_url);
        let mut rb = self.auth(self.http.get(&url));
        if let Some(p) = page {
            rb = rb.query(&[("page", p.to_string())]);
        }
        if let Some(n) = items_per_page {
            rb = rb.query(&[("itemsPerPage", n.to_string())]);
        }
        if let Some(s) = search {
            rb = rb.query(&[("search", s)]);
        }
        if let Some(c) = collection {
            rb = rb.query(&[("collection", c)]);
        }
        if let Some(o) = order_by {
            rb = rb.query(&[("orderBy", o)]);
        }
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    /// Fetch a single video by its GUID.
    pub async fn get_video(&self, library_id: i64, video_id: &str) -> Result<Video> {
        let vid = Self::encode(video_id);
        let url = format!("{}/library/{library_id}/videos/{vid}", self.base_url);
        let resp = self.send(self.auth(self.http.get(&url))).await?;
        self.parse_response(resp).await
    }

    /// Create a new video record (does not upload media — use [`upload_video`] for that).
    ///
    /// [`upload_video`]: StreamClient::upload_video
    pub async fn create_video(&self, library_id: i64, body: &CreateVideo) -> Result<Video> {
        let url = format!("{}/library/{library_id}/videos", self.base_url);
        let resp = self
            .send(self.auth(self.http.post(&url)).json(body))
            .await?;
        self.parse_response(resp).await
    }

    /// Update metadata on an existing video.
    pub async fn update_video(
        &self,
        library_id: i64,
        video_id: &str,
        body: &UpdateVideo,
    ) -> Result<StatusMessage> {
        let vid = Self::encode(video_id);
        let url = format!("{}/library/{library_id}/videos/{vid}", self.base_url);
        let resp = self
            .send(self.auth(self.http.post(&url)).json(body))
            .await?;
        self.parse_response(resp).await
    }

    /// Delete a video and all its associated files.
    pub async fn delete_video(&self, library_id: i64, video_id: &str) -> Result<StatusMessage> {
        let vid = Self::encode(video_id);
        let url = format!("{}/library/{library_id}/videos/{vid}", self.base_url);
        let resp = self.send(self.auth(self.http.delete(&url))).await?;
        self.parse_response(resp).await
    }

    /// Upload the raw binary content for a previously created video.
    ///
    /// `body` can be anything that converts to a [`reqwest::Body`], e.g.
    /// `Vec<u8>`, `bytes::Bytes`, or a `tokio::fs::File` wrapped with
    /// `reqwest::Body::wrap_stream`.
    ///
    /// This is the operation that Progenitor's codegen could not handle
    /// because it requires `application/octet-stream` with a binary body —
    /// a first-class advantage of the hand-written client.
    pub async fn upload_video(
        &self,
        library_id: i64,
        video_id: &str,
        body: impl Into<reqwest::Body>,
    ) -> Result<StatusMessage> {
        let vid = Self::encode(video_id);
        let url = format!("{}/library/{library_id}/videos/{vid}", self.base_url);
        let resp = self
            .send(
                self.auth(self.http.put(&url))
                    .header("Content-Type", "application/octet-stream")
                    .body(body),
            )
            .await?;
        self.parse_response(resp).await
    }

    /// Tell bunny.net to pull and ingest a video from a remote URL.
    ///
    /// Returns a `StatusMessage`; the actual video will be available
    /// asynchronously once the fetch job completes.
    pub async fn fetch_video(&self, library_id: i64, body: &FetchVideo) -> Result<StatusMessage> {
        let url = format!("{}/library/{library_id}/videos/fetch", self.base_url);
        let resp = self
            .send(self.auth(self.http.post(&url)).json(body))
            .await?;
        self.parse_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Video processing methods
    // -----------------------------------------------------------------------

    /// Trigger transcription / translation for a video.
    ///
    /// Pass `force = true` to re-transcribe a video that already has captions.
    /// `settings` is optional and overrides the library default transcribe settings.
    pub async fn transcribe_video(
        &self,
        library_id: i64,
        video_id: &str,
        force: bool,
        settings: Option<&TranscribeSettings>,
    ) -> Result<StatusMessage> {
        let vid = Self::encode(video_id);
        let url = format!(
            "{}/library/{library_id}/videos/{vid}/transcribe",
            self.base_url
        );
        let mut rb = self.auth(self.http.post(&url));
        if force {
            rb = rb.query(&[("force", "true")]);
        }
        if let Some(body) = settings {
            rb = rb.json(body);
        }
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    /// Fetch the engagement heatmap for a video.
    pub async fn get_video_heatmap(&self, library_id: i64, video_id: &str) -> Result<VideoHeatmap> {
        let vid = Self::encode(video_id);
        let url = format!(
            "{}/library/{library_id}/videos/{vid}/heatmap",
            self.base_url
        );
        let resp = self.send(self.auth(self.http.get(&url))).await?;
        self.parse_response(resp).await
    }

    /// Re-encode a video using the library's default output codecs.
    pub async fn reencode_video(&self, library_id: i64, video_id: &str) -> Result<Video> {
        let vid = Self::encode(video_id);
        let url = format!(
            "{}/library/{library_id}/videos/{vid}/reencode",
            self.base_url
        );
        let resp = self.send(self.auth(self.http.post(&url))).await?;
        self.parse_response(resp).await
    }

    /// Re-encode a video using a specific output codec.
    pub async fn reencode_video_using_codec(
        &self,
        library_id: i64,
        video_id: &str,
        codec: EncoderOutputCodec,
    ) -> Result<Video> {
        let vid = Self::encode(video_id);
        let codec_id = codec.as_int();
        let url = format!(
            "{}/library/{library_id}/videos/{vid}/outputs/{codec_id}",
            self.base_url
        );
        let resp = self.send(self.auth(self.http.put(&url))).await?;
        self.parse_response(resp).await
    }

    /// Repackage a video (re-segment HLS/DASH manifests).
    ///
    /// Pass `keep_original_files = false` to delete previous file versions
    /// after repackaging — the API default keeps them.
    pub async fn repackage_video(
        &self,
        library_id: i64,
        video_id: &str,
        keep_original_files: bool,
    ) -> Result<Video> {
        let vid = Self::encode(video_id);
        let url = format!(
            "{}/library/{library_id}/videos/{vid}/repackage",
            self.base_url
        );
        let mut rb = self.auth(self.http.post(&url));
        // The API default is true — only forward the param when the caller
        // explicitly wants to override it.
        if !keep_original_files {
            rb = rb.query(&[("keepOriginalFiles", "false")]);
        }
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    /// Trigger smart-generate (AI title/description/chapters/moments) for a video.
    pub async fn smart_generate(
        &self,
        library_id: i64,
        video_id: &str,
        settings: &SmartGenerateSettings,
    ) -> Result<StatusMessage> {
        let vid = Self::encode(video_id);
        let url = format!("{}/library/{library_id}/videos/{vid}/smart", self.base_url);
        let resp = self
            .send(self.auth(self.http.post(&url)).json(settings))
            .await?;
        self.parse_response(resp).await
    }

    /// Set the thumbnail for a video from a URL.
    ///
    /// The API accepts the thumbnail URL via the `thumbnailUrl` query string —
    /// pass `None` to drop a previously set thumbnail.
    pub async fn set_video_thumbnail(
        &self,
        library_id: i64,
        video_id: &str,
        thumbnail_url: Option<&str>,
    ) -> Result<StatusMessage> {
        let vid = Self::encode(video_id);
        let url = format!(
            "{}/library/{library_id}/videos/{vid}/thumbnail",
            self.base_url
        );
        let mut rb = self.auth(self.http.post(&url));
        if let Some(u) = thumbnail_url {
            rb = rb.query(&[("thumbnailUrl", u)]);
        }
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    /// Fetch the resolutions / encodings info for a video.
    pub async fn get_video_resolutions(
        &self,
        library_id: i64,
        video_id: &str,
    ) -> Result<StatusEnvelope<VideoResolutionsInfo>> {
        let vid = Self::encode(video_id);
        let url = format!(
            "{}/library/{library_id}/videos/{vid}/resolutions",
            self.base_url
        );
        let resp = self.send(self.auth(self.http.get(&url))).await?;
        self.parse_response(resp).await
    }

    /// Cleanup video resolutions — delete one or more renditions, optionally
    /// running in dry-run mode to preview the change.
    pub async fn cleanup_video_resolutions(
        &self,
        library_id: i64,
        video_id: &str,
        opts: &StreamCleanupResolutions<'_>,
    ) -> Result<StatusMessage> {
        let vid = Self::encode(video_id);
        let url = format!(
            "{}/library/{library_id}/videos/{vid}/resolutions/cleanup",
            self.base_url
        );
        let mut rb = self.auth(self.http.post(&url));
        if let Some(v) = opts.resolutions_to_delete {
            rb = rb.query(&[("resolutionsToDelete", v)]);
        }
        if opts.delete_non_configured_resolutions {
            rb = rb.query(&[("deleteNonConfiguredResolutions", "true")]);
        }
        if opts.delete_original {
            rb = rb.query(&[("deleteOriginal", "true")]);
        }
        if opts.delete_mp4_files {
            rb = rb.query(&[("deleteMp4Files", "true")]);
        }
        if opts.dry_run {
            rb = rb.query(&[("dryRun", "true")]);
        }
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    /// Fetch the storage size breakdown for a video.
    pub async fn get_video_storage_size(
        &self,
        library_id: i64,
        video_id: &str,
    ) -> Result<StatusEnvelope<VideoStorageSize>> {
        let vid = Self::encode(video_id);
        let url = format!(
            "{}/library/{library_id}/videos/{vid}/storage",
            self.base_url
        );
        let resp = self.send(self.auth(self.http.get(&url))).await?;
        self.parse_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Collection methods
    // -----------------------------------------------------------------------

    /// List collections in a library with optional filters and pagination.
    pub async fn list_collections(
        &self,
        library_id: i64,
        page: Option<u32>,
        items_per_page: Option<u32>,
        search: Option<&str>,
        order_by: Option<&str>,
    ) -> Result<PaginatedList<Collection>> {
        let url = format!("{}/library/{library_id}/collections", self.base_url);
        let mut rb = self.auth(self.http.get(&url));
        if let Some(p) = page {
            rb = rb.query(&[("page", p.to_string())]);
        }
        if let Some(n) = items_per_page {
            rb = rb.query(&[("itemsPerPage", n.to_string())]);
        }
        if let Some(s) = search {
            rb = rb.query(&[("search", s)]);
        }
        if let Some(o) = order_by {
            rb = rb.query(&[("orderBy", o)]);
        }
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    /// Fetch a single collection by its GUID.
    pub async fn get_collection(&self, library_id: i64, collection_id: &str) -> Result<Collection> {
        let col = Self::encode(collection_id);
        let url = format!("{}/library/{library_id}/collections/{col}", self.base_url);
        let resp = self.send(self.auth(self.http.get(&url))).await?;
        self.parse_response(resp).await
    }

    /// Create a new collection in a library.
    pub async fn create_collection(
        &self,
        library_id: i64,
        body: &CreateCollection,
    ) -> Result<Collection> {
        let url = format!("{}/library/{library_id}/collections", self.base_url);
        let resp = self
            .send(self.auth(self.http.post(&url)).json(body))
            .await?;
        self.parse_response(resp).await
    }

    /// Update an existing collection.
    pub async fn update_collection(
        &self,
        library_id: i64,
        collection_id: &str,
        body: &UpdateCollection,
    ) -> Result<Collection> {
        let col = Self::encode(collection_id);
        let url = format!("{}/library/{library_id}/collections/{col}", self.base_url);
        let resp = self
            .send(self.auth(self.http.post(&url)).json(body))
            .await?;
        self.parse_response(resp).await
    }

    /// Delete a collection.
    pub async fn delete_collection(
        &self,
        library_id: i64,
        collection_id: &str,
    ) -> Result<StatusMessage> {
        let col = Self::encode(collection_id);
        let url = format!("{}/library/{library_id}/collections/{col}", self.base_url);
        let resp = self.send(self.auth(self.http.delete(&url))).await?;
        self.parse_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Caption methods
    // -----------------------------------------------------------------------

    /// Add a caption track to a video.
    ///
    /// `srclang` is the BCP 47 language code (e.g. "en", "de", "fr").
    /// `captions_file` is the SRT subtitle content.
    pub async fn add_caption(
        &self,
        library_id: i64,
        video_id: &str,
        srclang: &str,
        captions_file: &str,
    ) -> Result<StatusMessage> {
        let vid = Self::encode(video_id);
        let lang = Self::encode(srclang);
        let url = format!(
            "{}/library/{library_id}/videos/{vid}/captions/{lang}",
            self.base_url
        );
        let rb = self
            .auth(self.http.post(&url))
            .json(&serde_json::json!({ "CaptionsFile": captions_file }));
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    /// Delete a caption track from a video.
    pub async fn delete_caption(
        &self,
        library_id: i64,
        video_id: &str,
        srclang: &str,
    ) -> Result<StatusMessage> {
        let vid = Self::encode(video_id);
        let lang = Self::encode(srclang);
        let url = format!(
            "{}/library/{library_id}/videos/{vid}/captions/{lang}",
            self.base_url
        );
        let resp = self.send(self.auth(self.http.delete(&url))).await?;
        self.parse_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Statistics methods
    // -----------------------------------------------------------------------

    /// Fetch statistics for a video library.
    pub async fn get_library_statistics(
        &self,
        library_id: i64,
        date_from: Option<&str>,
        date_to: Option<&str>,
        hourly: bool,
        video_guid: Option<&str>,
    ) -> Result<VideoStatistics> {
        let url = format!("{}/library/{library_id}/statistics", self.base_url);
        let mut rb = self.auth(self.http.get(&url));
        if let Some(v) = date_from {
            rb = rb.query(&[("dateFrom", v)]);
        }
        if let Some(v) = date_to {
            rb = rb.query(&[("dateTo", v)]);
        }
        if hourly {
            rb = rb.query(&[("hourly", "true")]);
        }
        if let Some(v) = video_guid {
            rb = rb.query(&[("videoGuid", v)]);
        }
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::VideoStatus;

    /// Helper: assert that a JSON string deserialises correctly.
    fn deser<T: serde::de::DeserializeOwned>(json: &str) -> T {
        serde_json::from_str(json).expect("deserialise failed")
    }

    #[test]
    fn video_status_roundtrip() {
        // Integer 4 should map to Finished
        let status: VideoStatus = serde_json::from_str("4").unwrap();
        assert_eq!(status, VideoStatus::Finished);
        let serialised = serde_json::to_string(&VideoStatus::Error).unwrap();
        assert_eq!(serialised, "5");
    }

    #[test]
    fn video_deserialises_minimal_response() {
        let json = r#"{
            "VideoLibraryId": 123,
            "Guid": "abc-def",
            "Title": "Test Video",
            "DateUploaded": "2024-01-01T00:00:00Z",
            "Views": 42,
            "IsPublic": true,
            "Length": 120,
            "Status": 4,
            "Framerate": 30.0,
            "Width": 1920,
            "Height": 1080,
            "OutputCodecs": "x264",
            "ThumbnailCount": 1,
            "EncodeProgress": 100,
            "StorageSize": 1000000,
            "HasMP4Fallback": false,
            "AverageWatchTime": 60,
            "TotalWatchTime": 2520
        }"#;
        let video: Video = deser(json);
        assert_eq!(video.guid, "abc-def");
        assert_eq!(video.status, VideoStatus::Finished);
        assert_eq!(video.width, 1920);
        assert!(video.captions.is_empty());
    }

    #[test]
    fn video_deserialises_with_captions() {
        let json = r#"{
            "VideoLibraryId": 1,
            "Guid": "g1",
            "Title": "T",
            "DateUploaded": "2024-01-01T00:00:00Z",
            "Views": 0,
            "IsPublic": false,
            "Length": 0,
            "Status": 4,
            "Framerate": 24.0,
            "Width": 1280,
            "Height": 720,
            "OutputCodecs": "x264",
            "ThumbnailCount": 0,
            "EncodeProgress": 100,
            "StorageSize": 0,
            "HasMP4Fallback": false,
            "AverageWatchTime": 0,
            "TotalWatchTime": 0,
            "Captions": [
                {"Srclang": "en", "Label": "English"},
                {"Srclang": "fr", "Label": "French"}
            ]
        }"#;
        let video: Video = deser(json);
        assert_eq!(video.captions.len(), 2);
        assert_eq!(video.captions[0].srclang, "en");
        assert_eq!(video.captions[1].label, "French");
    }

    #[test]
    fn paginated_list_deserialises() {
        let json = r#"{
            "TotalItems": 2,
            "CurrentPage": 1,
            "ItemsPerPage": 100,
            "Items": []
        }"#;
        let list: PaginatedList<Video> = deser(json);
        assert_eq!(list.total_items, 2);
        assert_eq!(list.current_page, 1);
        assert!(list.items.is_empty());
    }

    #[test]
    fn status_message_deserialises() {
        let json = r#"{"Success": true, "Message": "OK", "StatusCode": 200}"#;
        let msg: StatusMessage = deser(json);
        assert!(msg.success);
        assert_eq!(msg.status_code, 200);
    }

    #[test]
    fn create_video_serialises_required_only() {
        let body = CreateVideo::new("My Video");
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["Title"], "My Video");
        assert!(json.get("CollectionId").is_none());
        assert!(json.get("ThumbnailTime").is_none());
    }

    #[test]
    fn create_video_serialises_all_fields() {
        let body = CreateVideo::new("My Video")
            .collection_id("col-123")
            .thumbnail_time(5000);
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["Title"], "My Video");
        assert_eq!(json["CollectionId"], "col-123");
        assert_eq!(json["ThumbnailTime"], 5000);
    }

    #[test]
    fn update_video_skips_none_fields() {
        let body = UpdateVideo::new().title("New Title");
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["Title"], "New Title");
        // CollectionId was not set, must be absent (not null)
        assert!(json.get("CollectionId").is_none());
    }

    #[test]
    fn fetch_video_headers_builder() {
        let body = FetchVideo::new("https://example.com/video.mp4")
            .header("Authorization", "Bearer token")
            .title("Remote Video");
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["Url"], "https://example.com/video.mp4");
        assert_eq!(json["Title"], "Remote Video");
        assert_eq!(json["Headers"]["Authorization"], "Bearer token");
    }

    #[test]
    fn collection_deserialises() {
        let json = r#"{
            "VideoLibraryId": 99,
            "Guid": "col-guid",
            "Name": "My Collection",
            "VideoCount": 7,
            "TotalSize": 500000
        }"#;
        let col: Collection = deser(json);
        assert_eq!(col.video_count, 7);
        assert_eq!(col.name.as_deref(), Some("My Collection"));
    }

    #[test]
    fn stream_client_constructs() {
        let client = StreamClient::new("test-key");
        assert_eq!(client.api_key, "test-key");
        assert_eq!(client.base_url, BASE_URL);
        assert!(!client.debug);
    }

    #[test]
    fn stream_client_with_base_url() {
        let client = StreamClient::new("key").with_base_url("http://localhost:8080");
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    fn stream_client_with_debug() {
        let client = StreamClient::new("key").with_debug(true);
        assert!(client.debug);
    }

    #[test]
    fn encode_plain_guid_is_unchanged() {
        assert_eq!(StreamClient::encode("abc-def-123"), "abc-def-123");
    }

    #[test]
    fn encode_special_chars_in_id() {
        // A `?` must be percent-encoded so it cannot break the URL as a query separator.
        assert_eq!(StreamClient::encode("id?foo"), "id%3Ffoo");
    }

    #[test]
    fn encode_space_in_id() {
        assert_eq!(StreamClient::encode("my id"), "my%20id");
    }
}
