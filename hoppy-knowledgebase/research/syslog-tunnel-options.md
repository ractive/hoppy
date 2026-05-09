---
title: Syslog tunnel options for hoppy container logs
type: research
date: 2026-05-09
tags:
  - research
  - syslog
  - tunnel
  - magic-containers
status: completed
---

# Syslog tunnel options for hoppy container logs

`hoppy container logs` runs a local syslog receiver and needs a way to make
it reachable from the bunny.net log-forwarding workers. This note compares
the candidate tunnel tools so future maintainers can revisit the trade-offs
without redoing the analysis.

Context: see [[iteration-24-container-logs-tunnel]] for the iteration plan
and [[bunny-api-quirks]] for why a tunnel is needed at all (Bunny exposes
no logs-fetch API — logs only travel out via syslog forwarding).

## Candidates

### bore

- **What:** Tiny Rust TCP-tunnel relay. `bore local 5514 --to bore.pub` exposes
  a local TCP port via a public host:port assigned by the relay.
- **License + maintenance:** MIT, actively maintained
  ([github.com/ekzhang/bore](https://github.com/ekzhang/bore)), single-binary,
  trivial to self-host (`bore server` on a VPS).
- **Status in hoppy:** **default**. Spawned as a child process when
  `--tunnel bore` (the default) is in effect.
- **Trade-off:** `bore.pub` is a third-party relay. Log lines traverse it.
  Mitigated by `--tunnel-host` (BYO) or `--bore-server <host>` (self-hosted).

### frp

- **What:** Reverse-proxy tunnel daemon (Go), supports TCP/UDP/HTTP, plugin
  ecosystem, ACLs.
- **License + maintenance:** Apache-2.0, actively maintained
  ([github.com/fatedier/frp](https://github.com/fatedier/frp)).
- **Status in hoppy:** **out-of-scope as default**. Heavier config (server +
  client TOML), no public free relay equivalent to `bore.pub`.
- **Trade-off:** Operators who already run `frpc` can plug it in via
  `--tunnel-host`; we don't ship integration.

### rathole

- **What:** Rust rewrite of frp's core, lower memory, also TOML-configured.
- **License + maintenance:** Apache-2.0, maintained
  ([github.com/rapiz1/rathole](https://github.com/rapiz1/rathole)).
- **Status in hoppy:** **out-of-scope**. No public relay; same operator
  burden as frp.
- **Trade-off:** Better fit than frp for self-hosting if the operator is
  already Rust-shop, but not differentiated enough vs `bore server` to
  justify a second built-in.

### ngrok

- **What:** Commercial tunnel service with a free tier; well-known brand.
- **License + maintenance:** Proprietary client, hosted SaaS, actively
  maintained.
- **Status in hoppy:** **future opt-in slot**. We may add a `Tunnel::Ngrok`
  variant if there's demand — it's the most operator-recognised name.
- **Trade-off:** Free tier rate-limits and rewrites the public hostname on
  every reconnect. Paid tier solves both. Closed-source client.

### `ssh -R`

- **What:** Built-in to OpenSSH. `ssh -R 5514:localhost:5514 user@vps`
  forwards the remote port back to the local receiver.
- **License + maintenance:** BSD, ubiquitous, no install step.
- **Status in hoppy:** **escape hatch via `--tunnel-host`**. Operators with a
  VPS already get this for free. We do not spawn `ssh` ourselves — too many
  config knobs (keys, agents, jump hosts).
- **Trade-off:** Requires the operator to have shell + a public host
  configured to permit `GatewayPorts yes`. Best for sensitive workloads.

### Cloudflare Tunnel (`cloudflared`)

- **What:** Cloudflare's reverse tunnel; primarily HTTP/S but supports TCP
  via `cloudflared access tcp`.
- **License + maintenance:** Apache-2.0 client, hosted SaaS, actively
  maintained.
- **Status in hoppy:** **out-of-scope**. Setup requires a Cloudflare
  account, a named tunnel, and DNS records — not a one-liner.
- **Trade-off:** Excellent for long-lived tunnels with custom hostnames;
  overkill for an ad-hoc 30-minute log session.

### Tailscale Funnel

- **What:** Exposes a Tailscale node on a public `*.ts.net` URL.
- **License + maintenance:** BSD client (tailscaled), hosted control plane,
  actively maintained.
- **Status in hoppy:** **out-of-scope**. HTTPS-only — Magic Containers
  syslog forwarding wants raw TCP. Funnel doesn't offer that today.
- **Trade-off:** Great for HTTP services on a Tailnet; wrong layer for
  syslog.

## Decision

Default = **bore** (small, simple, self-hostable, single binary, MIT). Easy
to install via `cargo install bore-cli` or `brew install bore-cli`. Easy to
self-host via `bore server` on a VPS plus `--bore-server <host>`.

Escape hatch = **`--tunnel-host <host:port>`** for `ssh -R`, frp, rathole,
Cloudflare Tunnel, ngrok, or any other tool that can hand the operator a
publicly-reachable `host:port` pointing at the local receiver port.

Future iterations may add a dedicated `Tunnel::Ngrok` variant if user
demand justifies a second built-in spawner; the abstraction in
`bunny-syslog-receiver` (separate from the tunnel implementation) keeps that
door open.

## Related

- [[iteration-24-container-logs-tunnel]] — implementation iteration
- [[bunny-api-quirks]] — Bunny has no logs-fetch endpoint
- [[decision-log]] — `bunny-syslog-receiver` crate naming decision (iter-24)
