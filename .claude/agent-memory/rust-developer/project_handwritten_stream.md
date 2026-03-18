---
name: Hand-written stream client implementation notes
description: Key decisions and gotchas from implementing hoppy-api-stream by hand (iter-0.5/hand-written-experiment)
type: project
---

Hand-written `hoppy-api-stream` crate created at `crates/hoppy-api-stream/` on branch `iter-0.5/hand-written-experiment`.

**Why:** Part of a codegen-vs-hand-written comparison experiment. Progenitor could not handle `application/octet-stream` binary upload bodies — the hand-written client demonstrates this as a first-class advantage.

**Key implementation decisions:**
- `VideoStatus` uses `serde_repr` (integer enum) matching `VideoModelStatus` in the spec (0=Created … 8=JitPlaylistsCreated). The spec also has a `VideoStatus` schema that is identical — `VideoModelStatus` is the one referenced from `VideoModel`.
- `HasMP4Fallback` must be explicitly `#[serde(rename = "HasMP4Fallback")]` — serde's `rename_all = "PascalCase"` converts `has_mp4_fallback` to `HasMp4Fallback` (lowercases the acronym), which doesn't match the wire format. Check for similar acronym fields (MP4, URL, ID) on other structs.
- `PaginatedList<T>` items field uses `#[serde(default = "Vec::new")]` instead of `#[serde(default)]` — the latter triggers a `T: Default` bound on serde's generated impl, causing compile errors when `T` (e.g. `Video`) doesn't implement `Default`.
- `upload_video` accepts `impl Into<reqwest::Body>` — callers can pass `Vec<u8>`, `bytes::Bytes`, or a streaming `tokio::fs::File` via `reqwest::Body::wrap_stream`.

**How to apply:** When adding fields to `Video` or other structs, check if the JSON key contains an all-caps acronym and add an explicit `#[serde(rename = "...")]` if needed.
