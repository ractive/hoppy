---
title: Iter-28 — CLI UX polish (flag naming, enums, output formatting)
type: iteration
date: 2026-05-10
tags:
  - iteration
  - ux
  - cleanup
  - dogfooding
status: completed
branch: iter-28/dogfooding-ux-polish
---

# Iter-28 — UX consistency pass from dogfooding 2026-05-10

Seven small papercuts surfaced during the 2026-05-10 round. None of them
break functionality; together they make the CLI feel hand-written
inconsistently. Group them into one pass so flag/output conventions are
reviewed together rather than per-noun.

## Scope

### 1. Standardise flag names across nouns
Source: [[../backlog/flag-naming-consistency]]

- [x] storage: standardise on `--remote-path` everywhere (currently `ls`
      uses `--path`). Pick a consistent local-file flag — proposal: keep
      `--file` for upload (input) and rename `download --output` → `--file`
      OR rename upload `--file` → `--input`. Document the choice in
      `decision-log.md`.
- [x] `stream library delete` uses `--id`; siblings (`list`/`get`/`update`/
      `statistics`) use `--library-id`. Standardise on `--id` (matches
      pull-zone, dns zone, db, container — the rest of hoppy).
- [x] Audit every `delete` subcommand to confirm `--id` (no `--<noun>-id`)
      is the convention.

### 2. Replace raw numeric enum flags with named ValueEnums
Source: [[../backlog/numeric-enum-flags]]

Mirror iter-26's `pull-zone create --zone-tier {premium,volume}` pattern.

- [x] `script create --script-type` → `{dns, cdn, middleware}` (was 0/1/2)
- [x] `storage-zone create --zone-tier` → `{standard, edge}` (was 0/1)
- [x] Grep the cli.rs for any remaining `(0 = …, 1 = …)` help text.

### 3. Date/time flag friction
Source: [[../backlog/date-format-friction]]

- [x] Helper: accept `YYYY-MM-DD` as a synonym for `YYYY-MM-DDT00:00:00Z`
      across every time-window flag.
- [x] Apply to `db statistics --from/--to`, `db usage --from/--to`,
      `statistics --date-from/--date-to`, `video-library drm-statistics
      --date-from/--date-to`, etc.
- [x] `shield event-logs --date` currently wants `MM-dd-yyyy` (US-only).
      Accept ISO `YYYY-MM-DD` and translate client-side.
- [x] On bad input, hoppy should surface a clear message naming the
      accepted formats — never let "premature end of input" reach the
      user.

### 4. `db delete` / `db group delete` confirmation message
Source: [[../backlog/db-delete-output-format]]

- [x] Both currently print empty table headers. Make them print
      `Deleted database <id>` / `Deleted database group <id>` like every
      other delete subcommand.
- [x] JSON form: `{"deleted": "<id>"}` or `{}`.

### 5. `dns zone dnssec status` text view drops the DS record
Source: [[../backlog/dnssec-status-text-output-thin]]

- [x] Text/table view shows only `id`/`domain`/`enabled` today. When
      enabled, also show DS record, digest, key tag, algorithm, and
      `DsConfigured`. JSON view already has these.

### 6. `stream library statistics` shows "Engagement Score -1"
Source: [[../backlog/stream-library-stats-engagement-minus-one]]

- [x] When the API returns the `-1` "no data" sentinel, render `N/A` in
      text/table mode. Keep the raw `-1` in JSON for machine readers.

### 7. Dogfooding playbook docs vs. CLI mismatch
Source: [[../backlog/auth-login-missing-from-cli]]

- [x] Playbook references `hoppy auth login` and "non-default config paths
      via `hoppy auth --help`" — neither exists. Replace the auth section
      with the actual flow: set `BUNNY_API_KEY`, run `hoppy auth check` to
      validate.
- [x] Top-level `--help` already says "Set the BUNNY_API_KEY environment
      variable to authenticate" — keep the playbook in sync.

## Out of scope

The five real bugs ship in iter-27 — see
[[iteration-27-dogfooding-bugfixes]]. Don't try to fix them here.

## Acceptance

- [x] No remaining `--<noun>-id` flags on `delete` subcommands.
- [x] No remaining "(0 = …, 1 = …)" help text on `script create` and
      `storage-zone create` (iter-28 scope). Shield's numeric WAF / DDoS /
      access-list / rate-limit flags still expose raw enum ints — tracked
      separately as a follow-up backlog item.
- [x] Date flags accept date-only input across the CLI.
- [x] All `delete` commands print a confirmation line in text mode.
- [x] `dns zone dnssec status` text view shows DS record, digest, key tag,
      algorithm when DNSSEC is enabled.
- [x] `stream library statistics` shows `N/A` for the `-1` sentinel in
      text/table mode, retains `-1` in JSON.
- [x] Playbook reads correctly end-to-end against the current CLI.
- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings
      && cargo test --workspace --quiet` clean.

## Related

- [[../dogfooding/dogfooding-playbook]]
- [[iteration-27-dogfooding-bugfixes]]
- [[../decision-log]]
