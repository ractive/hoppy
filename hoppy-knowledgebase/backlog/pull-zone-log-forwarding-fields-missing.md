---
title: pull-zone update doesn't expose CDN log-forwarding fields
type: backlog
date: 2026-05-15
status: open
tags:
  - pull-zone
  - log-forwarding
  - cli-coverage
---

# Pull-zone CDN log forwarding is not exposed by `pull-zone update`

The bunny.net core API exposes these fields on a Pull Zone (per
`specs/core-platform.json` lines 6291–6325):

- `LogForwardingEnabled` (boolean)
- `LogForwardingHostname` (string)
- `LogForwardingPort` (int32)
- `LogForwardingToken` (string)
- `LogForwardingProtocol` (enum)
- `LoggingSaveToStorage` (boolean — persistent logs to a storage zone)
- `LoggingStorageZoneId` (int64)

`hoppy pull-zone update --help` exposes **none** of these.  The only
`log`-related flag matched today is
`--optimizer-static-html-wp-bypass-cookie` (which contains the substring
"logged-in").

## Why it matters

This is arguably the *more common* "log forwarding" surface than the
Magic Containers one in `container log-forwarding`. CDN access-log
streaming is a standard use case (SIEM, observability pipelines).
Without it, users have to use the dashboard.

## Want

- `pull-zone update` flags:
  - `--log-forwarding-enabled <bool>`
  - `--log-forwarding-hostname <host>`
  - `--log-forwarding-port <port>`
  - `--log-forwarding-token <token>`
  - `--log-forwarding-protocol <Udp|Tcp>`
  - `--logging-save-to-storage <bool>`
  - `--logging-storage-zone-id <id>`
- A read path: `pull-zone get --format json` should already include
  these — verify after the update path lands.

## Out of scope

- A `pull-zone logs tail` streaming command similar to
  `container logs` — useful, but a separate feature. The CDN doesn't
  push logs to us; it forwards to a configured syslog endpoint, so
  any "tail" command would need the same bore/tunnel plumbing the
  container path already has.
