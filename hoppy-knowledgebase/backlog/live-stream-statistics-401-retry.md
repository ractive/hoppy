---
type: backlog
title: "live_stream_library_lifecycle: extend retry-on-401 to the statistics step"
date: 2026-07-10
status: resolved
origin: live-API run 2026-07-10 (post iter-66..77)
---

# live_stream_library_lifecycle: extend retry-on-401 to the statistics step

During the 2026-07-10 live-API run (TEST_BUNNY_API_KEY), `stream library
statistics` failed once with HTTP 401 inside `live_stream_library_lifecycle`
(`cli_stream.rs:2023`) and passed on rerun. This is the documented quirk from
[[api/bunny-api-quirks]]: a fresh video library's per-library ApiKey takes
2–6s to become valid on the Stream API. The lifecycle test already retries
on 401 for other Stream calls, but the statistics step asserts immediately.

- [x] Wrap the statistics step (and audit the other post-create Stream calls
  in the lifecycle) with the existing retry-on-401 helper

## Resolution (2026-08-09, iter-81)

The one-off `create_collection_with_retry` was generalized into
`support::hoppy_live_json_with_401_retry` (same 5-attempt linear-backoff
policy, doc-comment scoped to Stream-API per-library-key calls). Wrapped:
the library statistics step plus all five collection Stream-API calls.
`drm-statistics`/`transcribing-statistics` deliberately left bare — they
hit the core API with the account key and don't suffer the propagation
quirk. `live_stream_library_lifecycle` and
`live_stream_collection_lifecycle` green on the live API 2026-08-09.
