use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Processing status of a video in the bunny.net Stream pipeline.
///
/// Integer values match the bunny.net API wire format:
/// `0=Created, 1=Uploaded, 2=Processing, 3=Transcoding, 4=Finished,
///  5=Error, 6=UploadFailed, 7=JitSegmenting, 8=JitPlaylistsCreated`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum VideoStatus {
    Created = 0,
    Uploaded = 1,
    Processing = 2,
    Transcoding = 3,
    Finished = 4,
    Error = 5,
    UploadFailed = 6,
    JitSegmenting = 7,
    JitPlaylistsCreated = 8,
}

impl std::fmt::Display for VideoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoStatus::Created => write!(f, "Created"),
            VideoStatus::Uploaded => write!(f, "Uploaded"),
            VideoStatus::Processing => write!(f, "Processing"),
            VideoStatus::Transcoding => write!(f, "Transcoding"),
            VideoStatus::Finished => write!(f, "Finished"),
            VideoStatus::Error => write!(f, "Error"),
            VideoStatus::UploadFailed => write!(f, "UploadFailed"),
            VideoStatus::JitSegmenting => write!(f, "JitSegmenting"),
            VideoStatus::JitPlaylistsCreated => write!(f, "JitPlaylistsCreated"),
        }
    }
}

// ---------------------------------------------------------------------------
// Response models
// ---------------------------------------------------------------------------

/// A caption track attached to a video.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Caption {
    /// ISO 639-1 language code (e.g. `"en"`, `"fr"`).
    #[serde(default)]
    pub srclang: String,
    /// Human-readable label shown in the player (e.g. `"English"`).
    #[serde(default)]
    pub label: String,
}

/// A single video returned by the bunny.net Stream API.
///
/// Fields that the API may omit on older records are annotated with
/// `#[serde(default)]` so they deserialise to their `Default` value
/// instead of failing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Video {
    /// The ID of the library that owns this video.
    pub video_library_id: i64,
    /// Unique GUID of the video.
    pub guid: String,
    /// Human-readable title.
    pub title: String,
    /// ISO 8601 upload timestamp.
    pub date_uploaded: String,
    /// Total view count.
    pub views: i64,
    /// Whether the video is publicly accessible without a token.
    pub is_public: bool,
    /// Duration in seconds.
    pub length: i32,
    /// Current processing status.
    pub status: VideoStatus,
    /// Frame rate of the original upload.
    pub framerate: f64,
    /// Width of the original file in pixels.
    pub width: i32,
    /// Height of the original file in pixels.
    pub height: i32,
    /// Encoded output codecs string (e.g. `"x264,vp9"`).
    #[serde(default)]
    pub output_codecs: String,
    /// Encoding progress percentage (0–100).
    #[serde(default)]
    pub encode_progress: i32,
    /// Storage used by this video in bytes.
    pub storage_size: i64,
    /// Whether MP4 fallback files have been generated.
    #[serde(rename = "HasMP4Fallback")]
    pub has_mp4_fallback: bool,
    /// Average viewer watch time in seconds.
    pub average_watch_time: i64,
    /// Total accumulated watch time in seconds across all viewers.
    pub total_watch_time: i64,

    // Optional / nullable fields
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub collection_id: Option<String>,
    #[serde(default)]
    pub thumbnail_file_name: Option<String>,
    #[serde(default)]
    pub thumbnail_blurhash: Option<String>,
    #[serde(default)]
    pub available_resolutions: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub captions: Vec<Caption>,
    #[serde(default)]
    pub rotation: Option<i32>,
    #[serde(default)]
    pub has_original: Option<bool>,
    #[serde(default)]
    pub original_hash: Option<String>,
}

/// A video collection returned by the bunny.net Stream API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Collection {
    /// The ID of the library that owns this collection.
    #[serde(default, alias = "videoLibraryId")]
    pub video_library_id: i64,
    /// Unique GUID of the collection.
    #[serde(default, alias = "guid")]
    pub guid: Option<String>,
    /// Human-readable name.
    #[serde(default, alias = "name")]
    pub name: Option<String>,
    /// Number of videos in the collection.
    #[serde(default, alias = "videoCount")]
    pub video_count: i64,
    /// Total storage size of all videos in the collection in bytes.
    #[serde(default, alias = "totalSize")]
    pub total_size: i64,
    /// Comma-separated video IDs used as preview thumbnails.
    #[serde(default, alias = "previewVideoIds")]
    pub preview_video_ids: Option<String>,
    /// URLs of preview images for videos in the collection.
    #[serde(default, alias = "previewImageUrls")]
    pub preview_image_urls: Vec<String>,
}

/// Generic paginated list response from the bunny.net Stream API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PaginatedList<T> {
    /// The result items on the current page.
    ///
    /// The bunny.net API marks this field nullable in its schema but always
    /// returns an array. We use a dedicated default fn so serde does not
    /// require `T: Default`.
    #[serde(default = "Vec::new", alias = "items")]
    pub items: Vec<T>,
    /// The current page number.
    #[serde(alias = "currentPage")]
    pub current_page: i64,
    /// The total number of items across all pages.
    #[serde(alias = "totalItems")]
    pub total_items: i64,
    /// Number of items returned per page.
    #[serde(default, alias = "itemsPerPage")]
    pub items_per_page: i32,
}

/// Generic status / acknowledgement response from the bunny.net Stream API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatusMessage {
    /// Whether the request succeeded.
    #[serde(alias = "success")]
    pub success: bool,
    /// Human-readable description of the outcome.
    #[serde(default, alias = "message")]
    pub message: Option<String>,
    /// HTTP-aligned status code echoed in the body.
    #[serde(default, alias = "statusCode")]
    pub status_code: i32,
}

// ---------------------------------------------------------------------------
// Request bodies
// ---------------------------------------------------------------------------

/// Request body for `POST /library/{id}/videos` — create a new video record.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateVideo {
    /// Title of the video (required).
    pub title: String,
    /// Optional collection to assign the video to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<String>,
    /// Optional timestamp (ms) used to extract the thumbnail frame.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_time: Option<i32>,
}

impl CreateVideo {
    /// Minimal constructor — only `title` is required by the API.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            collection_id: None,
            thumbnail_time: None,
        }
    }

    pub fn collection_id(mut self, id: impl Into<String>) -> Self {
        self.collection_id = Some(id.into());
        self
    }

    pub fn thumbnail_time(mut self, ms: i32) -> Self {
        self.thumbnail_time = Some(ms);
        self
    }
}

/// Per-upload encoding and AI-generation options for
/// `PUT /library/{libraryId}/videos/{videoId}`.
///
/// Every field is optional; `None` (or `false`) leaves the corresponding
/// query parameter off the request so the library default applies. The
/// resolution / codec / language fields are sent verbatim, so callers pass
/// the API's comma-separated form (e.g. `"720p,1080p"`).
#[derive(Debug, Clone, Default)]
pub struct VideoUploadOptions {
    /// Enable Just-In-Time encoding for this upload.
    pub jit_enabled: Option<bool>,
    /// Comma-separated list of resolutions to encode (e.g. `"720p,1080p"`).
    pub enabled_resolutions: Option<String>,
    /// Comma-separated list of output codecs (e.g. `"x264,vp9"`).
    pub enabled_output_codecs: Option<String>,
    /// Enable automatic transcription for this upload.
    pub transcribe_enabled: Option<bool>,
    /// Comma-separated list of transcription target languages (ISO 639-1).
    pub transcribe_languages: Option<String>,
    /// Source language of the video (ISO 639-1, e.g. `"en"`).
    pub source_language: Option<String>,
    /// Have the API generate a title from the video content.
    pub generate_title: Option<bool>,
    /// Have the API generate a description from the video content.
    pub generate_description: Option<bool>,
    /// Have the API generate chapters from the video content.
    pub generate_chapters: Option<bool>,
    /// Have the API generate highlight moments from the video content.
    pub generate_moments: Option<bool>,
}

/// Request body for `POST /library/{id}/videos/{videoId}` — update a video.
///
/// All fields are optional; only non-`None` values are serialised.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateVideo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<String>,
}

impl UpdateVideo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn collection_id(mut self, id: impl Into<String>) -> Self {
        self.collection_id = Some(id.into());
        self
    }
}

/// Request body for `POST /library/{id}/collections` — create a new collection.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateCollection {
    /// Name of the collection (required).
    pub name: String,
}

impl CreateCollection {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Request body for `POST /library/{id}/collections/{collectionId}` — update a collection.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateCollection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl UpdateCollection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Request body for `POST /library/{id}/videos/fetch` — pull a video from a URL.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct FetchVideo {
    /// Public URL to pull the video from (required).
    pub url: String,
    /// Optional HTTP headers to forward with the fetch request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    /// Optional title override for the created video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl FetchVideo {
    /// Construct a fetch request pointing at `url`.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            headers: None,
            title: None,
        }
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Video processing — request bodies and response types
// ---------------------------------------------------------------------------

/// Output codec used by `PUT /library/{id}/videos/{videoId}/outputs/{outputCodecId}`.
///
/// Wire format is the integer value (`0..=3`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum EncoderOutputCodec {
    X264 = 0,
    Vp9 = 1,
    Hevc = 2,
    Av1 = 3,
}

impl EncoderOutputCodec {
    pub fn as_int(self) -> u8 {
        self as u8
    }

    /// Parse from the user-friendly name (case-insensitive) or the integer value.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "x264" | "0" => Some(Self::X264),
            "vp9" | "1" => Some(Self::Vp9),
            "hevc" | "2" => Some(Self::Hevc),
            "av1" | "3" => Some(Self::Av1),
            _ => None,
        }
    }
}

impl std::fmt::Display for EncoderOutputCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::X264 => write!(f, "x264"),
            Self::Vp9 => write!(f, "vp9"),
            Self::Hevc => write!(f, "hevc"),
            Self::Av1 => write!(f, "av1"),
        }
    }
}

/// Request body for `POST /library/{id}/videos/{videoId}/transcribe`.
///
/// All fields are optional; only non-`None` values are serialised.
///
/// Unlike the older Stream request bodies (`CreateVideo`, `UpdateVideo`,
/// `FetchVideo`) which use `PascalCase`, the transcribe endpoint expects
/// `camelCase` keys. The wire format is asserted in the wiremock tests.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscribeSettings {
    /// Target languages (ISO 639-1 codes) for translation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_languages: Option<Vec<String>>,
    /// Whether the video title should be auto-generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_title: Option<bool>,
    /// Whether the video description should be auto-generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_description: Option<bool>,
    /// Whether video chapters should be auto-generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_chapters: Option<bool>,
    /// Whether video moments should be auto-generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_moments: Option<bool>,
    /// Source language (ISO 639-1 code).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_language: Option<String>,
}

impl TranscribeSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn target_languages<I, S>(mut self, langs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.target_languages = Some(langs.into_iter().map(Into::into).collect());
        self
    }

    pub fn source_language(mut self, lang: impl Into<String>) -> Self {
        self.source_language = Some(lang.into());
        self
    }

    pub fn generate_title(mut self, v: bool) -> Self {
        self.generate_title = Some(v);
        self
    }

    pub fn generate_description(mut self, v: bool) -> Self {
        self.generate_description = Some(v);
        self
    }

    pub fn generate_chapters(mut self, v: bool) -> Self {
        self.generate_chapters = Some(v);
        self
    }

    pub fn generate_moments(mut self, v: bool) -> Self {
        self.generate_moments = Some(v);
        self
    }
}

/// Request body for `POST /library/{id}/videos/{videoId}/smart`.
///
/// Like [`TranscribeSettings`], the smart-generate endpoint expects
/// `camelCase` keys (see that type's doc for context).
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartGenerateSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_title: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_description: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_chapters: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_moments: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_language: Option<String>,
}

impl SmartGenerateSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn source_language(mut self, lang: impl Into<String>) -> Self {
        self.source_language = Some(lang.into());
        self
    }

    pub fn generate_title(mut self, v: bool) -> Self {
        self.generate_title = Some(v);
        self
    }

    pub fn generate_description(mut self, v: bool) -> Self {
        self.generate_description = Some(v);
        self
    }

    pub fn generate_chapters(mut self, v: bool) -> Self {
        self.generate_chapters = Some(v);
        self
    }

    pub fn generate_moments(mut self, v: bool) -> Self {
        self.generate_moments = Some(v);
        self
    }
}

/// Heatmap response from `GET /library/{id}/videos/{videoId}/heatmap`.
///
/// `heatmap` maps a segment-index string (e.g. `"0"`, `"1"`, …) to a
/// normalised intensity (0–100). Missing segments imply 0.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoHeatmap {
    #[serde(default)]
    pub heatmap: Option<BTreeMap<String, i32>>,
}

/// Resolution + storage path pair returned inside [`VideoResolutionsInfo`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionReference {
    #[serde(default)]
    pub resolution: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
}

/// Stored object descriptor returned inside [`VideoResolutionsInfo`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageObject {
    #[serde(default)]
    pub guid: Option<String>,
    #[serde(default)]
    pub storage_zone_name: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub object_name: Option<String>,
    #[serde(default)]
    pub length: i64,
    #[serde(default)]
    pub last_changed: Option<String>,
    #[serde(default)]
    pub server_id: i32,
    #[serde(default)]
    pub is_directory: bool,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub date_created: Option<String>,
    #[serde(default)]
    pub storage_zone_id: i64,
    #[serde(default)]
    pub checksum: Option<String>,
    #[serde(default)]
    pub replicated_zones: Option<String>,
}

/// Video resolutions info returned inside the `data` field of the response
/// to `GET /library/{id}/videos/{videoId}/resolutions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoResolutionsInfo {
    #[serde(default)]
    pub video_id: Option<String>,
    #[serde(default)]
    pub video_library_id: i64,
    #[serde(default)]
    pub available_resolutions: Vec<String>,
    #[serde(default)]
    pub configured_resolutions: Vec<String>,
    #[serde(default)]
    pub playlist_resolutions: Vec<ResolutionReference>,
    #[serde(default)]
    pub storage_resolutions: Vec<ResolutionReference>,
    #[serde(default)]
    pub mp4_resolutions: Vec<ResolutionReference>,
    #[serde(default)]
    pub storage_objects: Vec<StorageObject>,
    #[serde(default)]
    pub old_resolutions: Vec<StorageObject>,
    #[serde(default)]
    pub has_both_old_and_new_resolution_format: bool,
    #[serde(default)]
    pub has_original: bool,
}

/// Codec + resolution + size triple inside [`VideoStorageSize::encoded`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodecRenditionSize {
    #[serde(default)]
    pub codec: Option<String>,
    #[serde(default)]
    pub resolution: Option<String>,
    #[serde(default)]
    pub size: i64,
}

/// Video storage size info returned inside the `data` field of the response
/// to `GET /library/{id}/videos/{videoId}/storage`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoStorageSize {
    /// Map of `"<codec> <resolution>"` (or whatever key the API picks) to size info.
    #[serde(default)]
    pub encoded: Option<HashMap<String, CodecRenditionSize>>,
    #[serde(default)]
    pub thumbnails: i64,
    #[serde(default)]
    pub previews: i64,
    #[serde(default)]
    pub originals: i64,
    #[serde(default, alias = "mp4Fallback")]
    pub mp4_fallback: i64,
    #[serde(default)]
    pub miscellaneous: i64,
    #[serde(default)]
    pub calculated_at: Option<String>,
}

/// Generic envelope used by responses that wrap a `data` payload alongside
/// the standard [`StatusMessage`] fields (e.g. resolutions and storage info).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusEnvelope<T> {
    #[serde(default, alias = "Success", alias = "success")]
    pub success: bool,
    #[serde(default, alias = "Message", alias = "message")]
    pub message: Option<String>,
    #[serde(default, alias = "StatusCode", alias = "statusCode")]
    pub status_code: i32,
    #[serde(default = "Option::default")]
    pub data: Option<T>,
}

// ---------------------------------------------------------------------------
// Statistics types
// ---------------------------------------------------------------------------

/// Video library statistics returned by `GET /library/{libraryId}/statistics`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoStatistics {
    pub views_chart: Option<BTreeMap<String, i64>>,
    pub watch_time_chart: Option<BTreeMap<String, i64>>,
    pub country_view_counts: Option<BTreeMap<String, i64>>,
    pub country_watch_time: Option<BTreeMap<String, i64>>,
    #[serde(default)]
    pub engagement_score: i64,
}
