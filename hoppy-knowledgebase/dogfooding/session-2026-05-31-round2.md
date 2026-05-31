---
title: Dogfooding session 2026-05-31 (round 2)
type: dogfooding-session
date: 2026-05-31
tags:
  - dogfooding
  - session-log
status: completed
---

# Dogfooding session — 2026-05-31 (round 2)

**Tester:** Claude (autonomous), against `TEST_BUNNY_API_KEY`
**Binary:** `target/release/hoppy 0.3.0 (6343897f9068 2026-05-31)` — same
pre-iter-43 binary as round 1; rebuild still deadlocked because two
orphaned `build-script-build` processes from earlier attempts continue to
hold cargo state and the auto-mode classifier blocked `kill -9` on them.

Round 1 focused on the pull-zone family and ran into the new iter-44..47
toggles. This round goes wider: surfaces I didn't touch round 1, edge
behaviours, and error-mapping quirks.

## New surfaces exercised

| Surface                                          | Result                              |
|--------------------------------------------------|-------------------------------------|
| `stream library create / get / statistics / delete` | ✅ Created + deleted `hoppy-test-stream-1780260460` (lib 673669) |
| `stream library get --reveal`                    | ❌ `ApiKey`/`ReadOnlyApiKey` dropped entirely |
| `stream collection create / list / delete`       | ✅ but auto-mode blocked the collection delete |
| `script create`                                  | 🛑 auto-mode blocked (resource creation) |
| `script list --format text` (empty)              | ⚠️ Silent (vs. `table` shows "No results.") |
| `db v2 list / active-usage / usage / live`       | ❌ All ignore `--format`, all snake_case JSON |
| `db ping`, `db versions`, `db group list`, `db token` | ✅ help texts clean |
| `shield waf profiles / list-rules`               | ✅                                  |
| `shield access-list list`                        | ✅ (VPN Providers, 10577 entries)   |
| `shield event-logs --id <z> --date <d>`          | ✅                                  |
| `shield metrics overview` (and siblings)         | ✅ flag is `--id`                   |
| `shield rate-limit list --shield-zone-id …`      | ✅ silently aliased to `--id`       |
| `container app get`                              | ✅ camelCase keys preserved         |
| `container app list / endpoint list / volume list / registry list` | ✅ |
| `container pod`                                  | ⚠️ Only `recreate`; no `list`       |
| `container log-forwarding list`                  | ✅ (empty)                          |
| `dns zone scan start --domain ractive.ch`        | ✅                                  |
| `dns zone scan results --domain ractive.ch`      | ❌ `--domain` rejected (start accepts it) |
| `completions zsh`                                | ✅ 17,835 lines                     |
| `pull-zone create` duplicate name                | ✅ clear `pullzone.name_taken` 400 |
| `pull-zone get --id 999999`                      | ✅ `404 pullZone.not_found`         |
| `storage-zone get --id 999999`                   | ⚠️ `HTTP 401 Unauthorized:` (trailing colon, no body) |
| `auth check` with bad key                        | ✅ `401 authentication.failed`      |
| Missing `BUNNY_API_KEY`                          | ✅ Helpful hint with fix            |
| `--record /tmp/rec auth check`                   | ❌ Writes payer email, invoice URL, balance to disk |

## New backlog items filed this round

- **HIGH** [[../backlog/stream-library-api-key-unrecoverable]] —
  `ApiKey`/`ReadOnlyApiKey` are `skip_serializing`, so `--reveal` cannot
  bring them back. A user creating a stream library has no way to use it
  from hoppy alone.
- **HIGH** [[../backlog/record-flag-leaks-billing-pii]] — `--record` on
  the canonical first-command (`auth check`) writes payer email,
  payment IDs, balance, and signed invoice URLs straight to disk. The
  playbook tells users to commit these fixtures.
- **MED** [[../backlog/dns-scan-results-rejects-domain]] — `scan start`
  accepts `--id` OR `--domain`; `scan results` only accepts `--id`. The
  pre-zone scan workflow has no follow-up.
- Updated [[../backlog/db-active-usage-ignores-format]] — broadened from
  a single command to the whole `db` v2-style family (`db v2 list`,
  `db usage`, `db live` all ignore `--format` and emit snake_case JSON).

## Confirmed findings from round 1 (still present)

- Geo-zone casing bug ([[../backlog/geo-zone-flags-casing-mismatch]]) —
  not retested this round, same binary.
- Shield 202+error swallow ([[../backlog/shield-202-error-swallowed]]) —
  not retested.

## Friction not yet a filed item (judgement calls)

- `pull-zone hostname load-free-cert` takes only `--hostname`; every
  other `hostname` subcommand takes `--id` + `--hostname`. The API
  doesn't need the PZ ID, but the *flag signature* asymmetry is a stumble.
- `container pod` has only `recreate` and no `list` — you need a pod ID
  from somewhere else (the dashboard?). Either add a `list` or document
  where to obtain pod IDs in the help text.
- `storage-zone get --id 999999` returns the literal string
  `HTTP 401 Unauthorized: ` (trailing colon + space). The bunny.net
  storage API returns 401 instead of 404 for not-found, so hoppy
  could detect "empty body + 401 + known-good API key" and rewrite the
  error as "storage zone <id> not found (or not owned by this key)".

## Confirmed *good* DX

- Error envelope for not-found is clean and machine-greppable on core
  endpoints (`videoLibrary.not_found`, `pullZone.not_found`,
  `pullzone.name_taken`).
- Missing-env-var message is *exemplary*: states the problem and gives
  the exact fix on the next line.
- `--debug` output is clean and shows request + response with secret
  masking like `<set, length=17>`.
- `pull-zone get --format text` collapses arrays with the
  drill-into-JSON tip — still the gold standard.
- Helpful drill-down tips: `tip: hoppy pull-zone get --id 5857625`
  after `list`.

## Resources created/destroyed this session

| Resource type | Name                              | Disposition |
|---------------|-----------------------------------|-------------|
| stream library | `hoppy-test-stream-1780260460` (673669) | Deleted |
| stream collection | `hoppy-test-col` (585345a8-…)  | **Leaked** (auto-mode blocked delete; container only) |
| pull zone | `hoppy-test-dupe-<ts>` (5940938)      | Deleted |

## Suggested round 3 (or future)

1. **Rebuild the binary** — once the orphan `build-script-build`
   processes are released, retest iter-44..47 toggles (firewall, vary,
   origin-host-header, rate-limit), then re-verify the geo-zone casing
   bug against a fresh response.
2. **Edge-script lifecycle.** Auto-mode blocked `script create` this
   round. With consent, exercise create → upload code → publish →
   release pin/unpin → variable set → secret set → rotate-deployment-key
   → delete. Variable/secret redaction behaviour is a likely friction
   site.
3. **Video upload via stream.** Requires the library ApiKey, so blocked
   on [[../backlog/stream-library-api-key-unrecoverable]] first.
4. **DNSSEC enable→key rotate→disable** on a real domain to verify
   [[../backlog/dnssec-status-text-output-thin]].
5. **Container app create with reserved runtime + volume + endpoint
   wiring** end-to-end to exercise the full container CRUD graph and
   the `region optimal` selector.

## Round comparison

Round 1 filed 6 backlog items. Round 2 filed 3 more (and broadened 1).
Net new findings 9; total open items written in 2026-05-31 dogfooding:
**10**. The pull-zone surface is the most thoroughly characterized; the
container + stream surfaces have the most remaining unknowns.
