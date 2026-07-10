---
type: backlog
title: "live_stream_library_lifecycle: extend retry-on-401 to the statistics step"
date: 2026-07-10
status: open
origin: live-API run 2026-07-10 (post iter-66..77)
---

# live_stream_library_lifecycle: extend retry-on-401 to the statistics step

During the 2026-07-10 live-API run (TEST_BUNNY_API_KEY), `stream library
statistics` failed once with HTTP 401 inside `live_stream_library_lifecycle`
(`cli_stream.rs:2023`) and passed on rerun. This is the documented quirk from
[[api/bunny-api-quirks]]: a fresh video library's per-library ApiKey takes
2–6s to become valid on the Stream API. The lifecycle test already retries
on 401 for other Stream calls, but the statistics step asserts immediately.

- [ ] Wrap the statistics step (and audit the other post-create Stream calls
  in the lifecycle) with the existing retry-on-401 helper
