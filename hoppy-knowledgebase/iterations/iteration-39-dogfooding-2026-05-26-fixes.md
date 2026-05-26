---
title: Iter-39 — dogfooding 2026-05-26 fixes
type: iteration
date: 2026-05-26
tags:
  - iteration
  - bugfix
  - cli
  - pull-zone
  - debug
status: in-progress
branch: iter-39/dogfooding-2026-05-26-fixes
---

# Iter-39 — dogfooding 2026-05-26 fixes

## Why

Post-iter-38 dogfooding round against the live test account surfaced three
items that bit during the session. None are blockers, but the silent
no-op on log-forwarding updates is a real correctness issue and the
others were active friction while triaging it.

## Scope

### 1. Log-forwarding hostname silent no-op
Source: [[../backlog/log-forwarding-hostname-silent-noop]]

When `LogForwardingEnabled` is `false` on a pull zone, the bunny.net API
silently ignores `LogForwardingHostname` (and likely `Port` / `Protocol`
/ `Token`) updates — returns 200 OK with the field unchanged. The CLI
forwards the response unchanged, so the user thinks the update landed.

- [x] Reproduce against a live zone with `LogForwardingEnabled=false` —
      confirm the exact set of sub-fields the API silently drops.
- [x] In `crates/hoppy-cli/src/commands/pull_zone.rs` (update path),
      detect the case before the request: if any of
      `--log-forwarding-{hostname,port,protocol,token}` are passed AND
      `--log-forwarding-enabled` is **not** being set to `true` AND the
      current zone has `LogForwardingEnabled=false`, refuse with a clear
      error and a hint:
      `error: log-forwarding fields cannot be updated while disabled
       hint: pass --log-forwarding-enabled true to enable and update in one call`
- [x] Alternative (or additional) safety net: post-validate the response
      and warn on stderr when any passed log-forwarding field is not
      reflected in the response body.
- [x] e2e test: update with hostname-only on a disabled-LFE fixture
      should error out (or warn) — not return success.
- [x] Update [[../dogfooding/dogfooding-playbook]] if a new pattern
      emerges from the fix.

### 2. `pull-zone get` table is too wide
Source: [[../backlog/pull-zone-get-table-too-wide]]

`hoppy pull-zone get --id <id>` renders 11 columns side-by-side — wraps
unreadably on a 120-col terminal. Single-resource gets should pivot to a
vertical `Field / Value` table like `hoppy auth check`.

- [x] In `crates/hoppy-cli/src/commands/pull_zone.rs`, replace the
      horizontal `tabled` render in the `get` handler with a
      `Field / Value` table. Keep the list handler's wide layout.
- [x] Decide which fields appear at top (Id, Name, Origin URL, Enabled,
      Suspended, Bandwidth Used / Limit, Hostnames, …) — match the JSON
      key order or curate a sensible head-of-list.
- [x] Audit other single-resource gets that may have the same shape:
      `storage-zone get`, `stream library get`, `container app get`,
      `dns zone get`, `shield zone get`. Pivot any that are >~6 columns
      wide.
- [x] Refresh affected e2e snapshot tests (the new shape will redraw).
      Keep them drift-tolerant per the iter-37 playbook.
- [x] Dogfooding pass: re-run the five get commands and confirm output
      fits on a standard terminal.

### 3. `--debug` shows response but not request body
Source: [[../backlog/debug-flag-omits-request-body]]

Today `--debug` prints `>> METHOD URL`, `<< status`, and `<<< response`.
The request body — the most useful piece for diagnosing "why didn't my
update stick?" — is missing. The log-forwarding bug above would have
been faster to localise (CLI bug vs API bug) with the body visible.

- [x] In the HTTP client (probably `crates/bunny-net-api/src/.../client.rs`
      where `--debug` logging is wired in), add a `>>> <body>` line for
      requests with bodies (POST/PUT/PATCH/DELETE).
- [x] Apply the existing secret-redaction logic (the same one that
      redacts `LogForwardingToken` in responses) to the request body so
      `--debug` doesn't leak secrets in terminal output. Honour `--reveal`.
- [x] If the body is JSON, pretty-print it with the same compact-multi-line
      style as response output. If it's a stream/large, truncate with a
      `… (N bytes total)` tail.
- [x] Unit test: a debug-logged request with a `LogForwardingToken` is
      printed as `"<set, length=N>"`; with `--reveal`, the real value
      appears.

## Out of scope

- The bunny.net API's silent-ignore behaviour itself (upstream concern).
- Restructuring the broader `--debug` output format beyond adding the
  missing request-body line.
- Storage-zone `get` pivoting if it's already vertical — only fix the
  ones that are actually wide.

## Acceptance

- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings &&
      cargo test --workspace --quiet` clean.
- [x] Dogfooding repro of all three bugs against the live test account
      shows the fixed behaviour:
      - hostname-only update on disabled-LFE zone errors (or warns).
      - `pull-zone get` fits on one screen.
      - `--debug` shows request body, with `LogForwardingToken` redacted
        unless `--reveal`.
- [x] All three backlog items closed (`status=resolved`) with a link to
      this iteration.

## Related

- [[../backlog/log-forwarding-hostname-silent-noop]]
- [[../backlog/pull-zone-get-table-too-wide]]
- [[../backlog/debug-flag-omits-request-body]]
- Dogfooding round: 2026-05-26 (post-iter-38).
