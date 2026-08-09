---
title: Magic Containers client --debug omits the request body
type: backlog
date: 2026-08-09
tags: [backlog, containers, debug, dx]
status: open
priority: low
---

# Magic Containers client `--debug` omits the request body

## Observed (2026-08-09 dogfood, iter-81)

`hoppy --debug container log-forwarding create ...` prints the request
URL (`>> POST https://api.bunny.net/mc/log/forwarding`) and the response
(`<< 400 Bad Request`), but no `>>>` request-body lines. The core-platform
client prints the body fine (verified same session with
`--debug storage-zone create`, which shows `>>> {"Name": ..., "Region": ...}`).

The May 2026 repro in [[backlog/log-forwarding-create-empty-400]] did show
`>>>` for the same command, so this is a regression in the mc client's
debug plumbing (or the mc client was switched to a code path that never
gained it when [[backlog/debug-flag-omits-request-body]] was fixed).

## Why it matters

Debugging the still-open empty-400 log-forwarding issue requires seeing
exactly what we send. Low priority otherwise.

## Want

- `--debug` on mc-client requests prints the serialized request body,
  same as the core client.
