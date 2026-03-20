---
title: "Iteration 16 — Stream Video Processing"
type: iteration
date: 2026-03-20
tags:
  - iteration
  - api-coverage
  - stream
  - video
status: planned
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

- [ ] API client (`bunny-api-stream`): `POST /library/{libraryId}/videos/{videoId}/transcribe` — body: optional `TranscribeSettings` (language, force re-transcribe)
- [ ] Add `TranscribeSettings` type (check spec for fields)
- [ ] CLI: `hoppy stream video transcribe --library-id <id> --video-id <id> --language <lang>` (optional `--force`)
- [ ] Capture fixture via `--record`
- [ ] Wiremock + insta snapshot test
- [ ] Live E2E test: upload video → transcribe → verify status → cleanup

### 2. Video Heatmap / Engagement Analytics

- [ ] API client: `GET /library/{libraryId}/videos/{videoId}/heatmap`
- [ ] Add `VideoHeatmap` response type
- [ ] CLI: `hoppy stream video heatmap --library-id <id> --video-id <id>`
- [ ] Capture fixture via `--record`
- [ ] Wiremock + insta snapshot test
- [ ] Live E2E test: include in transcribe lifecycle test (heatmap may be empty for new videos — that's fine, just verify the call succeeds)

### 3. Re-encoding

- [ ] API client: `POST /library/{libraryId}/videos/{videoId}/reencode`
- [ ] API client: `PUT /library/{libraryId}/videos/{videoId}/outputs/{outputCodecId}` — reencode using specific codec
- [ ] CLI: `hoppy stream video reencode --library-id <id> --video-id <id>` (full reencode)
- [ ] CLI: `hoppy stream video reencode --library-id <id> --video-id <id> --codec <codec-id>` (codec-specific)
- [ ] Capture fixtures via `--record`
- [ ] Wiremock + insta snapshot tests
- [ ] Live E2E: upload video → wait for encoding → reencode → verify status

### 4. Repackage

- [ ] API client: `POST /library/{libraryId}/videos/{videoId}/repackage`
- [ ] CLI: `hoppy stream video repackage --library-id <id> --video-id <id>`
- [ ] Capture fixture via `--record`
- [ ] Wiremock + insta snapshot test

### 5. Smart Generate (AI Features)

- [ ] API client: `POST /library/{libraryId}/videos/{videoId}/smart`
- [ ] CLI: `hoppy stream video smart-generate --library-id <id> --video-id <id>`
- [ ] Check OpenAPI spec for request body fields — may include options for what to generate (captions, chapters, title, etc.)
- [ ] Capture fixture via `--record`
- [ ] Wiremock + insta snapshot test

### 6. Set Thumbnail

- [ ] API client: `POST /library/{libraryId}/videos/{videoId}/thumbnail` — body: `{ "ThumbnailUrl": "..." }` (check spec)
- [ ] CLI: `hoppy stream video set-thumbnail --library-id <id> --video-id <id> --thumbnail-url <url>`
- [ ] Capture fixture via `--record`
- [ ] Wiremock + insta snapshot test
- [ ] Live E2E: upload video → set thumbnail → verify via get → cleanup

### 7. Resolution Management

- [ ] API client: `GET /library/{libraryId}/videos/{videoId}/resolutions` — get available resolutions/codecs
- [ ] API client: `POST /library/{libraryId}/videos/{videoId}/resolutions/cleanup` — delete specific resolutions
- [ ] Add `VideoResolutions` response type
- [ ] CLI: `hoppy stream video resolutions --library-id <id> --video-id <id>` — list resolutions
- [ ] CLI: `hoppy stream video resolutions cleanup --library-id <id> --video-id <id>` — with confirmation
- [ ] Capture fixtures via `--record`
- [ ] Wiremock + insta snapshot tests
- [ ] Live E2E: part of reencode lifecycle

### 8. Storage Size

- [ ] API client: `GET /library/{libraryId}/videos/{videoId}/storage` — get storage breakdown
- [ ] Add `VideoStorageSize` response type
- [ ] CLI: `hoppy stream video storage --library-id <id> --video-id <id>`
- [ ] Capture fixture via `--record`
- [ ] Wiremock + insta snapshot test

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
