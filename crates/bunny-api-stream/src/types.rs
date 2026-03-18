use std::collections::HashMap;

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
    #[serde(default)]
    pub video_library_id: i64,
    /// Unique GUID of the collection.
    #[serde(default)]
    pub guid: Option<String>,
    /// Human-readable name.
    #[serde(default)]
    pub name: Option<String>,
    /// Number of videos in the collection.
    #[serde(default)]
    pub video_count: i64,
    /// Total storage size of all videos in the collection in bytes.
    #[serde(default)]
    pub total_size: i64,
    /// Comma-separated video IDs used as preview thumbnails.
    #[serde(default)]
    pub preview_video_ids: Option<String>,
    /// URLs of preview images for videos in the collection.
    #[serde(default)]
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
    #[serde(default = "Vec::new")]
    pub items: Vec<T>,
    /// The current page number.
    pub current_page: i64,
    /// The total number of items across all pages.
    pub total_items: i64,
    /// Number of items returned per page.
    #[serde(default)]
    pub items_per_page: i32,
}

/// Generic status / acknowledgement response from the bunny.net Stream API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatusMessage {
    /// Whether the request succeeded.
    pub success: bool,
    /// Human-readable description of the outcome.
    #[serde(default)]
    pub message: Option<String>,
    /// HTTP-aligned status code echoed in the body.
    #[serde(default)]
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
