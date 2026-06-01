---
title: shield event-logs discards the rich errorResponse body on 401
type: backlog
date: 2026-06-01
status: planned
priority: medium
origin: dogfooding-2026-06-01-round2
tags:
  - cli
  - shield
  - errors
  - dx
---

# `shield event-logs` swallows the real error message

When you query event logs outside the supported window, the API
returns a 401 with a meaningful body:

```json
{
  "logs": null,
  "hasMoreData": false,
  "continuationToken": null,
  "startToken": null,
  "errorResponse": {
    "statusCode": 401,
    "success": false,
    "message": "You can only view the past 3 days (72 hours) of Event Logs.",
    "errorKey": "invalid_datetime_window.event_logs"
  }
}
```

But the CLI prints only:

```text
Error: Shield API returned status 401 Unauthorized
```

The `errorResponse.message` and `errorKey` are dropped. This is
adjacent to iter-50's 202-error envelope work but applies to the
`/shield/event-logs/.../<date>/` endpoint which apparently uses a
different envelope shape (`errorResponse` sub-object vs top-level
`message`/`errorKey`).

## Reproduction

```sh
hoppy --debug shield event-logs --id <shield-zone> --date 2099-01-01
# << 401 Unauthorized
# <<< {"logs":null,"errorResponse":{"message":"You can only view the past 3 days (72 hours)...","errorKey":"invalid_datetime_window.event_logs"}}
# Error: Shield API returned status 401 Unauthorized
```

## Fix

In the Shield client's error mapping, when the response body deserialises
to `{ "errorResponse": { ... } }`, surface
`errorKey: message` in the rendered error string (same shape as
iter-50's `Shield API error <status> (<key>): <message>`).

Audit other Shield endpoints for the same envelope:

- `/shield/event-logs/.../<date>/`
- any sibling endpoint that returns `errorResponse` rather than
  the top-level shape iter-50 handled

## Acceptance

- `hoppy shield event-logs --id <z> --date 2099-01-01` prints
  `Error: Shield API error 401 (invalid_datetime_window.event_logs):
  You can only view the past 3 days (72 hours) of Event Logs.`
- Existing 401-no-body responses still print a graceful fallback.
- Same envelope handling applied to any sibling Shield endpoint that
  returns the `errorResponse`-wrapped shape.
