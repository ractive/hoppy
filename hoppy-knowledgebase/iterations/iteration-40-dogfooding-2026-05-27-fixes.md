---
title: Iter-40 — dogfooding 2026-05-27 fixes
type: iteration
date: 2026-05-27
tags:
  - iteration
  - bugfix
  - cli
  - get-tables
  - storage
status: planned
branch: iter-40/dogfooding-2026-05-27-fixes
---

# Iter-40 — dogfooding 2026-05-27 fixes

## Why

Post-iter-39 dogfooding round (against the live test account) verified
the three iter-39 fixes work as designed, but surfaced one significant
partial regression on the `get` pivot plus two smaller CLI consistency
issues. None block the user, but the first one undoes most of iter-39's
readability win for resources whose response contains nested fields.

## Scope

### 1. Embedded JSON blows out `get` table width
Source: [[../backlog/get-embedded-json-blows-width]]

iter-39 pivoted single-resource `get` commands to a vertical Field/Value
table. The pivot is correct for scalar-only resources (`storage-zone get`
= 70 chars wide ✓). But fields whose value is a JSON array or object
are still rendered as inline single-line JSON, blowing out the Value
column:

| Command | Width | Cause |
|---|---|---|
| `storage-zone get` | 70 chars | scalar fields only ✓ |
| `pull-zone get` | 176 chars | `Hostnames` rendered as `{"Id":...,"Value":"...",...}` |
| `container app get` | 922 chars | `repositorySettings`, `regionSettings`, `volumes` are nested objects/arrays |

922 chars is catastrophic on any terminal; 176 wraps unreadably on the
common 120-col case.

- [ ] In the shared Field/Value renderer (likely
      `crates/hoppy-cli/src/output/` or per-command in
      `crates/hoppy-cli/src/commands/`), detect when a value is a JSON
      array or object after serialisation and replace it with a summary
      in **table mode** only (JSON mode is untouched).
- [ ] Summary shape: `<3 hostnames>` / `<object: 7 fields>` /
      `<empty list>` — leading `<` and trailing `>` so the user can
      tell at a glance it's a placeholder, not data.
- [ ] After the summary cell, append a stderr hint pointing the user at
      the JSON view:
      `tip: hoppy --format json container app get --id <id> | jq .repositorySettings`
      (one hint per get, with the most useful field named — or a single
      generic hint if there are multiple nested fields).
- [ ] Audit pass on every `get` to find anywhere else nested JSON leaks
      into a table cell: `pull-zone`, `storage-zone`, `container app`,
      `container endpoint`, `stream library`, `stream video`,
      `dns zone`, `shield zone`, `script`, `database`.
- [ ] e2e snapshot updates for the new "summary cell" shape. Keep
      drift-tolerant per the iter-37 playbook.
- [ ] Dogfooding pass: re-run all five worst-offender gets and confirm
      output now fits a 120-col terminal.

### 2. `shield zone get` uses `--shield-zone-id` instead of `--id`
Source: [[../backlog/shield-zone-get-id-flag-inconsistent]]

Every other `get` command takes `--id`. `shield zone get` requires
`--shield-zone-id`. Clap helpfully suggests the right alternative on
typo, so this isn't a hard blocker — just unnecessary muscle-memory
friction.

- [ ] Rename the argument to `--id` in `crates/hoppy-cli/src/cli/shield.rs`
      (or wherever the subcommand is defined). Keep `--shield-zone-id`
      as a clap alias for one or two releases.
- [ ] Audit other resource-id args for the same shape:
      `grep -rn '#\[arg(long' crates/hoppy-cli/src/cli/ | grep -E '\-\-\w+\-id'`
      and rename + alias each one that has a non-`--id` form on a
      single-resource get/update/delete.
- [ ] e2e snapshot updates for changed `--help` text.

### 3. Storage display paths show `zone//path` when --remote-path has leading slash
Source: [[../backlog/storage-remote-path-double-slash]]

`hoppy storage upload --zone Z --file local --remote-path /foo.txt`
prints `Uploaded local → Z//foo.txt`. The double slash is just a string
join issue (`Z + "/" + "/foo.txt"`); the API resolves both forms the
same, so functionality is unaffected.

- [ ] In `crates/hoppy-cli/src/commands/storage.rs` (or wherever the
      upload/rm/download success messages are built), trim leading `/`
      from `remote_path` before joining: `remote_path.trim_start_matches('/')`.
- [ ] Apply to all three commands: `upload`, `rm`, `download`.
- [ ] e2e snapshot updates for the changed display strings.

## Out of scope

- Rewriting the Field/Value renderer to support nested sub-tables.
  Summary-cell + JSON-mode-redirect is the cheap path; nested rendering
  is a bigger UX project, not this round.
- Touching what the API itself returns. All three fixes are CLI-side.
- Renaming `--shield-zone-id` without an alias period — keep the alias
  for back-compat so existing scripts don't break.

## Acceptance

- [ ] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings &&
      cargo test --workspace --quiet` clean.
- [ ] Dogfooding repro of all three issues against the live test account
      shows the fixed behaviour:
      - `container app get` fits comfortably on a 120-col terminal;
        `repositorySettings` rendered as `<object: N fields>` with a
        `tip:` hint pointing at the JSON view.
      - `hoppy shield zone get --id <id>` works (no more "unexpected
        argument").
      - `hoppy storage upload --remote-path /foo.txt …` prints
        `zone/foo.txt`, not `zone//foo.txt`.
- [ ] All three backlog items closed (`status=resolved`) with a link to
      this iteration.

## Related

- [[../backlog/get-embedded-json-blows-width]]
- [[../backlog/shield-zone-get-id-flag-inconsistent]]
- [[../backlog/storage-remote-path-double-slash]]
- [[iteration-39-dogfooding-2026-05-26-fixes]] — original `get` pivot
- Dogfooding round: 2026-05-27 (post-iter-39).
