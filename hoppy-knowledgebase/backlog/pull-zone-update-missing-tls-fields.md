---
title: "`pull-zone update` doesn't expose --enable-tls-1 / --enable-tls-1-1 (API supports them)"
type: backlog
date: 2026-05-31
status: planned
priority: medium
origin: dogfooding-2026-05-31 (post-iter-42)
---

# `pull-zone update` can't toggle TLS 1.0 / 1.1

The bunny.net API returns `EnableTLS1` and `EnableTLS1_1` on every pull
zone, but `hoppy pull-zone update --help` has no flags for them:

```
hoppy pull-zone update --id 5857625 --enable-tls-1-1 false
error: unexpected argument '--enable-tls-1-1' found
```

A search of the update help text turns up zero mentions of TLS:

```sh
hoppy pull-zone update --help | grep -iE "tls|ssl"
# only output:  - tcp-encrypted: TLS-encrypted TCP (value 2)
# (that's the log-forwarding protocol enum, unrelated)
```

This matters because disabling deprecated TLS 1.0/1.1 is a baseline
compliance/security requirement many CDN users have to meet (PCI DSS,
SOC 2, etc.). Without CLI exposure, users have to use the dashboard.

## Fix

Add `--enable-tls-1` and `--enable-tls-1-1` (Option<bool>) to
`PullZoneAction::Update` in `crates/hoppy-cli/src/cli.rs`, wire them
through in `crates/hoppy-cli/src/commands/pull_zone.rs`, and add the
corresponding fields to the API client's `UpdatePullZone` payload in
`crates/bunny-net-api/src/core/...`.

Reference: bunny.net Pull Zone schema in `specs/core-platform.json`
(grep for `EnableTLS`).

## Audit

While doing this, check whether other `Enable*`/`Disable*` flags from
the Pull Zone response are similarly unexposed by `update`:

```sh
hoppy --format json pull-zone get --id <id> \
  | jq 'to_entries | map(select(.key | startswith("Enable") or startswith("Disable")))' \
  | jq '.[].key'
```

…and compare against `pull-zone update --help` flag names. Pull-zone is
the largest configuration surface; gaps elsewhere may be smaller but
worth a once-over.

## Out of scope

- The same audit on storage-zone update / dns zone update / etc.
  Worth a follow-up if this turns up a wider pattern.
- Validating the JSON shape iter-39 `--debug` improvements expose
  during update — already addressed.
