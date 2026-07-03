---
title: "pull-zone update: --log-forwarding-hostname silently no-ops when LogForwardingEnabled=false"
type: backlog
date: 2026-05-26
status: resolved
priority: medium
origin: dogfooding-2026-05-26 (post-iter-38)
---

# `--log-forwarding-hostname` silently no-ops on disabled zones

When `LogForwardingEnabled` is `false` on a pull zone, a CLI update like:

```sh
hoppy pull-zone update --id <id> --log-forwarding-hostname syslog.new.example
```

returns HTTP 200 and prints the standard success table — but the response
body shows `LogForwardingHostname` is unchanged. The API silently rejects
hostname-only updates while the feature is disabled, and the CLI does not
surface this.

Combined updates *do* work:

```sh
hoppy pull-zone update --id <id> \
    --log-forwarding-enabled true \
    --log-forwarding-hostname syslog.new.example
```

→ hostname is applied.

## Reproduction

1. Pull zone with `LogForwardingEnabled=false`.
2. `hoppy pull-zone update --id <id> --log-forwarding-hostname X` → 200 OK.
3. `hoppy pull-zone get --id <id>` → hostname unchanged.

## Hypothesis

The bunny.net API ignores log-forwarding sub-fields when the master
`LogForwardingEnabled` flag is `false`. The CLI client should either:

- pre-validate and refuse the update with a clear error
  (`hint: pass --log-forwarding-enabled true to update hostname`), **or**
- post-validate the response and warn when a passed field did not change.

## Out of scope

Whether the API behaviour is documented or considered a bug upstream.
We just need the CLI to not silently swallow user intent.

## Resolution

Fixed in [[iterations/iteration-39-dogfooding-2026-05-26-fixes]].
