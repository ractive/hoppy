---
title: --quiet flag classification per subcommand
type: reference
date: 2026-06-01
tags:
  - cli
  - global-flags
  - dx
---

# `--quiet` flag classification

`--quiet` is a global flag on every `hoppy` subcommand. Its meaning
depends on the command's category. The contract is:

- **Predicate commands** — the exit code already carries the
  success/failure signal. Under `--quiet`, the **entire stdout payload
  is suppressed on success**. Errors still print to stderr and exit
  non-zero, so the command is safe in shell conditionals:

  ```sh
  if hoppy auth check --quiet; then
      echo "key works"
  fi
  ```

- **Data commands** — the table/JSON payload *is* the point. Under
  `--quiet`, the primary payload still prints, but **ancillary lines**
  (drill-down hints, progress bars, `Saved to …` confirmations) are
  suppressed. Hints are suppressed centrally via
  `output::hints::set_enabled` in `main.rs` whenever `--quiet` is set.

## Classification

| Command                    | Category   | `--quiet` effect |
|----------------------------|------------|------------------|
| `auth check`               | predicate  | success silent; error to stderr |
| `db ping`                  | predicate  | success silent; error to stderr |
| `purge`                    | data       | hints suppressed |
| `pull-zone list/get/...`   | data       | hints suppressed |
| `storage-zone list/get/...`| data       | hints suppressed |
| `storage upload/download`  | data       | progress bars + `Uploaded/Downloaded` confirmations suppressed (existing behaviour) |
| `storage ls/get/delete`    | data       | hints suppressed |
| `dns ...`                  | data       | hints suppressed |
| `stream ...`               | data       | progress bars + hints suppressed |
| `shield ...`               | data       | hints suppressed |
| `script ...`               | data       | hints suppressed |
| `container ...`            | data       | hints suppressed |
| `db list/get/create/...`   | data       | hints suppressed |
| `statistics`               | data       | hints suppressed |
| `video-library ...`        | data       | hints suppressed |
| `completions`              | data       | n/a (shell output) |

## Notes

- `--no-hints` is a finer-grained flag that only suppresses hints
  without changing payload behaviour.
- `--format json` implies `--no-hints` (machine output stays clean)
  but does **not** change the `--quiet` payload contract.
- The `quiet` boolean is plumbed into command handlers that need to
  branch on it (currently `auth::handle` and `database::handle` for
  the predicate commands above; `storage::handle` and `stream::handle`
  use it for progress bars). Other handlers read `--quiet` indirectly
  through the global hints gate.
