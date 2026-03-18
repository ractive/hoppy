---
name: codegen-experiment-progenitor
description: Findings from running cargo progenitor against BunnyCDN API specs (Stream and Edge Scripting) to generate client crates
type: project
---

Progenitor codegen for hoppy-api-stream succeeded after three spec fixes applied to a copy at `specs/stream-fixed.json`.

**Why:** The original `specs/stream.json` has several features that progenitor 0.13.0 cannot handle; the fixed copy is the canonical input for this crate.

**How to apply:** When regenerating `crates/hoppy-api-stream`, always use `specs/stream-fixed.json`, not `specs/stream.json`. Also ensure `RUSTFMT` is set to the nightly rustfmt binary at `~/.rustup/toolchains/nightly-aarch64-apple-darwin/bin/rustfmt` because progenitor uses nightly-only rustfmt options (`wrap_comments`, `normalize_doc_attributes`).

## Fixes applied to stream-fixed.json

1. **Removed `application/octet-stream` request bodies** from:
   - `PUT /library/{libraryId}/videos/{videoId}` (binary video upload)
   - `POST /library/{libraryId}/videos/{videoId}/thumbnail` (binary thumbnail upload)
   These endpoints become stub methods with no body parameter. Binary upload must be handled manually outside the generated client.

2. **Normalized heterogeneous 400 response schemas** to `StatusModel` for:
   - `POST /library/{libraryId}/videos/{videoId}/captions/{srclang}` (was `CaptionValidationModel`)
   - `PUT /library/{libraryId}/videos/{videoId}/outputs/{outputCodecId}` (was `type: string`)

3. **Added `StatusModel` content bodies to all error responses (400-599) that had no body** — 92 responses total. Progenitor panics with `response_types.len() <= 1` assertion when error responses in an endpoint have heterogeneous types (some with body, some without).

## Generation command

```sh
RUSTFMT=~/.rustup/toolchains/nightly-aarch64-apple-darwin/bin/rustfmt \
  cargo progenitor -i specs/stream-fixed.json -o crates/hoppy-api-stream -n hoppy-api-stream -v 0.1.0
```

---

## hoppy-api-compute (Edge Scripting API)

Progenitor codegen succeeded after 11 spec fixes applied to `specs/edge-scripting-patched.json`. Crate compiles cleanly with `cargo check`.

**Generation command:**
```sh
cargo progenitor -i specs/edge-scripting-patched.json -o crates/hoppy-api-compute -n hoppy-api-compute -v 0.1.0
```
(No nightly rustfmt needed — this spec doesn't trigger the nightly-only format options.)

## Fixes applied to edge-scripting-patched.json (11 total)

1. **Bogus path param**: `POST /compute/script/{id}/publish` declared `uuid` as a path parameter but `{uuid}` is not in the URL template. Progenitor panics with "uuid missing from path". Removed the orphaned param.

2. **Multi-2xx responses** (same assertion `response_types.len() <= 1` but for success): `PUT /compute/script/{id}/variables` (200+204 both with body) and `PUT /compute/script/{id}/secrets` (200 with body + 204 without body). Removed the 204 responses.

3. **Mixed error response bodies**: 7 endpoints had some 4xx responses with JSON bodies (`ApiErrorData`) and others without. Same progenitor assertion fires for error responses. Removed the body from the 400 responses to make all error responses body-free. Trade-off: callers lose structured validation error details (error type becomes `Error<()>`).

4. **Nullable path parameter**: `uuid` path param in `POST /compute/script/{id}/publish/{uuid}` had `nullable: true`, causing progenitor to type it as `Option<&str>` which doesn't implement `Display` for path encoding. Removed `nullable`.

## Generated crate stats
- 23 async client methods
- 41 named public types (structs/enums)
- ~4110 lines of generated Rust

## Usability concerns

**Method name verbosity (significant):** Progenitor derives names from `operationId` which follows `EndpointClass_MethodName` pattern:
- `get_edge_script_code_endpoint_get_code` → should be `get_code`
- `edge_script_statistics_endpoint_get_edge_script_statistics_endpoint` (doubled)
- `get_edge_script_active_release_endpoint_get_currently_active_release_endpoint`
- `publish_edge_script_release_endpoint_publish2` (publish2 is opaque)

**Missing error detail:** The `ApiErrorData` type is generated but unused in method signatures (all `Error<()>`) due to fix #3 above.

**Duplicate type EdgeScriptTypes/EdgeScriptTypes2:** Same enum appears twice in spec without deduplication.

**Verdict:** Functional foundation, needs a thin ergonomic wrapper crate with shorter method aliases and restored error types.
