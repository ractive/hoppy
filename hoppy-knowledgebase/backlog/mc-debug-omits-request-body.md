---
title: Magic Containers client --debug omits the request body
type: backlog
date: 2026-08-09
tags:
  - backlog
  - containers
  - debug
  - dx
status: resolved
priority: low
---

# Magic Containers client `--debug` omits the request body

## Observed (2026-08-09 dogfood, iter-81)

`hoppy --debug container log-forwarding create ...` prints the request
URL (`>> POST https://api.bunny.net/mc/log/forwarding`) and the response
(`<< 400 Bad Request`), but no `>>>` request-body lines. The core-platform
client prints the body fine (verified same session with
`--debug storage-zone create`, which shows `>>> {"Name": ..., "Region": ...}`).

The May 2026 repro in [[backlog/log-forwarding-create-empty-400]] did show
`>>>` for the same command, so this is a regression in the mc client's
debug plumbing (or the mc client was switched to a code path that never
gained it when [[backlog/debug-flag-omits-request-body]] was fixed).

## Why it matters

Debugging the still-open empty-400 log-forwarding issue requires seeing
exactly what we send. Low priority otherwise.

## Want

- `--debug` on mc-client requests prints the serialized request body,
  same as the core client.

## Resolution (2026-08-09, iter-81)

Fixed for every domain client, not just Magic Containers.

`format_debug_body` (JSON pretty-print + secret-field redaction) and the new
`print_debug_request_body` helper (mutating-method check + streaming-body
fallback) moved out of `bunny-net-api::core::client` into
`bunny-net-api::recording::debug` — a shared, non-feature-gated module
(`recording` already sits under every domain feature via `core = ["recording"]`,
so no `Cargo.toml` feature-gate changes were needed). `core::client` re-exports
`format_debug_body` at its old path for backward compatibility.

Every client whose surface has mutating (POST/PUT/PATCH/DELETE) endpoints now
prints the `>>>` request-body line and redacts the response-body print too
(containers, stream, database, compute, storage, shield). `shield` was already
printing `>>> METHOD URL` but had never gained a request-body print or
redacted response formatting at all — both are now aligned with core.
`logging` and `origin_errors` are read-only (no mutating endpoints) and were
left untouched. Storage uploads are streaming bodies, so they print
`>>> <streaming body>` rather than attempting to buffer/format them.

Each client gained a `debug_reveal_secrets: bool` field and a
`with_debug_reveal_secrets` builder, mirroring `CoreClient`. On the CLI side,
`hoppy-cli/src/auth.rs` gained `database_client_with_reveal`,
`shield_client_with_reveal`, `compute_client_with_reveal`, and
`containers_client_with_reveal` (the old non-reveal wrappers were removed —
every call site now threads the flag). The global `--reveal` flag
(`redact_cfg.reveal_all` / a threaded `reveal: bool`) now reaches every
mutating client constructor: `container.rs`, `script.rs` (compute),
`shield.rs`, `storage.rs`, and `stream.rs` each had `reveal` threaded through
their handler call chains down to the client-builder call site.

Tests: `format_debug_body` unit tests moved to
`bunny-net-api/src/recording/debug.rs`. Added
`container_log_forwarding_create_debug_prints_and_redacts_body` and
`container_log_forwarding_create_debug_reveal_shows_body` (wiremock e2e,
`crates/hoppy-cli/tests/e2e/cli_container.rs`) asserting a mutating
`container log-forwarding create --token ...` command emits a `>>>` body line
with the token redacted by default and revealed with `--reveal`.

Gates: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace --quiet` (all green), and
`cargo clippy -p hoppy-cli --tests --features live-api -- -D warnings`
(compile-only check, also green).
