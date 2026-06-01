---
title: >-
  Stream library `ApiKey`/`ReadOnlyApiKey` are unreachable from the CLI (even
  with `--reveal`)
type: backlog
date: 2026-05-31
status: completed
priority: high
origin: dogfooding-2026-05-31
tags:
  - stream
  - secrets
  - dx
  - blocker
resolved_in: iter-52
---

# Stream `ApiKey` and `ReadOnlyApiKey` cannot be retrieved at all

`crates/bunny-net-api/src/core/types.rs` (around L2820–L2840) marks both
secret fields with `#[serde(default, skip_serializing)]`:

```rust
#[serde(default, skip_serializing)]
pub api_key: String,
#[serde(default, skip_serializing)]
pub read_only_api_key: String,
```

…with a comment saying "to prevent accidental exposure". As a result:

- `hoppy stream library get --id <N> --format json` omits both fields.
- `hoppy stream library get --id <N> --format json --reveal` **also** omits
  both fields. `--reveal` cannot reintroduce a `skip_serializing` field.
- `hoppy stream library create …` returns a response that hoppy parses
  but never echoes back the new library's keys.

The user has *no way* to obtain the ApiKey from hoppy. The ApiKey is needed
to call the Stream API (uploading videos, listing collections, etc.) —
so a hoppy user creating a stream library has to go to the bunny.net
dashboard to copy the key by hand.

## Repro

```sh
LIB_ID=673669
hoppy stream library get --id $LIB_ID --format json --reveal | jq 'keys'
# [
#   "AllowDirectPlay",
#   "DateCreated",
#   ...
#   "VideoCount"
# ]
# — no "ApiKey", no "ReadOnlyApiKey"
```

Confirmed via `--debug` that the API *does* return both keys; they make
it as far as the model, then get dropped by `skip_serializing`.

## Why this is a real bug, not just paranoia

- The `--reveal` flag is supposed to be the explicit-opt-in escape hatch
  for "yes I know it's a secret, give it to me". `skip_serializing` makes
  that flag a lie for these two fields.
- Every other secret hoppy redacts (env var values for shield/container,
  storage zone passwords, log-forwarding tokens) is opt-in revealable.
- A library created via `hoppy stream library create` is effectively
  orphaned from the CLI — the user has to open the dashboard to use it.

## Suggested fix

1. Remove `skip_serializing` from `api_key` and `read_only_api_key`.
2. Add the same reveal-gated redaction the rest of hoppy uses (mask with
   `***` or `<redacted>` by default; show the real value when
   `--reveal` is set or when the field is named in `--reveal-env`-style).
3. Add an e2e snapshot test that asserts:
   - default: keys appear as the redaction marker
   - `--reveal`: keys appear as the real string
   - `--format text`: same gating

## Related

- [[json-output-casing-inconsistency]]
- [[debug-flag-omits-request-body]]
