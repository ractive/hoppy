---
title: "Bunny.net Stream (Video) API Research"
date: 2026-03-18
tags:
  - bunny-net
  - api
  - stream
  - video
status: research-complete
---

# Bunny.net Stream (Video) API Research

## Overview

The Stream API is split across **two different APIs**:

| Concern | API | Base URL |
|---------|-----|----------|
| Video library CRUD | **Core API** | `https://api.bunny.net` |
| Video/collection management, upload, playback | **Stream API** | `https://video.bunnycdn.com` |

This is an important architectural detail: video libraries are managed through the Core API (same as pull zones, DNS, etc.), while videos within those libraries are managed through the dedicated Stream API.

## Authentication

### Core API (video libraries)

- **Header:** `AccessKey`
- **Key:** Account-level API key (same as pull zones, DNS, etc.)
- **Env var in hoppy:** `BUNNY_API_KEY`

### Stream API (videos, collections)

- **Header:** `AccessKey`
- **Key:** Per-library API key (found in video library settings, or from `ApiKey` field in the VideoLibraryModel response)
- **Env var in hoppy:** `BUNNY_STREAM_KEY` (or derive from library via Core API)

This mirrors the Storage API pattern: account key for zone-level management, per-zone key for content operations.

---

## Video Library Endpoints (Core API — `https://api.bunny.net`)

### List Video Libraries

```
GET /videolibrary?page={page}&perPage={perPage}&search={search}
```

**Query Parameters:**

| Parameter | Type | Default | Range | Description |
|-----------|------|---------|-------|-------------|
| page | int32 | 0 | 1–2147483647 | Page number |
| perPage | int32 | 1000 | 5–1000 | Items per page |
| search | string | — | — | Filter by name |

**Response (200):** Paginated envelope (same as pull zones):

```json
{
  "Items": [VideoLibraryModel, ...],
  "CurrentPage": 1,
  "TotalItems": 42,
  "HasMoreItems": false
}
```

**QUIRK:** Same pagination quirk as other Core API list endpoints — without `page`/`perPage` params, likely returns a bare array. Always send pagination params.

### Get Video Library

```
GET /videolibrary/{id}
```

**Path Parameters:** `id` (int64, required)

**Response (200):** `VideoLibraryModel`

### Create Video Library

```
POST /videolibrary
```

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| Name | string | Yes (min 1 char) | Library name |
| ReplicationRegions | string[] | No | Geo-replication regions |
| PlayerVersion | int32 | No | Player version |

**Response (200):** `VideoLibraryModel`
**Error (400):** `ApiErrorData` with ErrorKey, Field, Message

### Update Video Library

```
POST /videolibrary/{id}
```

**Path Parameters:** `id` (int64, required)

**Request Body (VideoLibraryUpdateModel) — all fields optional:**

| Field | Type | Description |
|-------|------|-------------|
| Name | string | Library name |
| CustomHTML | string | Custom HTML for player |
| PlayerKeyColor | string | Player key color |
| UILanguage | string | UI language |
| CaptionsFontColor | string | Caption font color |
| CaptionsBackground | string | Caption background |
| CaptionsFontSize | int32 | Caption font size |
| ViAiPublisherId | string | Vi.ai publisher ID |
| VastTagUrl | string | VAST tag URL |
| WebhookUrl | string | Webhook URL |
| Controls | string | Player controls config |
| PlaybackSpeeds | string | Available playback speeds |
| EnabledResolutions | string | Enabled encoding resolutions |
| FontFamily | string | Player font family |
| OutputCodecs | string | Output codec config |
| EnableTokenAuthentication | bool | Token auth |
| EnableTokenIPVerification | bool | Token IP verification |
| ResetToken | bool | Reset auth token |
| AllowEarlyPlay | bool | Allow early playback |
| PlayerTokenAuthenticationEnabled | bool | Player token auth |
| BlockNoneReferrer | bool | Block empty referrer |
| EnableMP4Fallback | bool | MP4 fallback |
| KeepOriginalFiles | bool | Keep originals |
| AllowDirectPlay | bool | Direct play |
| EnableDRM | bool | DRM protection |
| ShowHeatmap | bool | Show heatmap |
| EnableContentTagging | bool | Content tagging |
| EnableTranscribing | bool | Transcription |
| EnableTranscribingTitleGeneration | bool | Auto-generate titles |
| EnableTranscribingDescriptionGeneration | bool | Auto-generate descriptions |
| EnableTranscribingChaptersGeneration | bool | Auto-generate chapters |
| EnableTranscribingMomentsGeneration | bool | Auto-generate moments |
| EnableCaptionsInPlaylist | bool | Captions in playlist |
| RememberPlayerPosition | bool | Remember position |
| EnableMultiAudioTrackSupport | bool | Multi-audio support |
| UseSeparateAudioStream | bool | Separate audio stream |
| JitEncodingEnabled | bool | JIT encoding |
| RemoveMetadataFromFallbackVideos | bool | Strip metadata from MP4 |
| ScaleVideoUsingBothDimensions | bool | Scale using both dims |
| ExposeOriginals | bool | Expose original files |
| ExposeVideoMetadata | bool | Expose video metadata |
| WatermarkPositionLeft | int32 | Watermark X position |
| WatermarkPositionTop | int32 | Watermark Y position |
| WatermarkWidth | int32 | Watermark width |
| WatermarkHeight | int32 | Watermark height |
| PlayerVersion | int32 | Player version |
| Bitrate240p | int32 | Bitrate for 240p |
| Bitrate360p | int32 | Bitrate for 360p |
| Bitrate480p | int32 | Bitrate for 480p |
| Bitrate720p | int32 | Bitrate for 720p |
| Bitrate1080p | int32 | Bitrate for 1080p |
| Bitrate1440p | int32 | Bitrate for 1440p |
| Bitrate2160p | int32 | Bitrate for 2160p |
| DrmVersion | int32 | DRM version |
| EncodingTier | int32 | Encoding tier |
| AppleFairPlayDrm | object | Apple FairPlay DRM config |
| GoogleWidevineDrm | object | Google Widevine DRM config |
| TranscribingCaptionLanguages | string[] | Caption languages for transcription |

**Response (204):** `VideoLibraryModel`

**QUIRK:** Uses POST, not PATCH — same pattern as pull zone updates.

### Delete Video Library

```
DELETE /videolibrary/{id}
```

**Path Parameters:** `id` (int64, required)

**Response (204):** `VideoLibraryModel`

---

## VideoLibraryModel (response schema)

Key fields from Core API responses:

| Field | Type | Description |
|-------|------|-------------|
| Id | int64 | Library ID |
| Name | string | Library name |
| VideoCount | int64 | Number of videos |
| TrafficUsage | int64 | Monthly traffic bytes |
| StorageUsage | int64 | Total storage bytes |
| DateCreated | date-time | Creation date |
| DateModified | date-time | Last modified |
| ReplicationRegions | string[] | Geo-replication regions |
| ApiKey | string | **Stream API key for this library** |
| ReadOnlyApiKey | string | Read-only API key |
| HasWatermark | bool | Watermark enabled |
| PullZoneId | int64 | Associated pull zone |
| StorageZoneId | int64 | Associated storage zone |
| PullZoneType | enum | Premium or Volume |
| EnabledResolutions | string | Enabled resolutions |
| OutputCodecs | string | Output codecs |
| EnableDRM | bool | DRM enabled |
| EnableMP4Fallback | bool | MP4 fallback |
| KeepOriginalFiles | bool | Keep originals |
| PlayerTokenAuthenticationEnabled | bool | Token auth |
| AllowedReferrers | string[] | Allowed referrers |
| BlockedReferrers | string[] | Blocked referrers |
| JitEncodingEnabled | bool | JIT encoding |
| EnableTranscribing | bool | Transcription enabled |
| (many more player/encoding/DRM fields) | | |

**IMPORTANT:** The `ApiKey` field in this response is the Stream API key needed to manage videos in this library.

---

## Video Endpoints (Stream API — `https://video.bunnycdn.com`)

All endpoints below require the **per-library API key** in the `AccessKey` header.

### List Videos

```
GET /library/{libraryId}/videos?page={page}&itemsPerPage={itemsPerPage}&search={search}&collection={collection}&orderBy={orderBy}
```

**Path Parameters:** `libraryId` (int64, required)

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| page | int32 | 1 | Page number |
| itemsPerPage | int32 | 100 | Items per page |
| search | string | — | Search filter |
| collection | string | — | Filter by collection ID |
| orderBy | string | "date" | Sort order |

**Response (200):** `PaginationListOfVideoModel`

```json
{
  "TotalItems": 150,
  "CurrentPage": 1,
  "ItemsPerPage": 100,
  "Items": [VideoModel, ...]
}
```

**QUIRK — Different pagination format from Core API:**
- Both APIs use PascalCase (despite the OpenAPI spec claiming camelCase for Stream)
- Stream API uses `ItemsPerPage` instead of `HasMoreItems`
- Stream API pagination query param is `itemsPerPage`, Core API uses `perPage`

### Get Video

```
GET /library/{libraryId}/videos/{videoId}
```

**Path Parameters:** `libraryId` (int64), `videoId` (string/GUID)

**Response (200):** `VideoModel`

### Create Video

```
POST /library/{libraryId}/videos
```

**Path Parameters:** `libraryId` (int64, required)

**Request Body (CreateVideoModel):**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| title | string | Yes (min 1) | Video title |
| collectionId | string | No | Collection to place video in |
| thumbnailTime | int32 | No | Time in ms for thumbnail extraction |

**Response (200):** `VideoModel` (includes the `guid` needed for upload)

**IMPORTANT:** Creating a video is a two-step process:
1. `POST /library/{libraryId}/videos` — creates a video placeholder, returns a `guid`
2. `PUT /library/{libraryId}/videos/{videoId}` — uploads the actual file to that guid

### Upload Video

```
PUT /library/{libraryId}/videos/{videoId}
Content-Type: application/octet-stream
```

**Path Parameters:** `libraryId` (int64), `videoId` (string/GUID)

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| jitEnabled | bool | Enable JIT encoding (Premium) |
| enabledResolutions | string | Comma-separated: 240p,360p,480p,720p,1080p,1440p,2160p |
| enabledOutputCodecs | string | Codec options: x264, vp9 |
| transcribeEnabled | bool | Enable transcription (costs extra) |
| transcribeLanguages | string | ISO 639-1 codes, comma-separated |
| sourceLanguage | string | ISO 639-1 code for spoken language |
| generateTitle | bool | Auto-generate title |
| generateDescription | bool | Auto-generate description |
| generateChapters | bool | Auto-generate chapters |
| generateMoments | bool | Auto-generate moments |

**Request Body:** Raw binary (application/octet-stream) — NOT multipart form data.

**Response (200):** `StatusModel`

**QUIRK — Upload is raw binary PUT, not multipart:**
The video file bytes are sent directly as the request body with `Content-Type: application/octet-stream`. This is different from many video APIs that use multipart form upload.

**QUIRK — Upload returns 400 if video already uploaded:**
You cannot re-upload to the same video ID. If the video already has content, the API returns 400.

### Update Video

```
POST /library/{libraryId}/videos/{videoId}
```

**Path Parameters:** `libraryId` (int64), `videoId` (string/GUID)

**Request Body (UpdateVideoModel):**

| Field | Type | Description |
|-------|------|-------------|
| title | string | Video title |
| collectionId | string | Collection ID |
| chapters | ChapterModel[] | Chapters array |
| moments | MomentModel[] | Moments array |
| metaTags | MetaTagModel[] | Meta tags array |

**Response (200):** `StatusModel`

### Delete Video

```
DELETE /library/{libraryId}/videos/{videoId}
```

**Response (200):** `StatusModel`

### Fetch Video (from URL)

```
POST /library/{libraryId}/videos/fetch?collectionId={collectionId}&thumbnailTime={thumbnailTime}
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| collectionId | string | Target collection |
| thumbnailTime | int32 | Thumbnail time in ms |

**Request Body (FetchVideoRequest):**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| url | string | Yes | Source video URL |
| headers | object | No | Custom headers for fetching |
| title | string | No | Video title |

**Response (200):** `StatusModel`

**QUIRK:** Returns 429 if too many concurrent fetch requests.

### Reencode Video

```
POST /library/{libraryId}/videos/{videoId}/reencode
```

**Response (200):** `VideoModel`

Returns 400 if original file is missing.

### Set Thumbnail

```
POST /library/{libraryId}/videos/{videoId}/thumbnail?thumbnailUrl={thumbnailUrl}
```

**Query Parameters:** `thumbnailUrl` (string, optional) — URL to fetch thumbnail from

**Request Body:** Binary (application/octet-stream, optional) — direct thumbnail upload

**Response (200):** `StatusModel`

Can either upload a thumbnail directly as binary or point to a URL.

### Add Caption

```
POST /library/{libraryId}/videos/{videoId}/captions/{srclang}
```

**Request Body (CaptionModelAdd):**

| Field | Type | Description |
|-------|------|-------------|
| srclang | string | Language code |
| label | string | Display label |
| captionsFile | string | **Base64-encoded** captions file content |

**QUIRK:** Captions file must be base64-encoded, not raw text or multipart.

**Response (200):** `StatusModel`

### Delete Caption

```
DELETE /library/{libraryId}/videos/{videoId}/captions/{srclang}
```

**Response (200):** `StatusModel`

### Transcribe Video

```
POST /library/{libraryId}/videos/{videoId}/transcribe?force={force}
```

**Query Parameters:** `force` (bool, default: false)

**Request Body (TranscribeSettings, optional/nullable):**
Details not fully documented in the OpenAPI spec.

**Response (200):** `StatusModel`

### Trigger Smart Actions (AI)

```
POST /library/{libraryId}/videos/{videoId}/smart
```

**Request Body (SmartGenerateModel):**
Details not fully documented.

**Response (202):** `StatusModel` — Note: returns 202 Accepted, not 200.
Returns 429 if limit exceeded.

### Get Video Play Data

```
GET /library/{libraryId}/videos/{videoId}/play?token={token}&expires={expires}
```

**Response (200):** `VideoPlayDataModel` — includes player URLs, DRM config, caption paths, etc.

### Get Video Heatmap

```
GET /library/{libraryId}/videos/{videoId}/heatmap
```

**Response (200):** `VideoHeatmapModel` — `{ "heatmap": { "0": 100, "1": 95, ... } }`

### Get Video Resolutions Info

```
GET /library/{libraryId}/videos/{videoId}/resolutions
```

**Response (200):** `StatusModelOfVideoResolutionsInfoModel`

### Get Video Storage Size

```
GET /library/{libraryId}/videos/{videoId}/storage
```

**Response (200):** `StatusModelOfVideoStorageSizeModel`
Returns 429 on rate limit.

### Cleanup Unconfigured Resolutions

```
POST /library/{libraryId}/videos/{videoId}/resolutions/cleanup
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| resolutionsToDelete | string | — | Specific resolutions |
| deleteNonConfiguredResolutions | bool | false | Delete unconfigured |
| deleteOriginal | bool | false | Delete original |
| deleteMp4Files | bool | false | Delete MP4 fallbacks |
| dryRun | bool | false | Preview without deleting |

**Response (200):** `StatusModel`

### Add Output Codec

```
PUT /library/{libraryId}/videos/{videoId}/outputs/{outputCodecId}
```

**Path Parameters:** `outputCodecId` (EncoderOutputCodec enum)

**Response (200):** `VideoModel`

### Repackage Video

```
POST /library/{libraryId}/videos/{videoId}/repackage?keepOriginalFiles={keepOriginalFiles}
```

**Query Parameters:** `keepOriginalFiles` (bool, default: true)

**Response (200):** `VideoModel`
Returns 400 if Enterprise DRM is not enabled.

### Get Video Statistics

```
GET /library/{libraryId}/statistics?dateFrom={dateFrom}&dateTo={dateTo}&hourly={hourly}&videoGuid={videoGuid}
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| dateFrom | date-time | — | Start date |
| dateTo | date-time | — | End date |
| hourly | bool | false | Hourly granularity |
| videoGuid | string | — | Filter to specific video |

**Response (200):** `VideoStatisticsModel`

```json
{
  "viewsChart": { "2026-03-18T00:00:00Z": 150, ... },
  "watchTimeChart": { "2026-03-18T00:00:00Z": 3600, ... },
  "countryViewCounts": { "US": 100, "DE": 50, ... },
  "engagementScore": 75
}
```

### OEmbed

```
GET /OEmbed?url={url}&maxWidth={maxWidth}&maxHeight={maxHeight}&token={token}&expires={expires}
```

**Response (200):** `VideoOEmbedModel`

---

## Collection Endpoints (Stream API — `https://video.bunnycdn.com`)

### List Collections

```
GET /library/{libraryId}/collections?page={page}&itemsPerPage={itemsPerPage}&search={search}&orderBy={orderBy}&includeThumbnails={includeThumbnails}
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| page | int32 | 1 | Page number |
| itemsPerPage | int32 | 100 | Items per page |
| search | string | — | Search filter |
| orderBy | string | "date" | Sort order |
| includeThumbnails | bool | false | Include thumbnail URLs |

**Response (200):** `PaginationListOfCollectionModel`

```json
{
  "TotalItems": 10,
  "CurrentPage": 1,
  "ItemsPerPage": 100,
  "Items": [CollectionModel, ...]
}
```

### Get Collection

```
GET /library/{libraryId}/collections/{collectionId}?includeThumbnails={includeThumbnails}
```

**Response (200):** `CollectionModel`

### Create Collection

```
POST /library/{libraryId}/collections
```

**Request Body:** `{ "name": "My Collection" }`

**Response (200):** `CollectionModel`

### Update Collection

```
POST /library/{libraryId}/collections/{collectionId}
```

**Request Body:** `{ "name": "New Name" }`

**Response (200):** `StatusModel`

**QUIRK:** Update uses POST, same pattern as everywhere else in bunny.net.

### Delete Collection

```
DELETE /library/{libraryId}/collections/{collectionId}
```

**Response (200):** `StatusModel`

---

## Key Data Models

### VideoModel

| Field | Type | Description |
|-------|------|-------------|
| videoLibraryId | int64 | Library ID |
| guid | string | Video GUID |
| title | string | Title |
| description | string? | Description |
| dateUploaded | date-time | Upload date |
| views | int64 | View count |
| isPublic | bool | Public visibility |
| length | int32 | Duration in seconds |
| status | int32 (enum) | Processing status |
| framerate | double | Frame rate |
| rotation | int32? | Rotation degrees |
| width | int32 | Width in pixels |
| height | int32 | Height in pixels |
| availableResolutions | string? | Comma-separated resolutions |
| outputCodecs | string | Output codecs |
| thumbnailCount | int32 | Number of thumbnails |
| encodeProgress | int32 | Encoding progress (0-100) |
| storageSize | int64 | Size in bytes |
| captions | CaptionModel[]? | Caption tracks |
| hasMP4Fallback | bool | MP4 fallback available |
| collectionId | string? | Collection ID |
| thumbnailFileName | string? | Thumbnail filename |
| thumbnailBlurhash | string? | Blurhash for thumbnail |
| averageWatchTime | int64 | Average watch time |
| totalWatchTime | int64 | Total watch time |
| category | string? | Content category |
| chapters | ChapterModel[]? | Chapters |
| moments | MomentModel[]? | Moments |
| metaTags | MetaTagModel[]? | Meta tags |
| transcodingMessages | TranscodingMessageModel[]? | Transcoding log |
| jitEncodingEnabled | bool? | JIT encoding |
| smartGenerateStatus | int32? (enum) | AI generation status |
| hasOriginal | bool? | Original file exists |
| originalHash | string? | Original file hash |
| hasHighQualityPreview | bool? | HQ preview exists |

### VideoModelStatus (enum)

| Value | Name | Description |
|-------|------|-------------|
| 0 | Created | Placeholder created, no file |
| 1 | Uploaded | File uploaded, not yet processing |
| 2 | Processing | Being processed |
| 3 | Transcoding | Being transcoded |
| 4 | Finished | Ready for playback |
| 5 | Error | Processing failed |
| 6 | UploadFailed | Upload failed |
| 7 | JitSegmenting | JIT segmenting in progress |
| 8 | JitPlaylistsCreated | JIT playlists ready |

### CollectionModel

| Field | Type | Description |
|-------|------|-------------|
| videoLibraryId | int64 | Library ID |
| guid | string? | Collection GUID |
| name | string? | Collection name |
| videoCount | int64 | Videos in collection |
| totalSize | int64 | Total size bytes |
| previewVideoIds | string? | Preview video IDs |
| previewImageUrls | string[]? | Preview image URLs |

### StatusModel

| Field | Type | Description |
|-------|------|-------------|
| success | bool | Success indicator |
| message | string? | Response message |
| statusCode | int32 | HTTP status code |

### ChapterModel

| Field | Type | Description |
|-------|------|-------------|
| title | string (required) | Chapter title |
| start | int32 | Start time (seconds) |
| end | int32 | End time (seconds) |

### MomentModel

| Field | Type | Description |
|-------|------|-------------|
| label | string (required) | Moment label |
| timestamp | int32 | Time (seconds) |

### MetaTagModel

| Field | Type | Description |
|-------|------|-------------|
| property | string? | Property name |
| value | string? | Property value |

---

## Key Quirks and Differences from Core API

### 1. Field naming: PascalCase (not camelCase as OpenAPI spec claims)

- **Core API** (video libraries): PascalCase — `Id`, `Name`, `VideoCount`, `ApiKey`
- **Stream API** (videos, collections): **Also PascalCase** — `Guid`, `Title`, `VideoLibraryId`, `DateUploaded`

**IMPORTANT:** The OpenAPI spec at `video.bunnycdn.com` describes fields in camelCase, but the actual live API returns PascalCase. Our types use `#[serde(rename_all = "PascalCase")]` which matches the real API behavior.

### 2. Pagination format differs

**Core API:**
```json
{ "Items": [...], "CurrentPage": 1, "TotalItems": 50, "HasMoreItems": true }
```

**Stream API:**
```json
{ "TotalItems": 50, "CurrentPage": 1, "ItemsPerPage": 100, "Items": [...] }
```

Both use PascalCase. The Stream API uses `ItemsPerPage` instead of `HasMoreItems`.

### 3. Pagination param name differs

- Core API: `perPage`
- Stream API: `itemsPerPage`

### 4. Two-step video upload

Upload is not a single request. You must:
1. Create a video object first (`POST /library/{libraryId}/videos`) to get a GUID
2. Upload the binary to that GUID (`PUT /library/{libraryId}/videos/{videoId}`)

### 5. Upload is raw binary PUT, not multipart

`PUT` with `Content-Type: application/octet-stream` and raw bytes as body. No multipart form data.

### 6. Video IDs are GUIDs (strings), not integers

Unlike Core API resources (pull zones, storage zones, DNS zones) which use integer IDs, videos and collections use string GUIDs.

### 7. Library IDs are integers, passed as path params to Stream API

The `libraryId` in Stream API paths is an int64 (same as the `Id` from the Core API's VideoLibraryModel).

### 8. Updates use POST (consistent with Core API)

Both Core and Stream APIs use `POST` for updates, not `PATCH` or `PUT`.

### 9. Captions upload requires base64 encoding

The caption file content must be base64-encoded in the JSON request body, not sent as a file upload.

### 10. Fetch video returns 429 on rate limit

The fetch-from-URL endpoint has rate limiting and returns 429 Too Many Requests.

### 11. Smart actions return 202 Accepted

The `/smart` endpoint returns HTTP 202 (async), not 200.

### 12. Re-upload not allowed

Uploading to a video ID that already has content returns 400. You cannot overwrite — you must delete and recreate.

### 13. Stream API key derivation

The per-library API key is available in the `ApiKey` field of the VideoLibraryModel (fetched via Core API). This is the same pattern as Storage zones where the `Password` field provides the per-zone key.

---

## Implementation Notes for Iteration 4

### New crate: `bunny-api-stream`

Needs its own HTTP client pointing at `https://video.bunnycdn.com` with:
- `AccessKey` header (per-library key)
- `#[serde(rename_all = "camelCase")]` on all types
- Separate `PaginatedList<T>` with camelCase fields and `itemsPerPage` instead of `HasMoreItems`
- Video IDs as `String` (GUIDs), library IDs as `i64`

### Auth resolution (same pattern as Storage)

1. Check `BUNNY_STREAM_KEY` env var
2. If absent, fetch the library via Core API (`GET /videolibrary/{id}`) and use the `ApiKey` field

### CLI command mapping

```
hoppy stream library list|get|create|update|delete    → Core API
hoppy stream video list|get|upload|delete              → Stream API
hoppy stream video fetch --url <url>                   → Stream API
hoppy stream collection list|get|create|update|delete  → Stream API
```

### Upload flow in CLI

```
hoppy stream video upload --library-id 123 --file video.mp4 --title "My Video"
```

Internally:
1. POST to create video with title → get GUID
2. PUT binary to upload file to that GUID
3. Optionally poll status until `Finished` (or print GUID and let user check)
