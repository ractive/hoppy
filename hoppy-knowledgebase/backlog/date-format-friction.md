---
title: date/time flags accept inconsistent formats with poor errors
type: backlog
date: 2026-05-10
status: completed
priority: medium
origin: dogfooding-2026-05-10
---

# Date/time flag friction

## `shield event-logs --date MM-dd-yyyy`

US-style format only. Most CLIs accept ISO 8601 (`YYYY-MM-DD`). Either
accept ISO and convert client-side, or accept both formats.

## `db statistics --from / --to` requires RFC 3339

Passing `--from 2026-05-01` produces:

```
Error: HTTP 400 Bad Request: {"error":"Issue with query string",
  "details":"Failed to deserialize query string: from: premature end of input"}
```

The user has no obvious cue that the API wants
`2026-05-01T00:00:00Z`. Either:

- Validate the format client-side with a clear error pointing to RFC 3339,
  or
- Auto-pad date-only input to `T00:00:00Z` (more forgiving).

Same applies to `db usage --from/--to`. Help text says "RFC 3339" but the
upstream error message is unhelpful when the user gets it wrong.

## Suggested rule

Accept both `YYYY-MM-DD` (date-only, padded to 00:00:00 UTC) and full RFC
3339 across every time-window flag. Document the accepted formats in the
flag's `long_help`.
