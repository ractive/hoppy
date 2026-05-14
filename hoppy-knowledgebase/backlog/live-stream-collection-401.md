---
title: "live_stream_collection_lifecycle returns 401 on collection create"
type: backlog
date: 2026-05-14
status: planned
priority: medium
origin: dogfooding-2026-05-14 (iter-33 §5 sweep)
---

# Stream collection lifecycle live test fails with 401

In the 2026-05-14 live-api sweep (test account), the test panicked:

```
thread 'cli_stream::live_stream_collection_lifecycle' panicked at
crates/hoppy-cli/tests/e2e/cli_stream.rs:1072:9:
collection create failed — stderr: record: updated core/GET_videolibrary_660783.json
{ "error": { "message": "HTTP 401 Unauthorized: ... StatusCode 401" } }
```

The library `660783` was created earlier in the same test (or by
`live_stream_library_lifecycle` which passed), and the test then tries to
POST a collection against it — bunny.net's stream API answers 401.

## Hypotheses to verify

1. The stream API uses a per-library access key, not the account API key
   — and the test's stream client isn't fetching/using it for
   collection ops. The library lifecycle test may succeed because its
   account-key operations are different from collection ops.
2. The library was deleted (by another test's cleanup) before the
   collection create ran, and the API returns 401 instead of 404 for the
   "missing library" case.
3. The test-account stream subscription doesn't include the collection
   feature; the 401 is a feature-gate disguised as an auth error.

## Next steps

- Reproduce in isolation: `cargo test --workspace --features live-api -- --test-threads=1 cli_stream::live_stream_collection_lifecycle`.
- Inspect the request: does the client send `AccessKey` from the
  library, or just the global `BUNNY_API_KEY`?
- If (1), fix the stream client to use the per-library key for
  collection ops and add a regression test.
