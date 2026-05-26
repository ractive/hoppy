---
title: Iter-38 — CDN log forwarding + dogfooding polish
type: iteration
date: 2026-05-15
tags:
  - iteration
  - pull-zone
  - log-forwarding
  - cli
  - dx
  - cleanup
status: completed
branch: iter-38/cdn-log-forwarding-and-polish
---

# Iter-38 — CDN log forwarding + dogfooding polish

## Why

The 2026-05-15 dogfooding round (after iter-37 landed) surfaced a
handful of fixable papercuts plus one *missing feature* that's strictly
more useful than the broken one we already ship:

- **Missing**: `pull-zone update` does not expose the CDN log-forwarding
  fields documented in `specs/core-platform.json` lines 6291–6325.
  This is the standard "stream CDN access logs to my SIEM" use case
  and currently requires the dashboard.
- **Broken upstream**: `container log-forwarding create` returns
  empty-body 400 for every payload variant tried — likely a feature
  gate or undocumented required field. Tracked in
  [[../backlog/log-forwarding-create-empty-400]]; out of scope here
  except to mark the command as known-broken in the README.
- **Papercuts**: a few small but visible CLI consistency issues that
  bit during the round.

Pulling these together into one iteration because they share a theme
(log forwarding + CLI consistency) and a code path (`pull-zone update`,
storage-zone create JSON, the container LF CLI surface).

## Target shape

After this iteration:
- `hoppy pull-zone update` accepts seven new flags to configure CDN
  log forwarding + permanent logging.
- `hoppy pull-zone get --format json` includes the log-forwarding
  fields in its output (verify; may already pass through).
- The empty-`storage-zone create --format json` password leak is
  fixed (`"Password": "string"` → real password or explicit
  redaction marker).
- `container log-forwarding delete` accepts both `--app-id` and
  `--id` (alias) for consistency with other delete commands.
- `container log-forwarding get` returns 200 + `null` instead of
  404 when no config exists (or surfaces a typed `NotConfigured`
  error variant that scripts can match on).
- The `container logs` cleanup path tolerates 404 on the final
  `log-forwarding delete` (it already may; verify).
- The dogfooding playbook flags `hoppy container logs` as
  known-broken pending the upstream investigation.

## Scope

### 1. CDN log-forwarding flags on `pull-zone update` [0/7]

Source: `crates/hoppy-cli/src/commands/pull_zone.rs` for the CLI surface,
`crates/bunny-net-api/src/core/types.rs` for the `PullZoneUpdateRequest`
struct (or whatever it is named today — verify).

Per `specs/core-platform.json` `PullZoneUpdateRequest`:

- [x] Add `--log-forwarding-enabled <bool>` flag.
- [x] Add `--log-forwarding-hostname <host>` flag.
- [x] Add `--log-forwarding-port <u16>` flag (validate range).
- [x] Add `--log-forwarding-token <token>` flag (treat as secret —
      respect `--reveal` semantics; do not echo in confirmation
      output unless `--reveal` is set).
- [x] Add `--log-forwarding-protocol <udp|tcp>` flag, mapped to the
      spec's `PullZoneLogForwarderProtocolType` enum.
- [x] Add `--logging-save-to-storage <bool>` flag.
- [x] Add `--logging-storage-zone-id <id>` flag.

All seven must round-trip via the same `pull-zone get` (proves the
update landed). Add one wiremock test per flag and one shape-first
serde-default test for the response.

### 2. Verify `pull-zone get` already surfaces these fields [0/2]

- [x] Run `pull-zone get --format json` against a real pull zone with
      log forwarding enabled (via dashboard or via §1 above) and
      confirm the seven fields appear in the JSON output.
- [x] If any field is filtered/redacted by the current `PullZone`
      type, un-filter (or add `--reveal` handling for the token
      specifically — it's a credential to a *third-party* syslog
      endpoint, so the redaction-by-default discipline applies).

### 3. Storage-zone create JSON password [0/2]

See [[../backlog/sz-create-json-password-string-literal]].

- [x] Identify where `Password: "string"` is coming from. Hypothesis:
      the create response struct has a `#[serde(default)]` field that
      falls back to the literal `"string"` from an OpenAPI placeholder,
      or hoppy is overwriting the value with the spec placeholder
      somewhere. Trace and fix.
- [x] After the fix: `storage-zone create --format json` returns the
      real password (unredacted on create — consistent with `get
      --reveal`) so scripts can capture it on first creation without
      a follow-up call.

### 4. `container log-forwarding` CLI consistency [0/3]

See [[../backlog/log-forwarding-create-empty-400]] §"Smaller related
issues".

- [x] `container log-forwarding delete` accepts `--id <id>` as an
      alias for `--app-id` (since LF configs are 1:1 with apps, the
      two are interchangeable from the user's POV).
- [x] `container log-forwarding get` for an app with no config:
      return 200 + `null` (or an explicit `not_configured: true`
      flag) instead of bubbling the upstream 404. The current 404
      forces every shell pipeline to check exit codes instead of
      reading JSON.
- [x] Verify `container logs` cleanup path is 404-tolerant on the
      final delete (it should be, since the create-then-delete is
      always racing tear-down).

### 5. Document `container logs` as known-broken [0/2]

See [[../backlog/log-forwarding-create-empty-400]] for the upstream
investigation.

- [x] Add a `> [!warning]` block at the top of the
      `container logs --help` long description: "as of 2026-05-15
      this command may fail at the log-forwarding-create step with
      an empty-body 400 from the bunny.net API. Tracking in
      backlog/log-forwarding-create-empty-400.md."
- [x] Same warning in `hoppy-knowledgebase/dogfooding/dogfooding-playbook.md`
      under any section that references `container logs`.

### 6. Verification [0/3]

- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings`
      clean.
- [x] `cargo test --workspace --quiet` clean.
- [x] Live dogfood: enable CDN log forwarding on a `hoppy-test-` pull
      zone, point it at a netcat listener on a public VPS or
      `--logging-save-to-storage` to a `hoppy-test-` storage zone,
      and confirm an end-to-end log line lands within Bunny's
      documented delay. Tear down. Commit any fixture drift from the
      live run.

## Out of scope

- **Fixing `container log-forwarding create`'s upstream 400.** Open
  a support ticket with bunny.net first; until they respond with
  what the body needs, any client-side fix is guesswork.
  Re-attack in a follow-up iteration once we have a server-side
  reason string.
- **A `pull-zone logs tail` streaming command.** The CDN pushes logs
  to a configured syslog endpoint or storage zone; it doesn't expose
  a pull/poll API. A `tail` would need its own bore/tunnel plumbing
  similar to `container logs`. Worthwhile, but separate iteration.
- **JSON output casing normalisation.** See
  [[../backlog/json-output-casing-inconsistency]] — three different
  casing conventions across surfaces. Larger DX-rewrite scope; not
  bundled here.
- **`cleanup.sh --yes` implementation.** See
  [[../backlog/leaked-test-resources-cleanup-script]] — still a
  skeleton. The 19 leaked resources from prior live-api runs are
  costing money but cleaning them is its own iteration.
- **Terraform deliverables.** See
  [[../research/terraform-provider-ideas]] — strategic, not tactical.

## Risks and mitigations

- **CDN log forwarding requires a real reachable receiver.** Without
  one, §6's verification step is just "the request returned 200".
  Mitigation: use `--logging-save-to-storage` as the verification
  receiver — it writes log files into a storage zone we already
  control, no external infra needed.
- **`PullZoneLogForwarderProtocolType` enum casing.** The spec
  declares the enum but the wire shape may be PascalCase / integer.
  Mitigation: derive from any existing fixture and round-trip
  before shipping; mirror the existing Protocol-enum aliasing
  pattern in `containers/types.rs` if the API is inconsistent.
- **Storage-zone password fix may be a serde-level behaviour
  change.** Touch with care — other commands consume the same type.
  Mitigation: add a unit test that pins the deserialisation
  behaviour before changing the field.

## Acceptance

- All seven `pull-zone update` log-forwarding flags work and
  round-trip via `pull-zone get`.
- `storage-zone create --format json` returns a usable password.
- `container log-forwarding {delete,get}` are scripting-friendly.
- `cargo fmt && cargo clippy ... && cargo test --workspace --quiet`
  green.
- One live dogfood run captured in the PR with a fixture-diff (or
  an explicit "no drift" note in the commit message).
- README + dogfooding playbook warn that `container logs` is
  upstream-broken pending the support ticket.

## Related

- [[../backlog/pull-zone-log-forwarding-fields-missing]] — §1 spec.
- [[../backlog/sz-create-json-password-string-literal]] — §3 spec.
- [[../backlog/log-forwarding-create-empty-400]] — §4–§5 context;
  the broken upstream is tracked there, not "fixed" here.
- [[../dogfooding/dogfooding-playbook]] — §5 docs target.
- [[iteration-37-cli-snapshot-filters]] — prior iteration; closed
  the value-coupled-CLI-test loop that this round depended on.

## Tasks

- [x] Add CDN log-forwarding flags to `pull-zone update` (§1).
- [x] Verify and adjust `pull-zone get` JSON output (§2).
- [x] Fix `storage-zone create --format json` password (§3).
- [x] Polish `container log-forwarding` CLI consistency (§4).
- [x] Document `container logs` as known-broken (§5).
- [x] Run quality gates and live dogfood verification (§6).
