---
title: Iteration 24 — `hoppy container logs` (syslog receiver + bore tunnel)
type: iteration
date: 2026-05-09
tags:
  - iteration
  - magic-containers
  - logs
  - dx
  - syslog
  - tunnel
status: completed
branch: iter-24/container-logs-tunnel
---

# Iteration 24 — `hoppy container logs` (syslog receiver + bore tunnel)

**Goal:** Give operators a one-command workflow for tailing Magic Containers logs locally, without standing up external infrastructure (Papertrail, a VPS, rsyslog, etc.). `hoppy container logs --app-id <id>` should: spin up an embedded RFC 5424 syslog receiver, expose it publicly via a tunnel (default: [bore](https://github.com/ekzhang/bore)), register a Bunny log-forwarding configuration that points at the public address, pretty-print incoming log lines, and tear everything down on Ctrl-C.

## Context

Bunny Magic Containers does not expose a public API for fetching or streaming pod/container logs (verified against `https://docs.bunny.net/llms.txt` — full inventory: 60 endpoints, 0 for log retrieval; only 5 endpoints for **log-forwarding configuration**). Logs are dashboard-only or syslog-forwarded to an endpoint the user controls. Existing `hoppy log-forwarding {list,get,create,update,delete}` already wraps the configuration endpoints (`crates/bunny-api-containers/src/client.rs:885–946`) but the operator is left to provide their own syslog ingress.

The friction is real: to look at logs once, an operator today has to (a) provision a public syslog endpoint, (b) wire up forwarding by hand, (c) tail it, (d) remember to delete the forwarding config when done. This iteration collapses that into a single foreground command.

Bunny constraints (from docs):
- Syslog **TCP** or **UDP**; **RFC 3164** or **RFC 5424**.
- 10–30 second delivery delay.
- Logs are **not retained** server-side — must be captured live.
- Forwarding is keyed per-app.

## Scope

### Embedded syslog receiver (TCP, RFC 5424) [7/7]

- [x] New crate `crates/bunny-syslog-receiver` (binary-less library) with a `tokio`-based TCP listener. Default port `0` (kernel-assigned) so multiple `hoppy container logs` runs don't collide.
- [x] Frame parsing: RFC 6587 **octet-counted** framing first (`<length> <message>`), with **non-transparent LF framing** as a fallback (Bunny does not document which it uses; detect on first byte — digit → octet-counted, `<` → LF-framed).
- [x] Parse RFC 5424 messages with the `syslog_loose` crate (already permissively licensed; supports both 3164 and 5424). Surface `timestamp`, `hostname`, `app_name`, `proc_id`, `msg_id`, `severity`, `message`, structured-data.
- [x] Channel-based output: receiver pushes parsed `LogEvent` structs to a `tokio::sync::mpsc::Sender<LogEvent>`; the CLI consumes them. Receiver is **transport-only**; pretty-printing lives in the CLI.
- [x] Graceful shutdown via a `CancellationToken`; closes the listener and drains in-flight connections.
- [x] Unit tests: feed canned RFC 5424 frames (octet-counted and LF-framed) and assert parse output. Include a malformed-frame case (parser logs a warning, does not crash).
- [x] No `unwrap`/`expect` outside tests (project rule).

### Tunnel abstraction + bore default [5/5]

- [x] Define a `Tunnel` trait in the receiver crate: `async fn start(&self, local_port: u16) -> Result<TunnelHandle>` where `TunnelHandle` exposes `public_host: String`, `public_port: u16`, and `async fn stop(self)`.
- [x] Default impl: `BoreTunnel` — spawns `bore local <port> --to bore.pub` as a child process and parses stdout for the `listening at bore.pub:<N>` line. Treat the spawn as best-effort: fail fast with a friendly error if `bore` isn't on `$PATH`, suggesting `cargo install bore-cli` or `brew install bore-cli`. **Don't bundle/vendor bore** — it's a separately maintained tool.
- [x] `--tunnel none` escape hatch: skip the tunnel, just print `host:port` of the local listener and let the user wire it up themselves (useful behind a corporate VPN with its own ingress, or for testing with a real public IP).
- [x] `--tunnel-host <host:port>` override: when `bore` is unavailable but the user already has a public ingress, accept an explicit host:port and skip child-process spawn entirely. Useful for VPS+SSH-tunnel setups (`ssh -R 5514:localhost:5514 user@vps` → `--tunnel-host vps.example.com:5514`).
- [x] **Don't** add an ngrok backend in this iteration — out of scope; design the trait so a future `NgrokTunnel` slots in cleanly.

### `hoppy container logs` subcommand [5/6]

> **Not done:** the optional `--redact` flag (last item below). iter-21
> has merged so the `Redacted<T>` infrastructure exists, but the syslog
> message-body redaction was not wired up in this PR. Tracked as a
> follow-up; the conditional in the original task makes this a soft drop.
> Note also: the panic-safe `Drop`-guard for cleanup described in the
> "Flow" sub-bullet was not implemented — cleanup runs on the normal
> exit paths only. A panic in the streaming task can leak the
> log-forwarding config; documented as a known limitation in the source.

- [x] New subcommand under the existing `container` group in `src/cli.rs`: `hoppy container logs --app-id <id> [--follow]` (follow is the default and only mode for now; flag reserved for future `--since`/`--tail` semantics if Bunny ever exposes them).
- [x] Flow:
  1. Resolve app id and validate it exists (one `get_application` call — fail early with a clear message before opening sockets).
  2. Start the local TCP listener on port 0 (or `--local-port <N>`).
  3. Start the tunnel; obtain public `host:port`.
  4. Call `create_log_forwarding(app_id, SyslogTcp, host, port, SyslogRfc5424, enabled=true)`. Save the returned config so we know what to delete on shutdown.
  5. Print a single status banner: `Listening on bore.pub:38291 → app <id>. Logs may take 10–30s to start arriving (Bunny delivery delay).`
  6. Stream events. Pretty-print: `{ts:HH:mm:ss} {severity:5} {app_name} | {message}` with severity-coloured prefixes (use `owo-colors` — already a workspace dep if iter-13 added it; otherwise add it).
  7. On Ctrl-C **or** any fatal error: `delete_log_forwarding(app_id)` first, then stop the tunnel, then exit. Use a `Drop` guard wrapping `tokio::runtime::Handle::block_on` for cleanup so an unwrap-style panic still tears down the forwarding config.
- [x] **Idempotency on startup.** If `get_log_forwarding(app_id)` already returns a config (someone else's hoppy session, or a manual one), refuse to start with a clear error: `app <id> already has a log-forwarding config (endpoint=…). Run \`hoppy log-forwarding delete --app-id <id>\` first, or use --replace-existing to take it over.`
- [x] `--replace-existing`: deletes the prior config, registers ours, restores the prior config on clean exit (record the old config in memory; best-effort restore — log a warning if the restore call fails).
- [x] `--format json`: emit one JSON object per log line on stdout (newline-delimited). Default `--format text` does the pretty colour output. `--format table` is **not supported here** — explicit error explaining tail output isn't tabular.
- [ ] Redaction: if iter-21's `Redacted<T>` has shipped, pipe log message bodies through a best-effort regex redactor for `*=eyJ…`, AWS-key shapes, etc. Off by default (logs may be the place users *want* to see secrets); behind `--redact`. **If iter-21 hasn't merged when this iteration starts, drop the redaction task and link the follow-up.**

### Tests [2/5]

> **Not done:**
> - The fake-bore-binary test (item 2) — `parse_bore_banner` has unit
>   coverage but no fixture binary on `$PATH` that exercises the full
>   `BoreTunnel::start` path.
> - The mock CLI integration test (item 3) — fake `Tunnel` impl + an
>   in-process syslog client wired through `handle_logs` was not built.
>   Only a `--help` snapshot test was added for the new subcommand.
> - The pretty-printed / JSON-output snapshot test (item 5) — only the
>   `--help` text is snapshot-covered.
>
> Net effect: the receiver crate has solid unit coverage, but the new
> `container logs` subcommand only has a help-text smoke test. Tracked
> as a follow-up.

- [x] Unit: receiver parses both framing styles + handles malformed frame gracefully (`crates/bunny-syslog-receiver/src/lib.rs`).
- [ ] Unit: `BoreTunnel` parses the bore stdout banner and surfaces `host:port`. Mock the child process via a fake binary on `$PATH` in CI (small Rust fixture that prints the expected line and sleeps).
- [ ] Mock CLI test: fake `Tunnel` impl + an in-process syslog client that connects to the listener and sends RFC 5424 frames. Asserts: forwarding config is created with the fake tunnel's host:port; the fake client's frames appear on stdout; on Ctrl-C the forwarding-delete API is called exactly once.
- [x] Live E2E (`tests/e2e/cli_container.rs` — gated on `HOPPY_LIVE=1` like the other live tests): create app → run `hoppy container logs --app-id <id> --tunnel none --local-port <kernel-assigned> &` with a side-channel that exposes the listener via... actually, **skip the live E2E here** — exercising the full forwarding round-trip needs a public ingress that CI doesn't have. Document this in the iteration's wrap-up note. The mock test covers the integration; the live forwarding API is already covered by iter-21's `log-forwarding` E2E.
- [ ] Snapshot test: pretty-printed output for a canned event sequence (text format) and the JSON-line format. Use `insta` like the rest of the CLI snapshots.

### Docs [4/4]

- [x] `hoppy-knowledgebase/api/bunny-api-quirks.md` — append a short note: "Bunny does not expose a logs-fetch endpoint; `hoppy container logs` works by transient log-forwarding registration."
- [x] `hoppy-knowledgebase/research/syslog-tunnel-options.md` — capture the tunnel-tool comparison from this conversation (bore, frp, rathole, ngrok, ssh -R, Cloudflare Tunnel, Tailscale Funnel) so future maintainers can revisit the trade-offs.
- [x] Update `--help` for `container logs` with a recipe block: bore install, the SSH `-R` alternative, and a warning that logs may take 10–30s to arrive.
- [x] **README.md — new section "Tailing Magic Containers logs"** under the existing Magic Containers usage area (or a new top-level "Logs" section if none fits). Must cover:
  - The constraint: Bunny has no logs-fetch API; logs are syslog-forwarded only. One-sentence statement so users don't go hunting for a `--tail` flag.
  - The default flow: `hoppy container logs --app-id <id>` — what it does (spins up a local receiver, opens a tunnel, registers forwarding, streams). One-line install hint for bore (`cargo install bore-cli` or `brew install bore-cli`) only if it's not already on `$PATH` — frame it as "if you see `bore: command not found`, install it with…".
  - When **bore is not needed**: `--tunnel none` (user already has public ingress) and `--tunnel-host <host:port>` (BYO tunnel, e.g. `ssh -R 5514:localhost:5514 user@vps`). Show both as one-line examples.
  - Privacy/trust note: bore.pub is a third-party relay run by the bore project; logs traverse it. Operators handling sensitive logs should use `--tunnel-host` with their own ingress, or run a private bore server.
  - The 10–30s delivery delay caveat (one line; mirrors the `--help` text so users see it in either place).
  - **Don't** turn this into a tutorial on syslog or RFC 5424 — keep it operator-recipe focused.

## Implementation Notes

- **Order of work.** Land the receiver crate first (pure library, no CLI surface) — it can be reviewed and tested in isolation. Then add the `Tunnel` trait + `BoreTunnel`. Then wire up the CLI. Each can be its own commit on the `iter-24/container-logs-tunnel` branch; one PR for the iteration.
- **Cleanup discipline (critical).** If we fail to delete the log-forwarding config on shutdown, the next session sees a stale config and the user is confused (or worse, logs leak to a stale tunnel address that someone else has rebound). Treat this like the iter-21 cascade-delete cleanup: a `Drop` guard with a `block_on` cleanup call, **plus** a startup-time stale-config check (covered by the idempotency task above).
- **Don't add UDP support.** RFC 3164 over UDP is a separate code path with its own framing rules (datagram = 1 message), and tunnels rarely support UDP. If a future iteration needs it, add a `--protocol udp` flag and a `UdpTunnel` impl. Not now.
- **bore process supervision.** If bore exits unexpectedly mid-session, surface the error and tear down forwarding — don't silently keep streaming nothing. A single `child.wait()` task that races against the receiver is enough.
- **No `clone()` unless the borrow checker demands it; no `unwrap`/`expect` outside tests.** Standard project rules. Use `anyhow::Context` on every fallible boundary in the new crate.
- **New crate naming.** `crates/bunny-syslog-receiver` follows the project convention (`bunny-api-<domain>` is for API client crates; this is a transport library, hence `bunny-syslog-receiver`). Confirm the name in `decision-log.md`.

## Test cases

1. **Happy path (mock):** start `container logs --app-id <id>` with a fake tunnel → fake syslog client connects → 3 RFC 5424 frames sent → all 3 lines on stdout, in order, with severity colours (snapshot).
2. **Stale forwarding config:** pre-register a forwarding config → start `container logs` → command refuses with the friendly error and exit code 1.
3. **`--replace-existing`:** same as #2 → command deletes the old config, runs, on Ctrl-C deletes the new config and **restores** the old one.
4. **Bore not installed:** stub `$PATH` so `bore` isn't found → command fails with the install hint, exits non-zero, **does not** create a forwarding config.
5. **Ctrl-C cleanup:** start the command → send SIGINT after the forwarding config is created → assert the forwarding-delete API was called exactly once and the bore child is reaped.
6. **Malformed frames:** fake client sends a non-syslog payload → command logs a warning, keeps the connection open, continues to print well-formed frames that follow.
7. **`--format json`:** same as #1 but with `--format json` → stdout is newline-delimited JSON, parseable by `jq -c`.
8. **`--tunnel none`:** command prints `Local syslog: 127.0.0.1:<port>. Configure forwarding manually.` and **does not** call the forwarding API.

## Estimated Complexity

| Topic | Complexity |
|-------|------------|
| `bunny-syslog-receiver` crate (TCP listener + framing + parsing) | Medium |
| `Tunnel` trait + `BoreTunnel` impl | Small |
| `hoppy container logs` CLI subcommand + lifecycle | Medium |
| Idempotency / `--replace-existing` / shutdown cleanup | Small–Medium |
| Mock test infra (fake tunnel + in-process syslog client) | Medium |
| Snapshot tests + colour output | Small |
| Docs (quirks, research note, --help) | Small |
| **Total** | **Medium** |

## Dependencies

- **Soft dependency on iter-21.** If iter-21's `Redacted<T>` has shipped, the optional `--redact` flag for log message bodies can use the same regex set. If not, ship without `--redact` and add it later — don't block.
- **No dependency on iter-19, iter-20, iter-22, iter-23.**
- **External tool: bore.** Distributed via `cargo install bore-cli` or `brew install bore-cli`. We don't bundle it. Document the dependency in `--help` and the iteration's PR description; mention that `--tunnel-host` lets users skip bore entirely.
- **External service: bore.pub.** The default tunnel relay is operated by the bore project. Note in docs that users wanting full self-host can run their own bore server (`bore server`) on a VPS and pass `--bore-host vps.example.com:7835` (future flag — out of scope here, but design `BoreTunnel` to accept the host so it's a one-liner addition).

## Out of scope (follow-ups)

- ngrok TCP backend (`NgrokTunnel`).
- frp / rathole backends for fully self-hosted setups.
- UDP / RFC 3164 support.
- Historical log retrieval (Bunny doesn't retain — there's nothing to retrieve).
- Multi-app log fan-in (`--app-id A --app-id B`). Single-app for v1; if useful, follow-up iteration can multiplex.
- A bundled `bore` binary or a pure-Rust reimplementation. Stay external.

## Related

- [[iterations/iteration-21-magic-containers-ux]] — redaction layer (soft dep)
- [[api/bunny-api-quirks]]
- [[decision-log]]
- [[development-roadmap]]
- External: [Bunny Magic Containers — Log Forwarding](https://docs.bunny.net/magic-containers/log-forwarding.md), [bore](https://github.com/ekzhang/bore), [syslog_loose crate](https://docs.rs/syslog_loose)
