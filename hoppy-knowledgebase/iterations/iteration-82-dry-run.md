---
title: Iter-82 — global --dry-run for mutating operations
type: iteration
date: 2026-08-09
tags:
  - iteration
  - cli
  - dx
  - safety
status: in-progress
branch: iter-82/dry-run
---

# Iter-82 — global `--dry-run` for mutating operations

## Why

From [[development-roadmap]] future iterations: "show what would happen
without executing". Complements the dogfooding safety discipline
([[dogfooding/dogfooding-playbook]]): a user (or LLM) can preview exactly
which request a mutating command would send — method, URL, body — without
touching the account.

## Design

### Interception layer: client `send()`/`execute()`

There is no shared HTTP core — 10 independent execution paths (9 service
clients + `TusUploader`). Each builds the full `reqwest::Request` before
executing, so the block goes there, gated on the existing
`recording::debug::is_mutating(method)` predicate (POST/PUT/PATCH/DELETE —
reliable here because bunny.net uses POST for updates and DELETE-with-body):

- GET/HEAD requests proceed normally, even under `--dry-run`. This is what
  makes composites truthful: `storage upload` may still resolve the zone
  password via `GET /storagezone`, `stream` may resolve the library key,
  `container app delete --cascade` may discover linked pull zones — then the
  first mutating request is blocked.
- A blocked request does NOT return a fake response. `send()` returns a typed
  error `DryRunSkipped { method, url, body }` (new module
  `crates/bunny-net-api/src/dry_run.rs`, not feature-gated, like
  `recording`). Body is captured pre-redacted via `format_debug_body`
  (respects `--reveal`); streaming bodies render as `<streaming body>`.
- `main.rs` detects `DryRunSkipped` on the returned error chain
  (`err.chain()` downcast — robust through `anyhow::Context` wrapping),
  prints the preview, and exits **0**.

### Output

Single formatting chokepoint in `main.rs` (the client never prints the
preview itself, it only carries the data in the error):

- `--format json` → stdout envelope:
  `{"status":"dry-run","method":"POST","url":"…","body":{…}}` (body as
  parsed JSON when possible, else string; omitted when empty). Mirrors the
  `print_mutation_result` envelope contract.
- `--format table|text` → stderr, keeping stdout pipe-clean:
  `[dry-run] Would send: POST https://…` followed by the (redacted) body.
  `[dry-run]` prefix matches the existing convention in
  `commands/stream.rs`.

### Prompts

`--dry-run` implies `--yes`: fold in `main.rs`
(`let yes = cli.yes || cli.dry_run`). Safe because the mutation is blocked
at the client layer regardless (defense in depth) — no need to touch the
three prompt implementations. Precedent already in the tree:
`if !*dry_run && !yes` in `commands/stream.rs`.

### Flag threading

- New global clap arg `--dry-run` on `Cli` (`global = true`, with `///` doc
  comment — `cli_help_completeness.rs` enforces help text).
- Bundle client-construction flags into a `ClientOpts` struct in `auth.rs`
  (`debug`, `dry_run`, `record`, `reveal_secrets`) instead of adding an 8th
  positional bool to ~20 handler signatures. Factories
  (`core_client_with_reveal` & siblings) and the two ad-hoc builders
  (`build_storage_client`, `resolve_stream_client`) take `&ClientOpts`.
  `RedactConfig` stays separate. If the refactor's blast radius explodes,
  plain positional threading is the acceptable fallback — but try
  `ClientOpts` first.
- All 9 clients + `TusUploader` gain `.with_dry_run(bool)`. TUS matters:
  on `--resumable` resume, the first request can be a TUS PATCH from
  `TusUploader`'s own client, bypassing `StreamClient::send`.

### Name collision: `stream video resolutions cleanup --dry-run`

That subcommand already has a **local** `--dry-run` mapping to the
server-side `?dryRun=true` query param (the API returns what *would* be
deleted — strictly more informative than a local preview). clap rejects a
global arg duplicating a local arg name, so:

- Remove the local flag; the global `--dry-run` drives the same behavior
  for this one command (documented exception: a request IS sent, but it is
  non-mutating by API contract).
- `StreamClient::cleanup_resolutions(…, dry_run: true)` must bypass the
  client-level block for exactly this call (private unchecked send path).
- Prompt behavior unchanged (`--dry-run` already skipped it).

### Scope boundaries (documented, not implemented)

- Read-only commands ignore `--dry-run` entirely — including ones with
  local side effects (`storage download`, `logs export`, invoice PDFs write
  local files). Out of scope; file a backlog item if dogfooding says
  otherwise.
- Multi-request composites abort at the **first** blocked mutation and
  preview only that request (`container app delete --cascade` shows the app
  DELETE, not the follow-up pull-zone deletes). Truthful-but-partial beats
  fabricating IDs that don't exist yet. Document in the flag's help text
  and CHANGELOG.
- `container logs` (local syslog listener + bore tunnel): out of scope.

## Tasks

### 1. bunny-net-api: dry-run plumbing

- [ ] New `src/dry_run.rs`: `DryRunSkipped { method, url, body: Option<String> }`
      error type (implements `std::error::Error` + `Display`) and a shared
      `check_dry_run(&reqwest::Request, dry_run: bool, reveal: bool) -> Result<(), DryRunSkipped>`
      helper reusing `is_mutating` + `format_debug_body`
- [ ] `with_dry_run(bool)` builder + field on all 9 service clients
- [ ] Intercept in all 9 `send()`/`execute()` fns (core, shield, containers,
      stream, storage, database, compute, logging, origin-errors) — after
      `rb.build()`, before `http.execute()`; debug logging still fires first
- [ ] `TusUploader`: `with_dry_run` + intercept its three request sites
      (create POST / offset HEAD passes / upload PATCH)
- [ ] `StreamClient::cleanup_resolutions`: exempt the `?dryRun=true` call
      from the block via a private unchecked send
- [ ] Unit tests (wiremock, per-crate e2e targets): mutating call under
      dry_run → `DryRunSkipped` error, zero requests received; GET under
      dry_run → executes normally; TUS create blocked; cleanup_resolutions
      exemption still sends

### 2. hoppy-cli: flag, threading, output

- [ ] `cli.rs`: global `--dry-run` flag with doc comment ("Preview mutating
      API calls without sending them; read-only requests still execute;
      implies --yes")
- [ ] Remove local `dry_run` from `StreamCleanupResolutions`; rewire the
      cleanup handler arm to the global flag
- [ ] `auth.rs`: `ClientOpts` struct; migrate factories + call sites +
      `build_storage_client` + `resolve_stream_client`
- [ ] `main.rs`: fold `yes = cli.yes || cli.dry_run`; after `run()` error,
      walk `err.chain()` for `DryRunSkipped`; print preview (json → stdout
      envelope, table/text → stderr `[dry-run]` lines); exit 0
- [ ] Suppress progress bars under dry-run where cheap (storage/stream
      upload arms) — polish, don't chase every spinner

### 3. e2e tests (crates/hoppy-cli/tests/e2e/)

- [ ] `pull-zone create --dry-run`: exit 0, zero requests to mock, stderr
      has `[dry-run]` + method/URL, body shown redacted
- [ ] `pull-zone delete --dry-run` WITHOUT `--yes`, no stdin: exits 0
      without hanging (prompt skipped), zero requests
- [ ] `--dry-run --format json`: snapshot the stdout envelope
- [ ] `storage upload --dry-run`: mock the `GET /storagezone` preflight
      (expect 1), assert zero PUTs (filter `received_requests()` by method)
- [ ] One delete-path test each for dns record / shield / container app /
      db — mechanical clones of the pull-zone pattern
- [ ] `stream video resolutions cleanup --dry-run`: request IS sent with
      `dryRun=true` query param (the documented exception)
- [ ] `--dry-run --reveal`: body shown unredacted
- [ ] Read-only command with `--dry-run` (e.g. `pull-zone list`): request
      still sent, normal output

### 4. Docs + housekeeping

- [ ] CHANGELOG entry (Added: global `--dry-run`; Changed: stream cleanup
      local flag folded into the global one)
- [ ] README global-flags section (if flags are listed there)
- [ ] KB: `cli/command-tree.md` top-level flags; decision-log entry
      (interception layer + error-abort + implies-yes + cleanup exception);
      prune the `--dry-run` bullet from [[development-roadmap]]
- [ ] `cargo fmt` && `cargo clippy --workspace --all-targets -- -D warnings`
      && `cargo test --workspace --quiet` — all green

### 5. Dogfood

- [ ] `cargo build --release`; against the test account
      (TEST_BUNNY_API_KEY): `pull-zone create --dry-run`,
      `pull-zone delete --dry-run`, `storage upload --dry-run`,
      `db create --dry-run` — verify nothing appears in the account; file
      friction as backlog items

## Acceptance

- [ ] Every mutating command under `--dry-run` sends zero mutating requests
      (e2e-asserted for representative commands across services), prints a
      method/URL/body preview, and exits 0
- [ ] `--dry-run` skips confirmation prompts without hanging on stdin
- [ ] Read-only commands behave identically with and without `--dry-run`
- [ ] `stream video resolutions cleanup --dry-run` still performs the
      server-side preview (`?dryRun=true`)
- [ ] Secrets in previewed bodies are redacted unless `--reveal`
- [ ] All three quality gates pass

## Related

- [[development-roadmap]]
- [[dogfooding/dogfooding-playbook]]
- [[decision-log]]
- [[cli/command-tree]]
