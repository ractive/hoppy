---
title: shield error responses surface as "Shield API error 0: unknown"
type: backlog
date: 2026-05-10
status: planned
priority: high
origin: dogfooding-2026-05-10
---

# Shield error mapping drops the API message

`hoppy shield api-guardian get --shield-zone-id <id>` returns
`Error: Shield API error 0: unknown` when the upstream response is:

```json
{
  "error": {
    "statusCode": 404,
    "success": false,
    "message": "No API Guardian configuration found for this shield zone.",
    "errorKey": "api_guardian.get_api_guardian.no_configuration"
  },
  "data": null
}
```

The user gets none of that. The pull-zone surface formats errors as
`bunny.net API error 400 (model.invalid): <message>` — shield should follow
the same shape (status code, errorKey, message).

## Likely cause

In the shield client the error mapper probably looks for fields at the top
level (`ErrorKey`, `Message`) — but the shield API wraps them under
`error: { statusCode, errorKey, message }`. Status `0` is the default for an
unmatched mapping.

## Fix

Add a shield-specific error type that deserialises the nested envelope, and
include the embedded `statusCode` rather than `0`. Also surface `errorKey`
in the formatted message. Apply uniformly across shield subcommands —
`bot-detection`, `upload-scanning`, `api-guardian`, `event-logs` etc. all
likely share the issue.
