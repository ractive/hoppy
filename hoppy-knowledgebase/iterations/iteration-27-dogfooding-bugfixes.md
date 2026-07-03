---
title: Iter-27 — fix bugs surfaced by dogfooding 2026-05-10
type: iteration
date: 2026-05-10
tags:
  - iteration
  - bugfix
  - dogfooding
status: completed
branch: iter-27/dogfooding-bugfixes
---

# Iter-27 — bug fixes from dogfooding 2026-05-10

Five real bugs surfaced during the 2026-05-10 dogfooding round. None overlap
with each other; all are small, localized fixes in different crates / CLI
modules. Bundling them so one PR ships the round's correctness work.

## Scope

### 1. `container logs` clap panic — CRITICAL
Source: [[backlog/container-logs-clap-panic]]

`hoppy container logs --app-id <id>` panics with
`Mismatch between definition and access of 'format'` before doing any work.
Iter-24's headline feature is unusable.

- [x] Reproduce locally with `cargo run -- container logs --app-id foo --tunnel none`
- [x] Pick a fix path: rename the per-command field (e.g. `tail_format`) and
      a different flag, OR drop the per-command override and validate the
      global `--format` value at the start of the handler. Latter keeps the
      surface clean.
- [x] Add a CLI parse test that constructs `Cli::parse_from(["hoppy",
      "container", "logs", "--app-id", "x"])` and asserts no panic.
- [x] Smoke-test against the test account once the build is green
      (`hoppy container logs --app-id <id> --tunnel none`).

### 2. `dns zone scan results` panics on pending scan

Source: [[backlog/dns-scan-results-null-records]]

API returns `"Records": null` while the scan is in flight; hoppy's
`Vec<Record>` deserialise blows up.

- [x] In `crates/bunny-api-core` (the DNS zone scan response type), make
      `records` tolerate null — either `Option<Vec<…>>` with `serde(default)`
      or a `deserialize_with` helper that turns null → empty vec.
- [x] Unit test with a fixture body for the in-progress shape.
- [x] Live-api e2e: `scan start` then *immediately* `scan results` (no
      sleep) and assert no error.

### 3. `pull-zone update --optimizer-classes` rejected by API

Source: [[backlog/optimizer-classes-rejected]]

Iter-26's optimizer-classes flag always returns 400 `model.invalid`.
Likely the bunny.net API expects a string-encoded JSON map on the request
side (matching the response side, where the deserializer already tolerates
"either string or array"), and the CLI is sending a raw object.

- [x] Confirm wire format with bunny docs / dashboard network tab.
- [x] If the API wants `"{...}"` (JSON string, not object), encode
      client-side in `src/commands/pull_zone.rs:232` so the user still
      passes a normal `--optimizer-classes '{"thumb":"..."}'`.
- [x] Live-api e2e that round-trips one class through update → get and
      asserts equality.

### 4. Shield error responses surface as "error 0: unknown"

Source: [[backlog/shield-api-error-mapping]]

Shield API errors are wrapped in
`{"error": {"statusCode", "errorKey", "message"}, "data": null}`. The
hoppy mapper looks at the top level only and prints `Shield API error 0:
unknown`, dropping every useful field.

- [x] In `crates/bunny-api-shield`, add an error-envelope type that matches
      the nested shape and use it in the error mapper.
- [x] Surface `statusCode` (not `0`) and the `errorKey: message` pair.
- [x] Apply uniformly: `bot-detection`, `upload-scanning`, `api-guardian`,
      `event-logs`, `waf`, `rate-limit`, `access-list`.
- [x] Sanity-check by hitting `shield api-guardian get` on a zone without
      a config — should report 404 + the API's real message.

### 5. `db config optimal-single` returns nonsense error

Source: [[backlog/db-config-optimal-single-broken]]

The endpoint replies with
`Failed to deserialize query string: missing field 'cdn_server_token'`,
which reads like the wrong route entirely.

- [x] Verify whether hoppy is calling the correct path (the error reads
      like a CDN endpoint, not a DB endpoint).
- [x] If the endpoint is server-side gated/broken, hide it behind the same
      gate as `db v2` (already labelled "(gated; some are broken upstream)").
- [x] Either way, the user should see a clean message, not a deserialise
      complaint.

## Out of scope

UX/consistency cleanup (flag names, numeric enums, date formats, output
formatting) is bundled into iter-28 — see
[[iteration-28-dogfooding-ux-polish]].

## Acceptance

- [x] All five bugs have a regression test (unit or e2e) and pass on CI.
- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings &&
      cargo test --workspace --quiet` clean.
- [x] One follow-up dogfooding round confirms each command no longer fails
      the way it did on 2026-05-10.

## Related

- [[dogfooding/dogfooding-playbook]]
- [[iteration-28-dogfooding-ux-polish]]
