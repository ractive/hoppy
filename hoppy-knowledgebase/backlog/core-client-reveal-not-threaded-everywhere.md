---
title: --reveal doesn't reach --debug output on some core-client commands
type: backlog
date: 2026-08-09
tags:
  - backlog
  - debug
  - dx
status: resolved
priority: low
origin: review-pr-94
---

# `--reveal` doesn't reach `--debug` output on some core-client commands

Iter-81 threaded `debug_reveal_secrets` through the client builders, but
14 call sites still construct the core client via the plain
`auth::core_client(debug, record)` wrapper, which pins reveal to `false`:
`account.rs` (billing/region/country/search/user — handlers don't receive
the global reveal flag), `auth.rs`, `dns.rs`, `container.rs`, `purge.rs`,
`storage.rs`, `statistics.rs`, `video_library.rs`, `stream.rs` (3).

Effect: on those commands `--debug --reveal` still prints `<set,
length=N>` placeholders in the `>>>`/`<<<` body dumps. **Fails safe** —
nothing leaks — but the flag silently under-delivers. The apikey path
(where it matters most) was fixed during the PR-94 review pass.

## Want

Thread the global reveal flag into the remaining handlers and switch
them to `core_client_with_reveal` (mechanical, mirrors what iter-81 did
for the other domain clients), then retire the reveal-less
`core_client()` wrapper so the compiler enforces the choice.

## Resolution (iter-84, 2026-08-13)

`ClientOpts` no longer derives `Default` — every construction must state
`reveal_secrets` explicitly, so this class of bug is now compiler-caught.
The redundant `core_client_with_reveal` alias was removed in favor of a
single `core_client(opts)`. `cli.reveal` is now threaded through: `auth
check`, `billing summary`/`payment-requests`/`*-pdf`, `region list`,
`country list`, `search`, `user audit`, `dns` (all subcommands), `purge`,
`statistics`, `video-library`, and the nested pull-zone cleanup client in
`container app delete --cascade`. See
[[iterations/iteration-84-backlog-fixes]].
