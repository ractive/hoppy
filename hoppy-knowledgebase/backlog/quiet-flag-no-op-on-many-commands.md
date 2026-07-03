---
title: --quiet flag is a no-op on auth check and likely other commands
type: backlog
date: 2026-06-01
status: resolved
priority: medium
origin: dogfooding-2026-06-01
tags:
  - cli
  - global-flags
  - dx
  - consistency
---

# `--quiet` is documented but doesn't suppress anything

```sh
$ hoppy auth check --quiet
+--------------------+------------------------------------------+
| Field              | Value                                    |
+--------------------+------------------------------------------+
| API Key            | valid                                    |
+--------------------+------------------------------------------+
| Balance            | $9.4204                                  |
...
```

The global `--quiet` flag is documented in every `--help` output as
"Suppress non-essential output", but on `auth check` (and likely many
other read commands) it has no observable effect — the full table is
still printed.

## Expected behaviour

Two reasonable interpretations:

1. **Strict**: `--quiet` only suppresses ancillary lines (drill-down
   hints, "Saved to …" confirmation prints, etc.), not the primary
   payload. If that's the design, then `auth check` already prints
   nothing ancillary and the flag is genuinely a no-op for that
   command — but the help should say so, or the flag should be hidden
   when nothing's suppressible.
2. **Liberal**: on commands like `auth check` where a non-zero exit
   code already encodes failure, `--quiet` should suppress the entire
   table on success and only print on error. Useful in shell scripts:
   `if hoppy auth check --quiet; then ...`.

Whichever direction we pick, the current "flag exists, does nothing"
is the worst option.

## Action

1. Decide on the contract (strict vs liberal).
2. Audit each command's stdout/stderr split. For each, list what
   `--quiet` should suppress.
3. Either hide the flag where it's truly a no-op, or implement it.

## Acceptance

- `--quiet` is either visibly effective or hidden from help on every
  command.
- Documented contract for `--quiet` lives in CLAUDE.md and/or the
  CLI surface notes.
