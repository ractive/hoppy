---
title: "Gap report: Stream API + Video Library"
type: research
date: 2026-07-03
status: active
origin: api-coverage deep dive 2026-07-03 (agent-generated, verified against specs + CLI source)
tags:
  - api-coverage
  - gap-analysis
  - stream
  - video-library
---

# Stream + Video Library gap report

Domain: Stream API (`video.bunnycdn.com`, 28 ops) + Video Library management (`api.bunny.net /videolibrary*`, 20 ops).
Sources: spec inventories vs `hoppy stream` / `hoppy video-library` help dumps, cross-checked against
`crates/hoppy-cli/src/commands/stream.rs`, `video_library.rs`, `crates/bunny-net-api/src/stream/client.rs` + `types.rs`,
and `crates/bunny-net-api/src/core/client.rs` + `types.rs`. The full help tree was grepped — no other command group
exposes any of these endpoints (pull zone `referrer` commands hit `/pullzone/*`, not `/videolibrary/*`).

## 1. Endpoint coverage

### Stream API (video.bunnycdn.com)

| METHOD path | CLI command | Status | Notes |
|---|---|---|---|
| GET /OEmbed | — | missing | No oembed command anywhere in the tree |
| GET /library/{lib}/collections | `stream collection list` | partial | Drops `includeThumbnails` |
| POST /library/{lib}/collections | `stream collection create` | covered | |
| GET /library/{lib}/collections/{col} | `stream collection get` | partial | Drops `includeThumbnails` |
| POST /library/{lib}/collections/{col} | `stream collection update` | covered | |
| DELETE /library/{lib}/collections/{col} | `stream collection delete` | covered | Confirmation prompt unless `--yes` |
| GET /library/{lib}/statistics | `stream library statistics` | covered | All 4 query params mapped |
| GET /library/{lib}/videos | `stream video list` | covered | All 5 query params + client-side `--all` |
| POST /library/{lib}/videos (Create Video) | `stream video upload` (step 1) | partial | No standalone create; drops `thumbnailTime` (client builder `CreateVideo::thumbnail_time` exists, no flag) |
| POST /library/{lib}/videos/fetch | `stream video fetch` | partial | Drops query `collectionId`, `thumbnailTime` and body `headers` |
| GET /library/{lib}/videos/{vid} | `stream video get` | covered | |
| POST /library/{lib}/videos/{vid} (Update) | `stream video update` | partial | Drops `chapters`, `moments`, `metaTags` (absent from client `UpdateVideo` too) |
| PUT /library/{lib}/videos/{vid} (Upload) | `stream video upload` (step 2) | partial | Streams body correctly, but drops all 10 query params (jit/resolutions/codecs/transcribe/generate*) |
| DELETE /library/{lib}/videos/{vid} | `stream video delete` | covered | Confirmation prompt unless `--yes` |
| POST /library/{lib}/videos/{vid}/captions/{srclang} | `stream video caption add` | partial | Drops body `label`; `srclang` body field intentionally omitted (path carries it) |
| DELETE /library/{lib}/videos/{vid}/captions/{srclang} | `stream video caption delete` | covered | |
| GET /library/{lib}/videos/{vid}/heatmap | `stream video heatmap` | covered | |
| PUT /library/{lib}/videos/{vid}/outputs/{codecId} | `stream video reencode --codec` | covered | Folded into reencode; codec accepts names x264/vp9/hevc/av1 and ints 0–3, maps to spec enum [0,1,2,3] |
| GET /library/{lib}/videos/{vid}/play | — | missing | Player-facing; `token`/`expires` signing unsupported |
| GET /library/{lib}/videos/{vid}/play/heatmap | — | missing | Distinct from `/heatmap` (which IS covered) |
| POST /library/{lib}/videos/{vid}/reencode | `stream video reencode` | covered | |
| POST /library/{lib}/videos/{vid}/repackage | `stream video repackage` | covered | `keepOriginalFiles` inverted as `--discard-originals`; only sent when false (API default true) |
| GET /library/{lib}/videos/{vid}/resolutions | `stream video resolutions list` | covered | |
| POST /library/{lib}/videos/{vid}/resolutions/cleanup | `stream video resolutions cleanup` | covered | All 5 query params mapped; destructive-op guard + `--dry-run` |
| POST /library/{lib}/videos/{vid}/smart | `stream video smart-generate` | covered | All 5 body fields mapped |
| GET /library/{lib}/videos/{vid}/storage | `stream video storage` | covered | |
| POST /library/{lib}/videos/{vid}/thumbnail | `stream video set-thumbnail` | partial | Only `thumbnailUrl` query variant; octet-stream binary-upload variant unsupported |
| POST /library/{lib}/videos/{vid}/transcribe | `stream video transcribe` | covered | `force` + all 6 body fields mapped |

### Video Library management (api.bunny.net)

| METHOD path | CLI command | Status | Notes |
|---|---|---|---|
| GET /videolibrary | `stream library list` | covered | Note: lives under `stream library`, not `video-library` |
| POST /videolibrary | `stream library create` | partial | Only `Name` sent; client's `replication_regions` builder not exposed (spec body is `<unknown>`) |
| GET /videolibrary/languages | — | missing | |
| GET /videolibrary/{id} | `stream library get` | covered | ApiKey/ReadOnlyApiKey redacted unless `--reveal` |
| POST /videolibrary/{id} | `stream library update` | partial | Only Name / AllowDirectPlay / EnableMP4Fallback / HasWatermark; the real API's large settings body (EnabledResolutions, player options, transcription defaults, …) unreachable |
| DELETE /videolibrary/{id} | `stream library delete` | covered | Confirmation prompt unless `--yes` |
| POST /videolibrary/{id}/addAllowedReferrer | — | missing | Pull-zone `referrer` group targets `/pullzone/{id}/…`, not videolibrary |
| POST /videolibrary/{id}/addBlockedReferrer | — | missing | |
| GET /videolibrary/{id}/drm/statistics | `video-library drm-statistics` | covered | |
| PUT /videolibrary/{id}/live/thumbnail | — | missing | |
| DELETE /videolibrary/{id}/live/thumbnail | — | missing | |
| PUT /videolibrary/{id}/live/watermark | — | missing | |
| DELETE /videolibrary/{id}/live/watermark | — | missing | |
| POST /videolibrary/{id}/removeAllowedReferrer | — | missing | |
| POST /videolibrary/{id}/removeBlockedReferrer | — | missing | |
| POST /videolibrary/{id}/resetApiKey | — | missing | No key-rotation path in CLI |
| POST /videolibrary/{id}/resetReadOnlyApiKey | — | missing | |
| GET /videolibrary/{id}/transcribing/statistics | `video-library transcribing-statistics` | covered | |
| PUT /videolibrary/{id}/watermark | — | missing | `stream library update --has-watermark` only toggles the flag; no image upload |
| DELETE /videolibrary/{id}/watermark | — | missing | |

## 2. Flag-level gaps per command

### hoppy stream library list  (GET /videolibrary)

| Spec param | Flag |
|---|---|
| page | `--page` (client defaults to 1 when omitted) |
| perPage | `--per-page` (client defaults to `DEFAULT_PER_PAGE` when omitted) |
| search | `--search` |
| — | `--all` (CLI-only client-side auto-pagination, 1000/page) |

### hoppy stream library create  (POST /videolibrary)

| Spec body prop | Flag |
|---|---|
| Name | `--name` |
| ReplicationRegions | **MISSING** (exists in client `CreateVideoLibrary::replication_regions`, never reachable from CLI) |
| (rest of body — spec `<unknown>`) | **MISSING** |

### hoppy stream library update  (POST /videolibrary/{id})

| Spec body prop | Flag |
|---|---|
| Name | `--name` |
| AllowDirectPlay | `--allow-direct-play <true\|false>` |
| EnableMP4Fallback | `--enable-mp4-fallback <true\|false>` |
| HasWatermark | `--has-watermark <true\|false>` |
| everything else (EnabledResolutions, PlayerKeyColor, CaptionsFontSize, WebhookUrl, KeepOriginalFiles, AllowEarlyPlay, EnableDRM, transcription defaults, …) | **MISSING** — spec body is `<unknown>` but the live API accepts dozens of fields; `UpdateVideoLibrary` models only these 4 |

### hoppy stream library statistics  (GET /library/{lib}/statistics)

| Spec param | Flag |
|---|---|
| dateFrom | `--date-from` (normalised via `date::normalise_datetime_opt`) |
| dateTo | `--date-to` |
| hourly | `--hourly` |
| videoGuid | `--video-guid` |

### hoppy stream video list  (GET /library/{lib}/videos)

| Spec param | Flag |
|---|---|
| page | `--page` |
| itemsPerPage | `--items-per-page` |
| search | `--search` |
| collection | `--collection` |
| orderBy | `--order-by` (free string; spec/API values `date`/`title` not enumerated or validated) |
| — | `--all` (CLI-only) |

### hoppy stream video upload  (POST create + PUT upload, composite)

Create Video body:
| Spec body prop | Flag |
|---|---|
| title (required) | `--title` (defaults to filename — hardcoded fallback in stream.rs:540) |
| collectionId | `--collection-id` |
| thumbnailTime | **MISSING** (client builder exists: `CreateVideo::thumbnail_time`) |

Upload Video (PUT) query params — **all 10 MISSING**, client `upload_video()` accepts none:
| Spec param | Flag |
|---|---|
| jitEnabled | **MISSING** |
| enabledResolutions | **MISSING** |
| enabledOutputCodecs | **MISSING** |
| transcribeEnabled | **MISSING** |
| transcribeLanguages | **MISSING** |
| sourceLanguage | **MISSING** |
| generateTitle / generateDescription / generateChapters / generateMoments | **MISSING** (4 params) |

Body: streamed via `reqwest::Body::wrap_stream(ReaderStream)` with progress bar — compliant with the streaming-bodies guideline.

### hoppy stream video update  (POST /library/{lib}/videos/{vid})

| Spec body prop | Flag |
|---|---|
| title | `--title` |
| collectionId | `--collection-id` |
| chapters | **MISSING** (not in client `UpdateVideo` either) |
| moments | **MISSING** |
| metaTags | **MISSING** |

### hoppy stream video fetch  (POST /library/{lib}/videos/fetch)

| Spec param | Flag |
|---|---|
| query collectionId | **MISSING** (client `fetch_video` sends no query params) |
| query thumbnailTime | **MISSING** |
| body url (required) | `--url` |
| body title | `--title` |
| body headers | **MISSING** (client builder `FetchVideo::header` exists, no CLI flag — needed for auth-protected source URLs) |

### hoppy stream video caption add  (POST …/captions/{srclang})

| Spec param | Flag |
|---|---|
| path srclang | `--srclang` |
| body srclang | intentionally omitted (redundant with path) |
| body label | **MISSING** |
| body captionsFile | `--file` (CLI reads file with `read_to_string` and sends raw text as `CaptionsFile`; Bunny docs describe this field as base64-encoded content — see Observations) |

### hoppy stream video caption delete — path params only, fully mapped.

### hoppy stream video transcribe  (POST …/transcribe)

| Spec param | Flag |
|---|---|
| query force | `--force` |
| body sourceLanguage | `--language` |
| body targetLanguages | `--target-language` (repeatable) |
| body generateTitle / generateDescription / generateChapters / generateMoments | `--generate-title` / `--generate-description` / `--generate-chapters` / `--generate-moments` |

Note: flags are presence-only booleans — you can turn options on but cannot send an explicit `false` to override library defaults. Body omitted entirely when no setting flags given.

### hoppy stream video heatmap — GET …/heatmap, path params only, fully mapped.

### hoppy stream video reencode  (POST …/reencode and PUT …/outputs/{codecId})

| Spec param | Flag |
|---|---|
| path outputCodecId enum [0,1,2,3] | `--codec` accepts x264/vp9/hevc/av1 or 0–3 → full enum coverage; when omitted, hits the plain `/reencode` endpoint |

### hoppy stream video repackage  (POST …/repackage)

| Spec param | Flag |
|---|---|
| keepOriginalFiles | `--discard-originals` (inverted; param only sent as `false` when flag given — API default `true` otherwise) |

### hoppy stream video smart-generate  (POST …/smart)

| Spec body prop | Flag |
|---|---|
| generateTitle / generateDescription / generateChapters / generateMoments | `--generate-*` flags (presence-only booleans, no explicit-false) |
| sourceLanguage | `--language` |

### hoppy stream video set-thumbnail  (POST …/thumbnail)

| Spec param | Flag |
|---|---|
| query thumbnailUrl | `--thumbnail-url` (required by CLI, though spec makes it optional) |
| octet-stream body (upload image binary) | **MISSING** — client `set_video_thumbnail` has no body support; also no way to clear a thumbnail (client accepts `None` but CLI requires the flag) |

### hoppy stream video resolutions list — GET …/resolutions, path params only, fully mapped.

### hoppy stream video resolutions cleanup  (POST …/resolutions/cleanup)

| Spec param | Flag |
|---|---|
| resolutionsToDelete | `--resolutions` |
| deleteNonConfiguredResolutions | `--delete-non-configured` |
| deleteOriginal | `--delete-original` |
| deleteMp4Files | `--delete-mp4-files` |
| dryRun | `--dry-run` |

### hoppy stream video storage — GET …/storage, path params only, fully mapped.

### hoppy stream collection list  (GET /library/{lib}/collections)

| Spec param | Flag |
|---|---|
| page | `--page` |
| itemsPerPage | `--items-per-page` |
| search | `--search` |
| orderBy | `--order-by` |
| includeThumbnails | **MISSING** (client `list_collections` doesn't send it) |
| — | `--all` (CLI-only) |

### hoppy stream collection get  (GET /library/{lib}/collections/{col})

| Spec param | Flag |
|---|---|
| includeThumbnails | **MISSING** (client `get_collection` doesn't send it) |

### hoppy stream collection create / update — `name` → `--name`; fully mapped.

### hoppy stream collection delete — path params only, fully mapped.

### hoppy video-library drm-statistics  (GET /videolibrary/{id}/drm/statistics)

| Spec param | Flag |
|---|---|
| dateFrom | `--date-from` |
| dateTo | `--date-to` |

### hoppy video-library transcribing-statistics  (GET /videolibrary/{id}/transcribing/statistics)

| Spec param | Flag |
|---|---|
| dateFrom | `--date-from` |
| dateTo | `--date-to` |

## 3. CLI-only surface

No command in this domain hits an undocumented endpoint — every URL built in `stream/client.rs` and the videolibrary
section of `core/client.rs` appears in the spec inventories. CLI-only *behaviors*:

- `--all` on `stream library list`, `stream video list`, `stream collection list`: client-side auto-pagination
  (hardcoded 1000 items/page loop), not an API param.
- `stream video upload` is a composite command: POST Create Video, then PUT Upload Video with a streamed body and
  progress bar. There is no standalone `create` (metadata-only) command.
- `stream video reencode --codec` multiplexes two spec endpoints (`/reencode` vs `/outputs/{codecId}`).
- Stream-key auto-resolution (`resolve_stream_client`, stream.rs:397): when `BUNNY_STREAM_KEY` is unset, the CLI
  makes an extra documented call `GET /videolibrary/{id}` on the core API to fetch the library's ApiKey. Extra
  traffic, not an undocumented endpoint.
- Redaction UX: `ApiKey`/`ReadOnlyApiKey` hidden unless `--reveal` — output-side only.
- `stream video update` performs a follow-up `GET` to print the updated video (extra documented call).

## 4. Observations

- **Deprecated endpoints**: none of the 48 inventory operations are marked deprecated in either spec inventory.
- **Pagination**: handled well overall. Stream lists map `page`/`itemsPerPage` and add `--all`; "has more" is computed
  client-side (`current_page * items_per_page < total_items`, stream.rs:1266) because the Stream API's paginated list
  lacks a `HasMoreItems` field. Core `list_video_libraries` always sends `page`/`perPage` (defaults applied client-side).
  `includeThumbnails` on collection list/get is the only dropped read-side param in the Stream API.
- **TUS resumable uploads**: bunny.net Stream supports TUS resumable uploads (`https://video.bunnycdn.com/tusupload`
  with signature-based auth). It is absent from the spec inventory AND from client/CLI. For large files over flaky
  links, the single-shot PUT is the only option — a known real-world gap that the inventory diff alone won't surface.
- **Caption encoding risk**: `caption add` sends the SRT file content as a raw string in `CaptionsFile`
  (stream.rs:1060, client.rs:569). Bunny's docs describe `captionsFile` as base64-encoded caption content. If the API
  requires base64, non-trivial files will be corrupted or rejected — worth a live verification/dogfood check.
  `read_to_string` also fails on non-UTF-8 caption files (e.g. UTF-16 SRTs).
- **Boolean flags can't express `false`**: transcribe/smart-generate `--generate-*` flags are presence-only, so a
  library-default of `true` can't be overridden to `false` per-call. Same pattern for `--hourly`, `--force`.
- **`set-thumbnail` requires `--thumbnail-url`** even though the spec marks the query param optional and the endpoint
  accepts a binary image body — no local-file thumbnail upload, no thumbnail clearing.
- **`video-library` vs `stream library` split**: core `/videolibrary` CRUD lives under `hoppy stream library`, while
  the top-level `hoppy video-library` group holds only the two statistics commands. Functional, but a user grepping
  `video-library --help` won't find create/update/delete.
- **The library-update settings surface is the biggest blind spot**: the spec inventory shows `<unknown>` for the
  create/update bodies, but the live API accepts a large settings object (enabled resolutions, output codecs, player
  settings, webhooks, DRM, transcription defaults…). The CLI reaches exactly 4 fields.

## Summary counts

- **Total operations**: 48 (Stream API 28, Video Library management 20)
- **Covered**: 22 (Stream 17, Video Library 5)
- **Partial**: 10 (Stream 8: collection list/get, video create/upload, fetch, update, caption add, set-thumbnail; Video Library 2: create, update)
- **Missing**: 16 (Stream 3: OEmbed, play, play/heatmap; Video Library 13: languages, 4 referrer ops, 2 watermark ops, 4 live thumbnail/watermark ops, resetApiKey, resetReadOnlyApiKey)

**5 most impactful gaps:**
1. `resetApiKey` / `resetReadOnlyApiKey` missing — no way to rotate a leaked Stream library key from the CLI (security-relevant).
2. Video library update reaches only 4 body fields — the entire library settings surface (enabled resolutions, output codecs, player/webhook/DRM/transcription config) plus the 4 referrer allow/block endpoints are unreachable.
3. Upload Video drops all 10 per-upload query params (`jitEnabled`, `enabledResolutions`, `enabledOutputCodecs`, `transcribeEnabled`, `transcribeLanguages`, `sourceLanguage`, `generate*`) and there is no TUS resumable upload — no control over encoding/transcription at upload time.
4. Update Video drops `chapters`, `moments`, `metaTags` — video metadata beyond title/collection cannot be managed (not even modeled in the client).
5. Watermark management missing end-to-end: `PUT/DELETE /videolibrary/{id}/watermark` and the 4 live thumbnail/watermark ops have no command; `--has-watermark` only toggles a flag with no way to upload the watermark image.
