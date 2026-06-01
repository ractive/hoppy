---
title: Dogfooding Playbook — safe real-API testing
type: docs
date: 2026-05-09
status: active
tags: [dogfooding, qa, bunny-net-api, safety]
---

# Dogfooding playbook

This playbook lets you run hoppy against a real bunny.net account without destroying anything that is already there. Follow it before/after every iteration that touches an API surface.

## Pre-flight: pick the account

Two accepted modes:

1. **Dedicated test account (preferred)** — register a separate bunny.net account, set a low billing alert, never use it for production. All resources created here can be wiped without consequence.
2. **Shared account (fallback)** — only acceptable if every resource you create is prefixed with `hoppy-test-` (see "Naming prefix" below) so cleanup is grep-able. Never run a destructive op without first listing what matches the prefix.

Authenticate by setting the `BUNNY_API_KEY` environment variable to your bunny.net account API key (see [bunny.net dashboard → Account Settings → API](https://dash.bunny.net/account/settings)). Validate the key with `hoppy auth check` before running any other command — it returns a 200 OK on success or a clear error otherwise. There is no `hoppy auth login` and no on-disk config file: the env var is the only authentication surface.

## Naming prefix

Every resource hoppy creates during dogfooding **must** start with `hoppy-test-`. Examples:

- pull zone: `hoppy-test-cdn-2026-05-09`
- storage zone: `hoppy-test-storage-roundtrip`
- DNS zone: `hoppy-test.example.com` (use a real domain you own that you can safely use for testing)
- container app: `hoppy-test-app-smoke`

Why: the cleanup script greps for `hoppy-test-` and refuses to touch anything that doesn't match. If you forget the prefix, cleanup will leak the resource — you'll have to remove it by hand from the dashboard.

## The safe loop

```
1. Read-only smoke test:    hoppy <noun> list                         # confirm auth + no surprises
2. Cleanup script first:    hoppy-knowledgebase/dogfooding/cleanup.sh # idempotent, scoped to prefix
3. Build:                   cargo build --release
4. Use:                     ./target/release/hoppy <command>          # always with hoppy-test- prefix
5. Note friction:           hoppy-knowledgebase/backlog/<short>.md    # one item per friction point
6. Cleanup after:           hoppy-knowledgebase/dogfooding/cleanup.sh
7. Verify in dashboard:     visit dash.bunny.net and confirm no orphans remain
```

## Destructive-op rules

- Every `delete` / `purge` / `update` / `disable` command is "destructive". Always pass `--dry-run` first if available, then re-run without it.
- Never run a destructive op without either an interactive confirmation (current default) or an explicit `--yes`/`-y` flag.
- If `--dry-run` is missing on a command and you'd want it during dogfooding, file a backlog item — it should be added.

## live-api feature

`cargo test --workspace --features live-api --quiet` runs E2E tests that hit the real API. They:

- read credentials from the `BUNNY_API_KEY` environment variable (see the auth section above — there is no on-disk config file)
- create resources prefixed `hoppy-test-` and tear them down at the end
- are gated so plain `cargo test --workspace` never touches the network

If a `live-api` test fails halfway and leaks a resource, the cleanup script (next section) is your fallback.

### Refreshing fixtures

**Two-step process** — record into a scratch directory first, then map the recordings
back to the hand-authored descriptive fixtures with `fixture-refresh`.

**Step 1 — record fresh responses into a scratch directory:**

```sh
SCRATCH="$(pwd)/fixtures-recorded"
HOPPY_RECORD_DIR="$SCRATCH" BUNNY_API_KEY=<live> \
    cargo test --workspace --features live-api -- --test-threads=1
```

- Recording writes under `$SCRATCH/<domain>/` using auto-derived filenames like
  `GET_billing.json`, `PUT_dnszone_50001.json`.
- `--test-threads=1` is required so two tests don't race on the same filename.
- Recording is idempotent: identical bytes are skipped silently.

**Step 2 — preview drift, then apply:**

```sh
# Dry-run: see what would change
cargo run --bin fixture-refresh -- --recorded fixtures-recorded

# Apply: overwrite descriptive fixtures that drifted
cargo run --bin fixture-refresh -- --recorded fixtures-recorded --apply
```

The tool scans `crates/**/tests/**/*.rs` to build a mapping of
`fixtures/<domain>/<name>.json → (HTTP method, path)`, then matches each
recording file by method + path-shape (numeric IDs in the recording match any
numeric ID in the fixture path). Output:

- `drift: <fixture> (Δ N bytes)` — content changed
- `collision: <recording> → [cand1, cand2]` — ambiguous; resolve manually
- `unmapped: <recording>` — no descriptive fixture references this endpoint

**Step 3 — verify:**

1. Run `git diff -- fixtures/` and spot-check changed fixtures. Even though
   `--record` redacts PII by default (see below), always do a manual diff review
   before committing fixture changes.
2. Look for account-specific leakage: account IDs, `LastUpdated` timestamps,
   per-account hostnames. Redacted fields (balance, email, tokens, etc.) will show
   `"<redacted>"` or `0` — that is expected.
3. Re-run `cargo test --workspace --quiet` to confirm the offline suite still passes.
4. Commit the drift together with the iteration change that drove it.
5. Clean up the scratch directory: `rm -rf fixtures-recorded`

#### PII redaction in `--record` fixtures

`--record` (and `HOPPY_RECORD_DIR`) redacts sensitive fields **by default** before
writing any fixture to disk. The following are masked automatically:

- **Field-name patterns** (case-insensitive substring): `email`, `payer`,
  `paymentid`, `balance`, `charges`, `recharge`, `invoice`, `downloadurl`,
  `apikey`, `accesskey`, `token`, `password`.
- **Value patterns**: URLs containing `?token=`, `&token=`, `signature=`, or
  `expires=`; JWT-shaped strings (three base64url segments).

String values become `"<redacted>"`; numbers become `0`; booleans and array/object
structure are preserved so fixture diffs remain meaningful.

To capture raw responses (e.g. to inspect the real API shape), pass `--no-redact`
or set `HOPPY_NO_REDACT=1`. **Do not commit raw output** — it may contain live
billing balances, payer emails, payment IDs, and short-lived signed download URLs.

#### Surfacing CLI-redacted secrets with `--reveal`

The CLI applies a separate redaction pass to its own output (independent of
`--record`) so secrets stay out of terminals and logs by default. The global
`--reveal` flag opts in to printing the raw value for every redacted field
across every output format (JSON/table/text). A separate `--reveal-env <KEY>`
flag exists for the container env-var case — it reveals a single env-var by
name and is not a variant of `--reveal`.

Commands that surface secrets:

- `hoppy storage-zone get --reveal --id <id>` — `Password` / `ReadOnlyPassword`.
- `hoppy stream library get --reveal --id <id>` — `ApiKey` / `ReadOnlyApiKey`.
  Also applies to `stream library create` (JSON/table/text) and
  `stream library list` (JSON only — the list table omits these fields).
- `hoppy db token mint --reveal …` — minted DB token.
- `hoppy container app get --reveal-env <KEY>` — single env-var by name
  (independent of `--reveal`).

**Notes on collisions and unmapped recordings:**

- *Collisions* occur when multiple descriptive fixtures share the same (method, path)
  (e.g. `pullzone_get.json`, `pullzone_get_with_edgerules.json` both served from
  `GET /pullzone/<id>`). Resolve by inspecting which fixture is closer to the live
  response and copying manually.
- *Unmapped recordings* are API calls hit by the live suite with no corresponding
  descriptive fixture — either a new endpoint added since the last refresh, or a
  path pattern the tool can't invert (e.g. unusual segment shapes). File a backlog
  item and add the fixture manually.

### Shape-first asserts in wiremock tests

Offline tests that assert on hand-authored fixture values will break every time a fixture refresh changes those values. Write **shape-first asserts** instead:

| Instead of… | Write… |
|---|---|
| `assert_eq!(billing.balance, 42.50)` | `assert!(billing.balance.is_finite()); assert!(billing.balance >= 0.0)` |
| `assert_eq!(zone.id, 1001)` | `assert!(zone.id > 0)` |
| `assert_eq!(result.items.len(), 2)` | `assert!(!result.items.is_empty())` |
| `assert_eq!(stats.total_bandwidth_used, 5368709120)` | `assert!(stats.total_bandwidth_used >= 0)` |
| `assert_eq!(chart.len(), 3)` | `assert!(!chart.is_empty())` |

**Three categories** — only the first changes:

- **Value-coupled** (rewrite): `assert_eq!` on a number or string that came directly from the fixture and could change on the next live sweep. Rewrite as an invariant (finite, non-negative, non-empty) or a presence check.
- **Shape-coupled** (keep): tests that verify serde behaviour — e.g. `assert!(billing.automatic_payment_card_type.is_none())` in a partial-response test. These are intentionally testing defaults and should not be loosened.
- **Wire-format** (keep): assertions on the *request* body or query string the client sent. These test what hoppy sends, not what the server returned, and do not depend on fixture values.

`insta` snapshot tests that embed full fixture output are implicitly value-coupled. Replace `insta::assert_snapshot!` calls with structural checks (valid JSON, expected field keys, non-empty collections) when the snapshot includes live-drifting fields.

### Serde-default gap — why `>= 0` isn't enough

Many model structs use `#[serde(default)]`. This means a renamed or removed JSON key silently deserialises to `0` / `false` / `""` instead of failing. A loosened assert like `assert!(billing.balance >= 0.0)` passes even when the `Balance` key was renamed to `Bal` in a new API version — the field is just 0.0.

Fix: after deserialising, also parse the **raw fixture body** as `serde_json::Value` and assert that the expected keys exist with the right JSON type:

```rust
let json: serde_json::Value = serde_json::from_str(FIXTURE_GET).unwrap();
assert!(json["Balance"].is_number(),  "Balance key missing or not a number");
assert!(json["BillingEnabled"].is_boolean(), "BillingEnabled key missing");
assert!(json["Items"].is_array(),     "Items key missing or not an array");
```

Apply this pattern when:

- The field type is numeric (integer or float) with `#[serde(default)]` — `>= 0` is vacuously true on a missing key.
- The field is a bool with `#[serde(default)]` — `let _ = field;` verifies nothing.
- The collection has `#[serde(default)]` — an empty-vec default passes `is_empty()` checks silently.

You do **not** need this for partial-response tests that intentionally exercise serde defaults (e.g. `get_billing_partial_response_uses_defaults`) — those tests are the default behaviour under test.

### Drift-tolerant CLI e2e snapshots and `stdout.contains` checks

The same drift-coupling problem occurs in CLI e2e tests (`crates/hoppy-cli/tests/e2e/`). After a fixture refresh, tests that snapshot the full CLI stdout or call `stdout.contains("150000")` will fail because the values came from the fixture and may now be different.

**Three layers to fix:**

1. **`insta::assert_snapshot!` on full CLI stdout** — prefer converting to structural asserts rather than adding `insta::with_settings!(filters => …)`, because `tabled` table column widths are dynamic (sized to the longest value). A filter replaces the value but the surrounding whitespace padding still changes, so the snapshot still fails. Structural asserts are robust:
   ```rust
   // Instead of snapshotting the table, check headers and key invariants:
   assert!(stdout.contains("Total Bandwidth Used"), "expected bandwidth column");
   assert!(Regex::new(r"Cache Hit Rate\s*\|\s*\d+\.\d+%").unwrap().is_match(&stdout));
   ```
   Keep `insta` snapshots only for output whose structure genuinely can't be tested any other way.

2. **`stdout.contains("specific_value")`** — replace with a regex that matches the column header or field name followed by any numeric/string value:
   ```rust
   // Instead of:
   assert!(stdout.contains("150000"));
   // Write:
   assert!(Regex::new(r"Total Requests Served\s*\|\s*\d+").unwrap().is_match(&stdout));
   ```

3. **`assert_eq!(json["field"], specific_value)`** — replace with a type check and optional invariant:
   ```rust
   assert!(json["TotalBandwidthUsed"].is_number());
   assert!(json["TotalBandwidthUsed"].as_i64().unwrap_or(-1) >= 0);
   ```

**When to keep a snapshot**: snapshots remain useful for testing that a subcommand's `--help` text or error output has the right shape (not value-coupled). The `assert_cli_snapshot!` macro in `tests/e2e/support/mod.rs` handles Windows `.exe` suffix normalisation.

## Cleanup script

`hoppy-knowledgebase/dogfooding/cleanup.sh` is **currently a skeleton**. Each surface block prints the manual `hoppy <noun> list` command you should run; no automated deletion has been implemented yet, and `--yes` deliberately refuses to proceed until the real delete paths exist. Until then, treat this section as a checklist for manual cleanup:

1. Run the script in its (default) dry-run mode to see the listing commands per surface.
2. For each surface, run `hoppy <noun> list`, grep for `hoppy-test-`, and delete matches manually (via `hoppy <noun> delete --id <id> --yes` or the dashboard).
3. Track implementation of the automated path in iter-25 / a dedicated backlog item.

Once implemented, the script will:

- list-and-skip anything not matching the prefix (defence in depth — never delete an unprefixed resource even if requested)
- print what it is about to delete and require `--yes` to proceed (default is dry-run)
- exit non-zero if any deletion fails so CI can catch leaks
- be **idempotent** — safe to run before AND after a session

The script is intentionally a shell script (not Rust) so anyone can read and audit it. It's a knowledgebase helper script, not part of the build — see CLAUDE.md "Code Patterns" for the Rust-only rule and its exemption.

## Friction → backlog

Anything that surprises, frustrates, or trips you up during a session goes into `hoppy-knowledgebase/backlog/` as a separate file. Use the `backlog` type:

```yaml
---
title: <short title>
type: backlog
date: <today>
status: planned
priority: low|medium|high|critical
origin: dogfooding-2026-05-09
---
```

This is how iterations get fed. The dogfooding session is where the next iteration's plan is born.

## Known broken commands

These commands are documented as broken as of their listed date. Avoid running them in production or dogfooding sessions until fixed.

- `hoppy container logs` — As of 2026-05-15, may fail at the log-forwarding-create step with an empty-body 400 from the bunny.net API. See [[../backlog/log-forwarding-create-empty-400]].

## Related

- [[../decision-log]] — convention rules
- [[../iterations/iteration-23-hyalo-best-practices]]
- [[../iterations/iteration-25-publish]]
