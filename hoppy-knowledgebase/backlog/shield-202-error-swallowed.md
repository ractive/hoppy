---
title: Shield 202-with-error responses are reported as "No results", hiding the upgrade message
type: backlog
date: 2026-05-31
status: planned
priority: high
origin: dogfooding-2026-05-31
tags:
  - cli
  - shield
  - error-handling
  - dx
---

# Shield API 202 + error envelope is silently swallowed

bunny.net's Shield API returns **HTTP 202** with a payload like

```json
{
  "data": null,
  "error": {
    "statusCode": 202,
    "success": false,
    "message": "Unable to make changes whilst on the Basic tier of Bunny Shield. Please upgrade to Advanced to enable Bot Detection.",
    "errorKey": "invalid_plan_type.bot_detection"
  }
}
```

…when a feature is gated by plan tier. hoppy currently treats this as a
successful read with empty data and prints:

```
No bot detection data returned.
```

The user has no way to learn that the call was actually rejected because of
their plan tier — they think the data is genuinely absent or that the CLI is
broken. Even `--format json --debug` shows the raw payload in stderr but
stdout is still the misleading prose line.

## Repro

```sh
hoppy shield bot-detection get --shield-zone-id 118829 --debug --format json
# >> GET https://api.bunny.net/shield/shield-zone/118829/bot-detection
# << 202 Accepted
# <<< {"data":null,"error":{"statusCode":202,...,"errorKey":"invalid_plan_type.bot_detection"}}
# No bot detection data returned.
```

Almost certainly the same path is hit by `shield rate-limit list --shield-zone-id 118829`
(which also returned `No results.` on a Basic-tier zone) and probably
other plan-gated shield endpoints.

## Suggested fix

In the shield client/response handler:

1. Treat HTTP 202 with `error.success == false` as an error, not an empty
   success. Map `errorKey` like `invalid_plan_type.*` to a typed
   `ShieldError::PlanUpgradeRequired { feature, message }`.
2. CLI surface should print the upstream `message` verbatim on stderr
   and exit non-zero. Example:
   ```
   Error: bot detection requires an Advanced Shield plan (errorKey: invalid_plan_type.bot_detection)
   ```
3. For `--format json`, emit the envelope's `error` object directly under a
   top-level `error` key.

## Related

- [[shield-api-error-mapping]]
- [[debug-flag-omits-request-body]]
