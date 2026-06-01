---
title: Dogfooding session 2026-06-01 (round 2 — post iter-54..58)
type: dogfooding-session
date: 2026-06-01
tags:
  - dogfooding
  - session-log
status: completed
---

# Dogfooding session — 2026-06-01 (round 2)

**Tester:** Claude (autonomous), against `TEST_BUNNY_API_KEY`
**Binary:** `target/release/hoppy 0.3.0 (788732fa0df5 2026-06-01)` —
fresh build off `main` at `788732f`, after iter-54..58 all merged.

Goal: (1) verify the five iterations just landed, (2) probe less-trodden
surfaces (DNS export/dnssec/issue-cert, shield event-logs, container app
create edge cases, JSON casing) for fresh friction.

## Verification of iter-54..58

| Iteration | Check | Result |
|---|---|---|
| 54 | `shield metrics waf-rule --id <z> --rule-id <r>` | ✅ works |
| 54 | `--shield-zone-id` alias still works | ✅ works |
| 54 | `--help` shows `--id` (alias hidden) | ✅                       |
| 55 | `hoppy statistics --format json` byte-identical across runs | ✅ |
| 55 | Chart keys ascending date order | ✅ |
| 56 | `db config show --format table` two tables | ✅ |
| 56 | `db config show --format text` tab-separated | ✅ |
| 56 | `db config limits --format table` field/value | ✅ |
| 56 | `db v2 list --format table` empty → `No results.` + pageinfo footer | ✅ |
| 57 | `auth check --quiet` silent + exit 0 | ✅ |
| 57 | `auth check --quiet` bad key → error + exit 1 | ✅ |
| 57 | `pull-zone list --quiet` keeps table, drops hint | ✅ |
| 57 | `--quiet` help text explains predicate vs data | ✅ |
| 58 | `dns zone scan results --domain X` → `Domain: X` | ✅ |
| 58 | `dns zone scan results --id <z>` → resolved domain | ✅ |
| 58 | `--format json` includes `Domain` field | ✅ |

All five iterations verified clean.

## Fresh friction discovered

| Surface                                                | Finding                                                       |
|--------------------------------------------------------|---------------------------------------------------------------|
| `dns zone export`                                      | ❌ Empty output for empty zones (no header, no `;; empty`)    |
| `dns zone export --format json`                        | ❌ Ignores `--format`, always emits raw BIND text             |
| `dns zone issue-cert` undelegated                      | ⚠️ Returns generic `500 ()` — no friendly translation        |
| `shield event-logs` future date                        | ❌ Surfaces `401 Unauthorized`, discards `errorResponse.message` ("You can only view the past 3 days (72 hours)...") |
| `container app create --min -1`                        | ⚠️ Clap rejects negative int as "unexpected argument"        |
| `pull-zone list`                                       | ⚠️ Has `--page`/`--per-page` but no `--all` flag (cf. `shield event-logs --all`) |
| `pull-zone create --name`                              | ⚠️ Help description is empty (every other flag has one)      |
| `shield access-list create --type`                     | ℹ️ Numeric enum (0..5) — already filed                       |
| JSON casing                                            | ℹ️ Confirmed mixed: `shield access-list` snake_case, `pull-zone` PascalCase, `container app get` camelCase, `db v2` snake_case — already filed |

## New backlog items filed this round

- **MEDIUM** [[../backlog/shield-event-logs-discards-error-body]] — `errorResponse.message` returned by `/shield/event-logs/.../<date>/` 401 responses is dropped on the floor; the user sees only "401 Unauthorized" instead of "You can only view past 3 days".
- **MEDIUM** [[../backlog/dns-zone-export-ignores-format]] — `dns zone export --format json` always returns raw BIND text. Should wrap in `{"Bind": "..."}` (or sibling-shaped envelope) for machine consumers.
- **LOW**    [[../backlog/dns-zone-export-empty-zone-silent]] — Empty zones produce literally no output. At minimum a `;; empty zone <domain>` header.
- **LOW**    [[../backlog/dns-issue-cert-error-translation]] — Generic `500 ()` for an undelegated zone is the documented expected case. Translate to "Zone is not delegated to bunny.net nameservers — set NS records to <list> and retry".
- **LOW**    [[../backlog/container-app-create-negative-int-rejection]] — `--min -1` (and similar) fails with clap "unexpected argument" instead of a domain validation. Switch to `allow_hyphen_values` then reject in code with a clear message.
- **LOW**    [[../backlog/pull-zone-list-missing-all-flag]] — `pull-zone list` (and several other `list` commands) lack the `--all` auto-paginate flag that `shield event-logs` has. Sweep for parity.
- **LOW**    [[../backlog/pull-zone-create-name-help-empty]] — `--name` flag on `pull-zone create` has an empty help string.

## Notes

- Cleanup: created and deleted DNS zones 803455 / 803456 / 803457 / 803458;
  stream library 674387; pull zone 5945778. All confirmed deleted.
- Lots of `hoppy-edge-rule-*` / `hpst-*` / `hpmc-*` leftovers from prior
  iter-* test runs remain on the account — not a regression.
- The `--quiet` help-text wording introduced in iter-57 is a noticeable
  DX improvement: it explains the predicate/data split inline so the
  user doesn't have to look it up.

## Related

- [[session-2026-06-01]] — round 1 (the source of iter-54..58)
- [[../iterations/iteration-54-shield-metrics-flag-parity]]
- [[../iterations/iteration-55-deterministic-chart-ordering]]
- [[../iterations/iteration-56-db-format-cleanup]]
- [[../iterations/iteration-57-quiet-flag-contract]]
- [[../iterations/iteration-58-dns-scan-domain-column]]
- [[dogfooding-playbook]]
