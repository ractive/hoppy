---
title: container log-forwarding create returns 400 (empty body) on every app — including freshly created ones
type: backlog
date: 2026-05-15
status: open
tags:
  - log-forwarding
  - containers
  - error-handling
  - blocker
---

# `container log-forwarding create` consistently returns HTTP 400

The marquee feature `hoppy container logs` is **non-functional** end-to-end
against this account. Every `POST /mc/log/forwarding` call returns 400 with
an empty body.

## Repro (2026-05-15 dogfooding session)

Spun up a fresh nginx:alpine container app in DE region, runtime Shared,
min=max=1. After waiting for `status=active` and `containerInstances=1`:

```sh
hoppy --debug container log-forwarding create \
  --app-id Y96syunVT8TGxn2 \
  --forwarding-type SyslogTcp \
  --endpoint 127.0.0.1 --port 5514 \
  --syslog-format SyslogRfc5424 --enabled --format json

>> POST https://api.bunny.net/mc/log/forwarding
>>> {"app":"Y96syunVT8TGxn2","type":"SyslogTcp","endpoint":"127.0.0.1",
     "port":5514,"format":"SyslogRfc5424","enabled":true}
<< 400 Bad Request
<<<     (empty body)
```

Same 400 against the 13 pre-existing (leaked) container apps. Same 400 via
`hoppy container logs --app-id <id>` with the default `bore` tunnel.

## Shape variants tried — all 400

| Variant                         | Result |
|---------------------------------|--------|
| `"type":"SyslogTcp"` (current)  | 400, empty body |
| `"type":"syslogTcp"` (camelCase)| 400, empty body |
| `"type":1` (integer enum)       | 400, empty body |
| With `"productId":null` added   | 400, empty body |
| With `"token":"dummy"` added    | **401 Unauthorized** (surprising — different code path?) |
| Hostname endpoint, enabled=false| 400, empty body |
| UDP forwarding, port 514        | 400, empty body |

The 401 with a token field suggests adding `token` triggers a different
server-side path (possibly auth-checked). Could mean `token` is required
even though our spec says it's optional, or it triggers a stricter
validation path that returns a slightly different error class.

## Hypotheses to investigate next

1. **Feature gate on account plan.** Magic Containers log forwarding may
   require a non-free subscription tier. The test account shows
   `Balance: $0.00, Monthly Bandwidth: 0 GB` — possibly under-provisioned.
   Check upstream docs / dashboard whether log forwarding is a paid
   add-on.
2. **Missing required field not in the spec.** Our `LogForwardingRequest`
   has `app, type, endpoint, port, format, enabled, token?`. Real API
   may want `productId`, `name`, `description`, or similar.
3. **Endpoint validation.** `127.0.0.1` was rejected; `logs.example.com`
   was also rejected. Maybe the API resolves DNS at validation time and
   `example.com` is filtered. Try a real reachable host.
4. **App needs an HTTP endpoint first.** The freshly created app had
   `displayEndpoint: null` (no exposed endpoint, just nginx running
   internally). The leaked apps DID have a displayEndpoint and still
   failed, so this is less likely — but worth confirming with a
   `container endpoint add` step in the repro.
5. **`bore.pub` host being blocked.** The bore-tunnel path produces a
   hostname like `bore.pub:<port>`; if Bunny filters known public tunnel
   hosts that would explain 400 with no body. Try `--tunnel-host` with a
   server you own.

## Smaller related issues found

- `container log-forwarding delete --app-id <id>` against an app with no
  config returns `Error: API error (HTTP 404 Not Found):` — expected, but
  the cleanup path in `container logs` (which always runs on Ctrl-C / error)
  will hit this on every aborted session and the error swallowing assumes
  404=ok. Verify the cleanup path is 404-tolerant.
- `container log-forwarding get` returns 404 for an app with no config.
  Friendlier scripting would return 200 + `null`.
- `container log-forwarding delete --id <id>` (with `--id`) is rejected;
  only `--app-id` is accepted. Inconsistent with other delete commands.

## Validated working separately

- `container logs --tunnel none` starts a local syslog listener on
  `0.0.0.0:<port>` and prints the address. Tunnel/listener plumbing is
  fine. Only the LF-create step fails.
- `container logs` with `bore` (after `cargo install bore-cli`) opens
  the tunnel successfully, then fails at LF-create with the same 400.
- `cargo install bore-cli` from this `--debug` run took ~2 minutes.
- The "bore missing" error message is excellent (names both install
  paths and both fallbacks).

## Suggested next action

Open an issue or support ticket with bunny.net showing the request body
and the empty-400 response. The empty body is the real blocker — without
a server-side reason it's almost impossible to fix from our side.

Until then, document `hoppy container logs` as **known-broken** in the
README and dogfooding playbook so users don't waste time on it.
