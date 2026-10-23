---
title: Positional id for the remaining single-resource commands
type: backlog
date: 2026-10-23
status: planned
priority: low
origin: iter-86
---

# Commands not yet converted to `IdArg`

iter-86 converted every subcommand whose only identifier is a plain `#[arg(long)] id` (108 sites plus the 13 `get`s). Left on `--id` only:

- Shield commands using `#[arg(long = "id", alias = "shield-zone-id")] shield_zone_id` (`shield zone get`, `api-guardian get`, `upload-scanning`, `bot-detection`, custom-page, ...): `IdArg` has no alias hook yet.
- `container log-forwarding get` (`--id`/`--app-id` alias).
- Commands with several identifiers (`--library-id` + `--video-id`, `--app-id` + `--container-id`, `--shield-zone-id` + `--id`): positional order would be ambiguous.
- Optional-id variants.
