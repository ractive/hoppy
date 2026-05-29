---
title: Iter-42 — dogfooding 2026-05-29 fixes
type: iteration
date: 2026-05-29
tags:
  - iteration
  - bugfix
  - cli
  - help-text
  - consistency
status: in-progress
branch: iter-42/dogfooding-2026-05-29-fixes
---

# Iter-42 — dogfooding 2026-05-29 fixes

## Why

Post-iter-41 dogfooding confirmed `<1 item>`, list-table truncation, and
the truncation tip on stderr all work as designed. But it surfaced one
clear partial regression (iter-41 §3 only fixed one of the four
commands listed in its own plan) plus two related consistency issues
that all touch sub-resource argument naming and presentation. Bundling
them because §1 directly continues iter-41's unfinished work and §2/§3
are adjacent to it.

## Scope

### 1. Finish iter-41 §3 — add `--help` text to the three missed commands
Source: [[../backlog/iter-41-sub-resource-help-incomplete]]

iter-41 §3 committed to documenting the parent-resource arg on four
sub-resource commands. Only `pull-zone edge-rule list` actually got the
fix. Three are still undocumented:

```
hoppy pull-zone hostname add --help    → --id <ID>            (no help)
hoppy container endpoint list --help   → --app-id <APP_ID>    (no help)
hoppy dns record list --help           → --zone-id <ZONE_ID>  (no help)
```

- [x] Add `help = "Pull zone ID"` to the `--id` arg on every
      `pull-zone hostname <verb>` subcommand (`add`, `remove`,
      `load-free-cert`, `force-ssl`, `add-cert`, `remove-cert`).
- [x] Add `help = "Container app ID"` to the `--app-id` arg on every
      `container endpoint <verb>` and any other `container <noun> <verb>`
      that takes an `--app-id`.
- [x] Add `help = "DNS zone ID"` to the `--zone-id` arg on every
      `dns record <verb>` and any other `dns <noun> <verb>` that takes
      a `--zone-id`.
- [x] Broader sweep: re-run the audit grep iter-41 §3 specified and
      flag any other sub-resource args that still lack a `help = "..."`.
      Don't trust the "just these four" scope this time — the iter-41
      miss happened precisely because the audit step was skipped.
      ```sh
      grep -rn -B1 -A3 '#\[arg' crates/hoppy-cli/src/cli/ \
        | grep -B3 '"id"\|"app_id"\|"zone_id"\|"library_id"\|"video_id"\|"record_id"'
      ```
- [x] e2e snapshot refresh for changed `--help` output across all
      affected subcommands.

### 2. Unify parent-resource arg name across sub-resource commands
Source: [[../backlog/parent-resource-arg-name-inconsistency]]

Three different names for the same concept (parent resource id):

| Sub-resource command | Current arg |
|---|---|
| `pull-zone edge-rule list` | `--id` |
| `pull-zone hostname add` | `--id` |
| `container endpoint list` | `--app-id` |
| `dns record list` | `--zone-id` |

Pick **Option A from the backlog**: rename to `--id` everywhere, keep
the old name as a clap alias for back-compat. Matches iter-40 §2's
top-level `--id` unification.

- [x] In `crates/hoppy-cli/src/cli/container.rs`, rename the
      `--app-id` arg on `endpoint <verb>` (and any other
      `container <noun> <verb>` that takes it) to `--id`, with
      `alias("app-id")`.
- [x] In `crates/hoppy-cli/src/cli/dns.rs`, rename `--zone-id` on
      `record <verb>` (and adjacent commands like `dnssec`, `export`,
      `import`) to `--id`, with `alias("zone-id")`.
- [x] Audit other surfaces (`stream library`, `stream video`,
      `script`, `database`, `shield waf`) for the same pattern. Apply
      the same rename + alias.
- [x] **Don't** rename ambiguity-sensitive cases — if a sub-resource
      command takes both a parent ID AND its own ID (e.g.
      `pull-zone edge-rule update --id <pz> --rule-id <r>`), leave
      both names alone. The rename is for commands where there is no
      ambiguity.
- [x] e2e snapshot refresh for changed `--help` output.
- [x] Dogfooding pass: re-run the four canonical commands with `--id`
      and confirm they accept it; also re-run with the old names and
      confirm aliases still work.

### 3. Table-label vs JSON-key naming mismatch
Source: [[../backlog/table-label-json-key-case-mismatch]]

`hoppy shield waf profiles` (and likely other camelCase-API-backed
commands) renders table headers in Title-Case while JSON output uses
camelCase, with non-obvious mappings (`Category` → `profileCategory`,
`Premium` → `isPremium`). The iter-41 truncation tip even names the
column, but a user querying `.Description` against the JSON output
gets `null` — they need `.description`.

- [x] **Smallest useful fix**: update the iter-41 truncation tip so it
      cites the JSON key, not the column label, when the two differ.
      E.g.:
      `tip: some values were truncated — use --format json (key: .description)`
      One-line tweak in the truncation tip helper.
- [x] Identify other commands with the same Title↔camel mismatch by
      running each `list` and comparing column headers vs JSON keys.
      Limit to commands whose JSON shape is camelCase (containers,
      most shield endpoints).
- [x] For those commands, audit whether the table renderer is using a
      `humanise(field_name)` helper or hand-written column labels.
      If the former, consider rendering the JSON-key shape verbatim so
      the user only has to learn one name per field.
- [x] Out of scope: changing the bunny.net API key shapes themselves
      (that's upstream).
- [x] e2e snapshot refresh if column labels change.
- [x] Dogfooding pass: confirm `tip:` cites the right key for
      `shield waf profiles`, and re-run any other affected commands.

## Out of scope

- Renaming the JSON shape (camelCase ↔ PascalCase is the API's choice).
- Rebuilding the truncation helper (iter-41 shipped it; we're only
  tweaking the tip text).
- The `--shield-zone-id` legacy alias from iter-40 — leave it.

## Acceptance

- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings &&
      cargo test --workspace --quiet` clean.
- [x] Dogfooding repro against the live test account shows:
      - `hoppy pull-zone hostname add --help` lists
        `--id <ID>  Pull zone ID`.
      - `hoppy container endpoint list --id <id>` works AND
        `--app-id <id>` still works (alias preserved).
      - `hoppy dns record list --id <id>` works AND `--zone-id <id>`
        still works.
      - `hoppy shield waf profiles` truncation tip cites `.description`
        (or whatever the camelCase JSON key is), not `Description`.
- [x] All three backlog items closed (`status=resolved`) with a link to
      this iteration.

## Related

- [[../backlog/iter-41-sub-resource-help-incomplete]]
- [[../backlog/parent-resource-arg-name-inconsistency]]
- [[../backlog/table-label-json-key-case-mismatch]]
- [[iteration-40-dogfooding-2026-05-27-fixes]] — top-level `--id` unification
- [[iteration-41-dogfooding-2026-05-28-polish]] — original (incomplete) §3
- Dogfooding round: 2026-05-29 (post-iter-41).
