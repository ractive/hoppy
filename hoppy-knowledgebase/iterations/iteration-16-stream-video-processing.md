---
title: Iteration 16 — Stream Video Processing
type: iteration
date: 2026-03-20
tags:
  - iteration
  - api-coverage
  - stream
  - video
status: completed
branch: iter-16/stream-video-processing
---

# Iteration 16 — Stream Video Processing

**Goal:** Add video processing and analytics endpoints to the stream API. These cover transcription, re-encoding, thumbnails, heatmaps, resolution management, and storage size queries.

## Context

The stream API has 13 unimplemented endpoints beyond basic CRUD. These fall into three categories:

1. **Processing triggers** — kick off server-side work (transcribe, reencode, repackage, smart generate)
2. **Analytics** — read-only data about viewer engagement (heatmaps)
3. **Metadata queries** — resolution info, storage size

All operate on existing videos within a library. Most are simple POST (trigger) or GET (query) with minimal parameters.

**OpenAPI ref:** `specs/stream.json`

## Scope

### 1. Video Transcription

- [x] API client (`bunny-api-stream`): `POST /library/{libraryId}/videos/{videoId}/transcribe` — body: optional `TranscribeSettings` (language, force re-transcribe)
- [x] Add `TranscribeSettings` type (check spec for fields)
- [x] CLI: `hoppy stream video transcribe --library-id <id> --video-id <id> --language <lang>` (optional `--force`)
- [x] Wiremock + insta snapshot test
- [x] Live E2E test stub (gated by `live-api` feature, skips unless `TEST_VIDEO_PATH` set)

### 2. Video Heatmap / Engagement Analytics

- [x] API client: `GET /library/{libraryId}/videos/{videoId}/heatmap`
- [x] Add `VideoHeatmap` response type
- [x] CLI: `hoppy stream video heatmap --library-id <id> --video-id <id>`
- [x] Wiremock + insta snapshot test

### 3. Re-encoding

- [x] API client: `POST /library/{libraryId}/videos/{videoId}/reencode`
- [x] API client: `PUT /library/{libraryId}/videos/{videoId}/outputs/{outputCodecId}` — reencode using specific codec
- [x] CLI: `hoppy stream video reencode --library-id <id> --video-id <id>` (full reencode)
- [x] CLI: `hoppy stream video reencode --library-id <id> --video-id <id> --codec <codec-id>` (codec-specific)
- [x] Wiremock + insta snapshot tests

### 4. Repackage

- [x] API client: `POST /library/{libraryId}/videos/{videoId}/repackage`
- [x] CLI: `hoppy stream video repackage --library-id <id> --video-id <id>`
- [x] Wiremock + insta snapshot test

### 5. Smart Generate (AI Features)

- [x] API client: `POST /library/{libraryId}/videos/{videoId}/smart`
- [x] CLI: `hoppy stream video smart-generate --library-id <id> --video-id <id>`
- [x] Wiremock + insta snapshot test

### 6. Set Thumbnail

- [x] API client: `POST /library/{libraryId}/videos/{videoId}/thumbnail` — body: `thumbnailUrl` query param
- [x] CLI: `hoppy stream video set-thumbnail --library-id <id> --video-id <id> --thumbnail-url <url>`
- [x] Wiremock + insta snapshot test

### 7. Resolution Management

- [x] API client: `GET /library/{libraryId}/videos/{videoId}/resolutions` — get available resolutions/codecs
- [x] API client: `POST /library/{libraryId}/videos/{videoId}/resolutions/cleanup` — delete specific resolutions
- [x] Add `VideoResolutionsInfo`, `StreamCleanupResolutions` types
- [x] CLI: `hoppy stream video resolutions list --library-id <id> --video-id <id>`
- [x] CLI: `hoppy stream video resolutions cleanup --library-id <id> --video-id <id>` — with confirmation
- [x] Wiremock + insta snapshot tests

### 8. Storage Size

- [x] API client: `GET /library/{libraryId}/videos/{videoId}/storage` — get storage breakdown
- [x] Add `VideoStorageSize`, `CodecRenditionSize` types
- [x] CLI: `hoppy stream video storage --library-id <id> --video-id <id>`
- [x] Wiremock + insta snapshot test

---

## Implementation Order

1. **Transcription** — highest user value, standalone
2. **Heatmap** — read-only, simple
3. **Re-encoding + Repackage** — related processing triggers
4. **Resolution management** — depends on understanding codec outputs
5. **Smart generate** — may need API experimentation to understand options
6. **Thumbnail** — simple but needs a valid URL to test
7. **Storage size** — read-only, simple

## Implementation Notes

- Processing triggers (transcribe, reencode, repackage, smart) are fire-and-forget — the API returns immediately and processing happens asynchronously. The `Video.Status` field reflects progress.
- For live E2E tests involving encoding, the test needs to upload a short video file and may need to poll `get_video()` until `Status == Finished` before testing reencode/resolution endpoints.
- Keep a small test video file in `tests/fixtures/` for upload tests (existing test may already have one).
- The OEmbed endpoint (`GET /OEmbed`) is intentionally excluded — it's for embed HTML generation, not a typical CLI operation.

## Estimated Complexity

| Topic | New API methods | New CLI commands | Complexity |
|-------|----------------|-----------------|------------|
| Transcription | 1 | 1 | Small |
| Heatmap | 1 | 1 | Small |
| Re-encoding | 2 | 1 (with --codec flag) | Small |
| Repackage | 1 | 1 | Small |
| Smart generate | 1 | 1 | Small |
| Thumbnail | 1 | 1 | Small |
| Resolutions | 2 | 2 | Small |
| Storage size | 1 | 1 | Small |
| **Total** | **10** | **9** | **Medium** |

## Related

- [[development-roadmap]] — project roadmap
- [[adding-a-feature]] — implementation checklist
- [[api/bunny-stream-api-research]] — stream API research docs
- [[api/bunny-api-client-patterns]] — client patterns
