---
title: storage-zone create --format json shows Password "string" placeholder
type: backlog
date: 2026-05-15
status: resolved
tags:
  - storage
  - json
  - redaction
---

# `storage-zone create --format json` returns `"Password": "string"`

## Observed

```sh
hoppy storage-zone create --name hoppy-test-sz-... --region DE --format json
```

```json
{
  "Id": 1517186,
  ...
  "Password": "string"
}
```

The literal word `"string"` looks like an OpenAPI placeholder leaking
through. A subsequent `hoppy --reveal storage-zone get --id ... --format json`
returns a real 41-char password — so the data exists, but the **create**
endpoint either omits it or returns a generic schema placeholder.

## Expected

One of:

- The real password (consistent with `get --reveal`).
- An explicit `"<set, length=41>"` redaction marker (as the table format
  uses).
- A `null` (with a note that you must call `get --reveal` to fetch it).

The current `"string"` value is the worst of all worlds: it looks like a
real value, but copying it does nothing.

## Why it matters

A common workflow is "create the storage zone, capture its password into
an env var, hand off to a CI job". Today that requires a second call:

```sh
hoppy storage-zone create ... --format json   # password is "string" — useless
hoppy storage-zone get --id ... --reveal --format json | jq -r .Password
```

## Resolution (2026-08-09)

Stale — already fixed exactly as proposed:
`commands/storage_zone.rs:182-209` no longer prints the create response;
it immediately re-fetches the zone and prints real credentials
(unconditionally revealed on create; `get` still redacts without
`--reveal`). If the follow-up fetch fails, the error points at
`storage-zone get --id <id>`. Live-verified in the 2026-08-09 dogfood
round (see dogfooding notes).
