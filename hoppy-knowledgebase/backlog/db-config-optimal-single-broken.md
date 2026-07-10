---
title: db config optimal-single returns "missing field cdn_server_token"
type: backlog
date: 2026-05-10
status: completed
priority: low
origin: dogfooding-2026-05-10
resolved-by: iteration-66-spec-refresh-drift-fixes
---

# `db config optimal-single` looks broken

```sh
hoppy db config optimal-single
# Error: HTTP 400 Bad Request: Failed to deserialize query string:
# missing field `cdn_server_token`

```

Either:

- The hoppy client is calling the wrong endpoint (the error reads like a
  CDN path, not a DB path).
- The endpoint requires a parameter we don't expose.
- The endpoint is gated server-side and wasn't truly ready when iter-20
  shipped.

`db v2` is already labelled "(gated; some are broken upstream)" in the
top-level help, so this might be in the same bucket. Decide whether to:

1. Hide it behind the same gate as `db v2`, or
2. Fix the call signature.

Either way, drop the user a clearer error than the deserialize complaint.

## Resolution (iter-66)

Fixed in [[iterations/iteration-66-spec-refresh-drift-fixes]]. The spec marks
`cdn_server_token` as a **required query parameter** on both `/v1/config/optimal`
and `/v1/config/optimal_single`. hoppy now:

- Sends `cdn_server_token` on `get_optimal` and `get_optimal_single`.
- Exposes `--cdn-server-token` on `db config optimal` and un-hid
  `db config optimal-single` (was gated behind a stub error).
