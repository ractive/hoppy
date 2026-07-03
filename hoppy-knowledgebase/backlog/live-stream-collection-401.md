---
title: "stream API: live tests fail with 401 on collection create + library statistics"
type: backlog
date: 2026-05-14
status: resolved
priority: medium
origin: dogfooding-2026-05-14 (iter-33 §5 sweep)
---

# Stream collection lifecycle live test fails with 401

In the 2026-05-14 live-api sweep (test account), the test panicked:

```text
thread 'cli_stream::live_stream_collection_lifecycle' panicked at
crates/hoppy-cli/tests/e2e/cli_stream.rs:1072:9:
collection create failed — stderr: record: updated core/GET_videolibrary_660783.json
{ "error": { "message": "HTTP 401 Unauthorized: ... StatusCode 401" } }
```

The library `660783` was created earlier in the same test (or by
`live_stream_library_lifecycle` which passed), and the test then tries to
POST a collection against it — bunny.net's stream API answers 401.

## Hypotheses to verify

1. ~~The stream API uses a per-library access key, not the account API
   key — and the test's stream client isn't fetching/using it for
   collection ops.~~ **Disproven.** `resolve_stream_client`
   (`crates/hoppy-cli/src/commands/stream.rs`) has fetched the library via
   the Core API and used its per-library `ApiKey` as the Stream `AccessKey`
   since iter-32 — this was never the account API key. Mock e2e tests
   already assert the per-library key is sent for collection ops.
2. ~~The library was deleted (by another test's cleanup) before the
   collection create ran~~. **Disproven.** `live_stream_collection_lifecycle`
   creates its own library and immediately pushes its cleanup before doing
   anything else; there is no other test or cleanup action that could touch
   the same library id, and `--test-threads=1` rules out cross-test races.
3. ~~The test-account stream subscription doesn't include the collection
   feature~~. **Disproven.** Collection create succeeds reliably a few
   seconds later on the same account/library — it isn't gated off.

## Root cause (confirmed 2026-07-03)

**Key propagation delay.** `POST /videolibrary` (Core API) returns the new
library's per-library `ApiKey` immediately, but that key is not yet valid
against the Stream API (`video.bunnycdn.com`) for a short window right
after creation — requests in that window get a bare `401 Unauthorized`
(`{"Success":false,"Message":"Authentication has been denied for this
request.","StatusCode":401}`), not a 404 or a feature-gate error.

Verified empirically by scripting `hoppy stream library create` followed
immediately by `hoppy stream collection create` against 3 fresh libraries
on the test account: **3/3 trials hit a 401 on the very first attempt**,
then succeeded on retry 2-6 seconds later (2 trials succeeded on attempt 2,
1 trial needed attempt 3). This is a genuine short-lived eventual-consistency
window on bunny.net's side — the same class of issue already worked around
for storage zones (`cli_storage.rs` sleeps 5s after zone create before first
use) and DNS zone scans (`cli_dns.rs` polls with backoff), just previously
undocumented for Stream libraries.

**Fix:** `live_stream_collection_lifecycle` now retries the collection-create
step up to 5 times with an increasing backoff (2s, 4s, 6s, 8s) whenever the
response is a 401, via `create_collection_with_retry` in
`crates/hoppy-cli/tests/e2e/cli_stream.rs`. No production client change was
needed — `resolve_stream_client` was already correct. Verified stable across
4 consecutive live runs after the fix (no failures).

## Next steps

- None outstanding for collection create — resolved.
- Worth watching: `live_stream_video_processing_lifecycle` also calls a
  Stream API op (`stream video upload`) immediately after library creation
  and could theoretically hit the same window, but it's gated behind
  `TEST_VIDEO_PATH` and wasn't part of this failure — left as-is for now.
