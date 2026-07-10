---
title: Iter-77 — stream TUS resumable upload
type: iteration
date: 2026-07-03
tags:
  - iteration
  - api-coverage
  - stream
  - upload
status: completed
branch: iter-77/stream-tus-upload
priority: 4
related:
  - research/api-coverage-gap-analysis-2026-07
  - research/api-coverage-2026-07/stream
---

# Iter-77 — stream TUS resumable upload

> [!note] Carried forward from iter-76
> Iter-76 (containers polish) shipped clean — no shared code paths with
> Stream, no blocking API quirks. Two reusable patterns worth keeping in
> mind here, both from [[decision-log]]:
> - **Escape hatch for deep/uncertain schemas**: iter-76 used a
>   `--probes-json <file>` flag instead of ~40 individual flags for a
>   deeply nested schema. TUS's per-upload params (§3) list is already
>   large (`jitEnabled`, `enabledResolutions`, `enabledOutputCodecs`,
>   `transcribe*`, `generate*`, `sourceLanguage`) — if the metadata-header
>   encoding turns out to be as deep/uncertain as containers' probes,
>   prefer a similar file-based escape hatch over a flag explosion rather
>   than guessing at full flag mapping up front.
> - **Docs-only / schema-less surfaces get raw passthrough, not guessed
>   types**: iter-76 returned `serde_json::Value` verbatim for endpoints
>   the spec left schema-less, deferring typed models to live
>   verification. TUS here is docs-only entirely (no OpenAPI spec at
>   all) — resist the urge to over-type the TUS creation/offset
>   responses; verify shapes live before locking in structs, same
>   caution as api-coverage research §4.5.
>
> No scope changes needed — the plan below already reflects these.

## Why

Per [[research/api-coverage-2026-07/stream]] §4, bunny Stream supports
TUS resumable uploads (`video.bunnycdn.com/tusupload`,
signature-based auth) but it is docs-only — no spec, no client, no CLI.
For large files over flaky links the single-shot PUT is the only option
today. Biggest single feature of the plan; scheduled last for that
reason.

## Scope

### 1. TUS protocol client [3/3]

- [x] Implement a TUS 1.0 client in `crates/bunny-net-api/src/stream/`
  (or a `tus` submodule): creation request, `HEAD` offset probe,
  `PATCH` chunk upload with `Upload-Offset` handling, per
  `docs.bunny.net/stream/tus-resumable-uploads` (docs-only, no
  OpenAPI spec)
- [x] Signature-based auth (library ID + API key + expiry hash) as the
  docs describe; pre-signed metadata headers for library/video IDs
- [x] Chunked streaming reads — never buffer the whole file (project
  performance rule)

### 2. CLI surface [4/4]

- [x] `stream video upload --resumable` (or a dedicated subcommand if
  flag semantics get muddy) — reuse the existing create-then-upload
  composite shape
- [x] Retry with backoff on transient failures; resume from the server
  offset after interruption
- [x] Offset/session persistence on disk so a re-run resumes an
  interrupted upload; state file location must be Windows/Linux/macOS
  safe (`std::path::PathBuf`, no Unix-only assumptions)
- [x] Progress bar consistent with the existing PUT upload path

### 3. Per-upload params integration [1/1]

- [x] The per-upload params from
  [[iteration-69-filters-pagination-sweep]] (`jitEnabled`,
  `enabledResolutions`, `enabledOutputCodecs`, `transcribe*`,
  `generate*`, `sourceLanguage`) must also apply on the TUS path
  (metadata headers) — same flags, both transports

### 4. Tests [2/3]

- [x] Unit tests against a wiremock/minimal TUS server: offset resume,
  mid-upload interruption, checksum of assembled payload
- [x] e2e test for the new flag surface (`tests/e2e/` pattern)
- [ ] Live dogfood with a large file + forced interruption; note
  friction in the KB — deferred, requires a real bunny.net account

## Out of scope

- The 13 stale live-streaming endpoints in the Mintlify stream mirror —
  explicitly do-not-implement (gap analysis §1 caveat)
- Parallel/multi-connection chunk upload — resume correctness first

## Acceptance

- [x] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [x] e2e tests cover the new/changed upload commands (`tests/e2e/` pattern)
- [ ] Interrupted-then-resumed upload verified live (dogfooding playbook)
- [x] Help text updated for the resumable upload surface
- [x] `hyalo lint` clean on touched knowledgebase files

## Implementation notes

Shipped:

- `crates/bunny-net-api/src/stream/tus.rs` — TUS 1.0 client (`TusUploader`):
  presigned `AuthorizationSignature` = hex SHA-256 of
  `library_id + api_key + expiration + video_id`; `POST /tusupload` creation
  with `Upload-Metadata` header (per-upload params base64-encoded, mirroring
  the PUT query params), `HEAD` offset probe, chunked `PATCH`
  (`application/offset+octet-stream`) streaming one `chunk_size` window at a
  time via `AsyncReadExt` — never buffers the whole file. `sha2`/`tokio` are
  gated behind the existing `stream` Cargo feature; no `hex` dependency (hex
  encoding is a small local helper).
- `crates/hoppy-cli/src/commands/stream_tus.rs` — orchestration:
  JSON session persistence keyed on `library_id`+abs file path (state file in
  `--state-dir` or a `hoppy-tus` temp subdir, `PathBuf`-based, cross-platform),
  retry with exponential backoff that re-probes the server offset between
  attempts, and a progress bar consistent with the PUT path. Session file is
  removed on success; a server-side-expired session is transparently recreated.
- CLI: `stream video upload --resumable [--chunk-size <bytes>] [--state-dir <dir>]`.
- Tests: 8 API unit + 6 API wiremock e2e (create/offset/gone/full/resume/mismatch),
  6 CLI unit (session filename/dir/roundtrip/garbage) + 3 CLI e2e
  (full run cleans up state; resume-from-offset sends only the tail;
  cross-invocation resume — see review fix below).

**Review fix (post-merge-review, same PR):** the original cut always called
`create_video` before checking for a persisted session, so every re-run of
`--resumable` got a fresh video GUID that could never match the GUID stored
in the previous session file — cross-invocation resume (the headline
behavior) was dead on arrival. Fixed by adding
`stream_tus::find_resumable_session` to look up a still-valid session
(`library_id` + absolute file path + file length) *before* deciding whether
to create a new video; the CLI now reuses the prior video via `get_video`
when a session is found. Added
`stream_video_upload_resumable_second_invocation_reuses_video`, an e2e test
that drives two separate `hoppy` process invocations against the same state
dir and asserts the video-create endpoint fires exactly once — confirmed it
fails against the pre-fix code and passes against the fix.

Deferred (require a live bunny.net account — cannot run unattended):

- Live dogfood of a large file with a forced interruption (Scope §4, third item).
- Acceptance: "interrupted-then-resumed upload verified live". The resume path
  is covered by the offset-3 wiremock and CLI e2e tests, but a real large-file
  interruption over the wire is left for a dogfooding pass.
