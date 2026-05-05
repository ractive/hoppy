---
title: Iteration 21 — Magic Containers UX & safety + cross-cutting redaction
type: iteration
date: 2026-05-05
tags:
  - iteration
  - bugfix
  - magic-containers
  - safety
  - dx
  - security
status: completed
branch: iter-21/magic-containers-ux
---

# Iteration 21 — Magic Containers UX & safety + cross-cutting redaction

**Goal:** Address the Magic Containers (MC) gaps surfaced by `wardrobe-assistants.ch/kb/hoppy-bug-report-magic-containers.md` (2026-05-05). Includes one **high-severity operational footgun** (`template env` silently wipes all env vars), three Medium issues (auto-PZ orphans on app delete, list excludes auto-PZs from MC angle, plaintext secrets in `app get`), and two ergonomic Lows. Also lands a hoppy-wide secret redaction policy that iter-19 (storage-zone passwords) and iter-20 (database tokens) both depend on.

## Context

Bug reports (2026-05-05 cluster):
- `../wardrobe-assistants.ch/kb/hoppy-bug-report-magic-containers.md` — primary source for this iteration (six MC-specific issues).
- `../wardrobe-assistants.ch/kb/hoppy-bug-report-pullzone-storagezone.md` — sister report; SZ.1 (storage password) needs the same redaction stance.
- `../wardrobe-assistants.ch/kb/hoppy-bug-report-database-cli.md` — sister report; DB token mint needs the same redaction stance.
- `../wardrobe-assistants.ch/kb/hoppy-usage-report.md` — iter-9 follow-up flagged "no `--env` flag on `container app create`".

The 12-issue cross-reference index in the MC report shows the full picture: iter-19 covers PZ.x + SZ.1 + GEN.x, iter-20 covers DB.1 + GEN.4, **iter-21 covers MC.1–MC.6** plus the cross-cutting redaction layer that all three iterations need.

## Scope

### Cross-cutting secret redaction (foundation)

A reusable redaction layer used by storage-zone (iter-19), database tokens (iter-20), and Magic Containers (this iteration). Land it first; downstream issues consume it.

- [ ] Audit `bunny-api-*` types for fields whose names match `*_password`, `*_secret`, `*_token`, `*_key`, `*_credential` (case-insensitive)
- [ ] Provide a `Redacted<T>` newtype (or `#[serde(serialize_with = "redact")]` helper) that prints `<set, length=N>` in JSON and table outputs by default; a global `--reveal` and per-command `--reveal-<scope>` flags opt in to raw output
- [ ] Wire `--reveal` into the CLI's global flags layer (`Cli` struct in `src/cli.rs`); document precedence (CLI flag > env var > default)
- [ ] `--reveal` is **off by default** even with `--format json` — do not change behaviour silently if a future `BUNNY_REVEAL=1` env var is added
- [ ] Snapshot test pattern: every command that touches a secret-bearing field has a snapshot asserting redaction is the default and a parallel snapshot asserting `--reveal` shows the value. Pattern lives in `tests/support/`.
- [ ] Document in `decision-log.md` and `api/bunny-api-quirks.md`

This block is the **foundation** referenced by:
- iter-19 §"`storage-zone get` strips Password / ReadOnlyPassword" — flips from "we strip them" to "we surface them, redacted by default, with `--reveal`"
- iter-20 `db token mint` — uses the same `Redacted<String>` for the JWT
- iter-21 below — `container app get`, `container template get`

### Issue MC.1 — `container template env` silently wipes ALL env vars (HIGH)

**Issue:** `hoppy --yes container template env --app-id <a> --container-id <c>` (no `--env` flags) replaces all env vars with the empty set. Reproduced in iter-9: 9 vars → 0 vars, no warning, exit 0. Sign-in / DB / TLS broken at next pod start.

- [ ] **Refuse zero-`--env` calls by default.** Error: `at least one --env required, or use --clear to wipe explicitly.`
- [ ] Add `--replace-all` flag (combine with `--env K=V ...`): the destructive "set the whole array" behaviour, named explicitly. Without `--replace-all`, granular flags (MC.5 below) are the default.
- [ ] Add `--clear` flag (standalone): wipe all env vars with explicit consent. Mutually exclusive with `--add` / `--remove` / `--update` / `--env` **and** `--replace-all` (it is the named wipe; combining the two is meaningless). Equivalent to `--replace-all` with zero `--env`, but a separate flag because "wipe everything" is an intent worth naming.
- [ ] `--yes` alone is **not** sufficient to authorize either `--clear` or a destructive `--replace-all` that drops to zero. Interactive confirmation calls out the count: `Replace 9 environment variables with 0? Type "wipe" to confirm.`
- [ ] Help text loudly calls out the destructive default with a recipe block: `# Add a single var without losing the rest:\nhoppy container template env --add KEY=VAL ...`
- [ ] Mock test asserts the zero-`--env` flow fails with the friendly error and never sends a request to the API
- [ ] Live E2E: create a template with N env vars → run a no-op env command without `--replace-all` → assert the N vars survive

### Issue MC.5 — granular env operations

**Issue:** `template env` only "replaces all". Operators editing one var must keep the full set in scope.

- [ ] `hoppy container template env --add KEY=VAL [--add ...]` — idempotent add-or-update by name (read current set → merge → PATCH/PUT)
- [ ] `hoppy container template env --remove KEY [--remove ...]` — idempotent remove-if-present
- [ ] `hoppy container template env --update KEY=VAL` — alias for `--add`; keep semantics identical
- [ ] `hoppy container template env --replace-all --env K=V [...]` — explicit set-the-whole-array
- [ ] `hoppy container template env --clear` — explicit wipe (see MC.1 above)
- [ ] `hoppy container template env --list` (or `--show`) — print env names only by default; redaction layer governs values
- [ ] `--add` / `--update` / `--remove` are repeatable; in a single invocation **all `--add`/`--update` inserts run before all `--remove` drops** (clap groups values by flag name, so we cannot recover argv ordering). Document this precedence in `--help` so a user passing both `--add KEY=v` and `--remove KEY` knows the remove wins. `--replace-all`, `--clear`, and `--list` are each mutually exclusive with all of `--add` / `--update` / `--remove`.
- [ ] Mock tests for each flag mode + a combined add-and-remove case; snapshot the final PATCH body

### Issue MC.3 — `container app delete` orphans the auto-managed Pull Zone

**Issue:** Deleting a Magic Container app leaves its auto-PZ live and billable.

- [ ] Discover dependents: when running `container app delete`, fetch the app's endpoints and collect auto-PZ ids (`endpoint.pull_zone_id`)
- [ ] Default behaviour: print the list of dependents and **refuse** to delete unless the user passes `--cascade` or explicitly accepts. The interactive confirmation must list each auto-PZ id.
- [ ] `--cascade`: delete the app, then delete each auto-PZ. Continue on individual PZ-delete failures (log and aggregate at end). Exit non-zero if any cleanup failed.
- [ ] `--no-cascade` (or just declining): delete only the app and print actionable cleanup commands: `Deleted app <id>. Note: 1 auto-managed Pull Zone (<id>) was NOT deleted; remove with: hoppy pull-zone delete --id <id> --yes`
- [ ] Live E2E: create app with endpoint → `app delete --cascade --yes` → assert both app and auto-PZ are 404
- [ ] Live E2E: create app with endpoint → `app delete --no-cascade --yes` → assert app is 404, auto-PZ still 200, message mentions the ID

### Issue MC.2 — `pull-zone list` excludes auto-managed PZs (MC angle)

**Note:** iter-19 covers this from the Pull Zone command surface; iter-21 ensures the **Magic Container** flow benefits from it. No duplicate work — this is a cross-reference, not a separate task.

- [ ] After iter-19's `pull-zone list --include-managed` (or whatever name lands), update the MC report's runbook test in this iteration's tests to confirm auto-PZs surface
- [ ] If `container app delete --cascade` from MC.3 above and `pull-zone list --include-managed` from iter-19 both land, document the operator workflow in `api/bunny-api-quirks.md` so both halves are discoverable from each side

### Issue MC.4 — `container app create` return is too thin

**Issue:** Provisioning a working stack today takes 3+ `app get` calls to chain ids. Operators (and LLMs) want the full object after `create`.

- [ ] `container app create` defaults to returning the full app document (matching `app get`) when run with `--format json`
- [ ] `--minimal` flag preserves the current `{"id": "..."}` output for users who specifically want it
- [ ] `--format table` output adds container template id and endpoint id columns
- [ ] Audit other `*_create` commands for the same gap; apply the same pattern (`*_create` returns the full object by default)
- [ ] Snapshot tests for both `--format json` (full doc) and `--minimal`

### Issue MC.6 — `container app get` / `container template get` return env values plaintext

**Issue:** Plaintext `BETTER_AUTH_SECRET`, `RESEND_API_KEY`, `DATABASE_AUTH_TOKEN` in terminal scrollback.

- [ ] Apply the cross-cutting `Redacted<String>` to env-var values across all container-template responses
- [ ] Snapshot tests in `tests/cli_container.rs` confirming default-redacted output and `--reveal` raw output
- [ ] `--reveal-env <KEY>` (per-key opt-in) for the cases where users only want one var; default is still redact-all. `--reveal-env` is repeatable. Precedence: `--reveal` (global) takes precedence — when present, every secret is revealed and `--reveal-env` entries become a no-op. Without either flag, redaction is on for every secret-bearing field.

### Container app create env-vars on create (from usage report #7)

**Issue:** `container app create` accepts no `--env` flags; setting env at creation time requires the dashboard or a separate `template env` call.

- [ ] Add `--env KEY=VAL` (repeatable) to `container app create` — passes through to the bunny API in the create request body if supported, else wraps create + `template env --add` into one CLI call
- [ ] Help text shows the merge-vs-batch trade-off so operators know what they're getting

## Implementation Notes

- **Order of work.** Land the redaction layer first, then MC.1 (highest severity), then MC.5, then MC.3, then MC.4 + MC.6. The remaining items are smaller and can interleave.
- **Don't merge MC.1 + MC.5 into one PR.** MC.1 is about safety-of-defaults; MC.5 is about new flag surface area. Reviewing them separately is cleaner.
- **Live E2E cleanup discipline.** Every MC live test that creates an app must register a `cleanup.push(&["container", "app", "delete", "--id", &id, "--cascade"])` so a panic doesn't leave billable resources behind. Update `tests/support/mod.rs` if the cleanup-stack abstraction needs a `--cascade`-aware variant.
- **Redaction must apply to all output formats**, not just JSON. Table and `--format text` outputs must also redact. Snapshot test all three formats.

## Test cases (from the MC report)

1. **Env-preservation:** create container with 5 env vars → `app update --image-tag <new>` → assert env still has 5 vars and matching values
2. **Cascade delete:** create container with auto-PZ → `app delete --cascade` → assert auto-PZ is 404
3. **No-cascade delete:** create container → `app delete --no-cascade` → assert auto-PZ still 200, message lists the orphan ID
4. **Provisioning round-trip count:** create → endpoint add → bind hostname doable in ≤2 hoppy invocations after MC.4 fix
5. **Redaction default:** `container app get` of an app with secret-named env vars → values are `<set, length=N>` in JSON, table, and text formats
6. **Reveal flag:** same command with `--reveal` → values appear raw

## Estimated Complexity

| Topic | Complexity |
|-------|------------|
| Cross-cutting redaction layer | Medium |
| MC.1: refuse zero-`--env` + `--replace-all` + `--clear` | Small |
| MC.5: `--add` / `--remove` / `--update` granular flags | Medium |
| MC.3: `--cascade` on `app delete` with dependent discovery | Medium |
| MC.4: full-object return on `app create` (+ audit other creates) | Small |
| MC.6: redaction wired into container outputs | Small (depends on layer) |
| `--env` on `container app create` | Small |
| Mock + live tests | Medium |
| Docs (decision log, quirks, help text) | Small |
| **Total** | **Medium–Large** |

## Dependencies

**iter-21 can — and should — ship first.** MC.1 (silent env-wipe) is the highest-severity issue across all three field reports: a single command with no warning destroys runtime configuration. Don't gate this on iter-19 or iter-20.

- **No hard dependency on iter-19.** iter-19's forward-compat enum strategy is a nice-to-have for any new enums introduced here, but the MC surface is mostly existing types. If a new enum value is added (e.g. for `--cascade` discovery), apply the same `Unknown(i32)` fallback inline; iter-19 hoists the pattern crate-wide later.
- **No hard dependency on iter-20.** iter-20 doesn't share code with this iteration.
- **Reverses the redaction-layer dependency.** iter-21 owns and ships the cross-cutting redaction layer first; iter-19 (SZ.1) and iter-20 (DB token mint) then **consume** it when they ship. This is the cleanest ordering — the layer lands once, in its native iteration, and downstream consumers just import it.
- **Cross-iteration coordination.** MC.2 (auto-PZ visibility) overlaps with iter-19's `pull-zone list --include-managed` work. iter-21 doesn't block on that — the verification test is a follow-up that runs once iter-19 also lands.

Recommended sequence: **iter-21 → iter-19 → iter-20** (severity-ordered; downstream iterations consume iter-21's redaction layer).

## Related

- Field reports: `../wardrobe-assistants.ch/kb/hoppy-bug-report-magic-containers.md`, `../wardrobe-assistants.ch/kb/hoppy-bug-report-pullzone-storagezone.md`, `../wardrobe-assistants.ch/kb/hoppy-bug-report-database-cli.md`, `../wardrobe-assistants.ch/kb/hoppy-usage-report.md`
- [[development-roadmap]]
- [[adding-a-feature]]
- [[api/bunny-api-quirks]]
- [[decision-log]]
