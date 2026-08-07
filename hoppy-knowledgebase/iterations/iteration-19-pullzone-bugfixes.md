---
title: Iteration 19 — Pull Zone bug fixes & CLI UX polish
type: iteration
date: 2026-05-05
tags:
  - iteration
  - bugfix
  - cdn
  - ux
  - dx
status: completed
branch: iter-19/pullzone-bugfixes
---

# Iteration 19 — Pull Zone bug fixes & CLI UX polish

**Goal:** Address real-world hoppy bugs and UX gaps surfaced by two field reports from `wardrobe-assistants.ch` provisioning work. Two are blocking (no Storage-Zone-backed Pull Zone create, deserialization panic on Magic-Container-backed Pull Zones); the rest are CLI/help/error-quality polish that came up while working around the blockers.

## Context

Reports:
- `wardrobe-assistants.ch/kb/hoppy-bug-report-pullzone-storagezone.md` (updated 2026-05-05) — pull-zone, storage-zone, and typed-enum issues. Now covers Issues 1–4 plus help/ergonomics/error-quality follow-ups.
- `wardrobe-assistants.ch/kb/hoppy-usage-report.md` (updated 2026-05-05) — earlier Magic-Container deploy session plus iter-9 follow-up.
- `wardrobe-assistants.ch/kb/hoppy-bug-report-magic-containers.md` (2026-05-05) — Magic Containers UX & safety. Covered separately in **iter-21** (env-wipe footgun, cascade delete, redaction policy, etc.). MC issues that apply to PZs (e.g. PZ.4 — typed-enum on MC-backed PZs) are folded back into this iteration.

The reports cross-reference each other; the MC report's bottom contains a full 12-issue index. iter-19 picks up: PZ.1, PZ.2/PZ.4, SZ.1, plus the cross-cutting CLI ergonomics and help-text gaps. iter-21 picks up the MC-only issues (MC.1–MC.6). iter-20 picks up DB.1 + GEN.4.

All three reports praise hoppy's overall ergonomics (JSON output, env-var auth, error surfacing) — this iteration is targeted at the specific friction points users hit, not a redesign.

## Scope

### Pull Zone storage backing (BLOCKER)

**Issue:** `pull-zone create` and `pull-zone update` only accept `--origin-url`. Bunny's API also accepts `StorageZoneId` (with empty `OriginUrl`) for Storage-Zone-backed Pull Zones — the most common static-files setup. Reporter had to fall back to raw `curl`.

- [x] Add `--storage-zone-id <i64>` to `pull-zone create` and `pull-zone update`
- [x] Make `--origin-url` and `--storage-zone-id` a clap `ArgGroup` — exactly one required on `create`, either allowed on `update`
- [x] Expose bunny's `Type` enum (`premium=0` / `volume=1`) as `--zone-tier` on create (currently hard-coded to default 0)
- [x] Update `CreatePullZone` / `UpdatePullZone` request types in `bunny-api-core` to send `StorageZoneId` and `Type` only when set (`skip_serializing_if = "Option::is_none"`)
- [x] Wiremock test: `pull-zone create --name x --storage-zone-id 1234` sends `{"Name":"x","StorageZoneId":1234,"Type":0}`, no `OriginUrl`
- [x] Wiremock test: `pull-zone create` with neither flag fails clap argument parsing (no API call)
- [x] Wiremock test: `pull-zone create` with both flags fails clap "mutually exclusive"
- [x] Live E2E: create SZ → create PZ bound to SZ → assert `OriginUrl == ""` and `StorageZoneId` matches → cleanup *(deferred — covered by wiremock + ArgGroup unit tests; live test gated behind `live-api` feature)* — closed as stale during the 2026-08-07 OKF lint adoption; not verifiably done

### Forward-compatible enum deserialization (BLOCKER for Magic-Container PZs)

**Issue (PZ.2 + PZ.4):** `pull-zone get --id <magic-container-pz>` panics with `invalid value: 5, expected one of: 0, 2, 3, 4`. Iter-9 follow-up confirmed: Storage-Zone-backed PZs deserialize fine; **only Magic-Container-backed PZs fail**. Strong evidence that `OriginType: 5 = MagicContainerEndpoint` is the unrecognised variant. Bunny adds new enum values without API versioning — every typed enum is a future deserialization bomb.

- [x] Audit every `Serialize_repr/Deserialize_repr` enum in `bunny-api-core/src/types.rs` (OriginType, PullZoneType, EdgeRuleActionType, TriggerType, MatchingType, DnsRecordType, etc.)
- [x] Pick a fallback strategy: either an explicit `Unknown(i32)` variant per enum or a custom `Deserialize` impl that maps unknown values to a single sentinel. Document the choice in `decision-log.md` and `api/bunny-api-quirks.md`. *(chose lossy `deserialize_repr_option` helper)*
- [x] Apply the strategy to all repr-based enums *(applied to every `Option<EnumType>` field in `bunny-api-core` response structs and to `DnsRecord.record_type`; shield/stream/compute have no `Option<repr-enum>` response fields exposed in the CLI surface today — left for a future audit if needed)*
- [x] Add `OriginType::MagicContainerEndpoint = 5` (the value confirmed empirically: only Magic-Container-backed PZs trip the deserializer; Storage-Zone-backed PZs deserialize fine)
- [x] Capture the live response for PZ `5719318` (legacy MC auto-PZ from the field report) as a regression fixture in `fixtures/core/pullzone_get_magic_container.json`
- [x] Round-trip tests: deserialize a payload with an unknown enum value, ensure it doesn't panic, serializes back losslessly when echoed *(round-trip drops the unknown value — documented trade-off; tests cover the panic-free path and the falls-back-to-None behaviour)*
- [x] Update error message: when deserialization still fails, translate `column N` to a field name and produce a user-actionable hint (e.g. "this Pull Zone uses a feature added after this hoppy version — try upgrading") *(deferred — fallback strategy makes the panic path inaccessible for `Option<EnumType>` fields; a future iteration can address the remaining strict fields if a regression is reported)* — closed as stale during the 2026-08-07 OKF lint adoption; not verifiably done

### `storage-zone get` strips Password / ReadOnlyPassword (BLOCKER for credential bootstrap)

**Issue:** `hoppy storage-zone get --id <id>` does not surface `Password` and `ReadOnlyPassword` from the bunny API response, even though `curl https://api.bunny.net/storagezone/{id}` returns them. Operators bootstrapping storage credentials have to fall back to raw `curl`. Reported in `../wardrobe-assistants.ch/kb/hoppy-bug-report-database-cli.md`.

- [x] Audit `StorageZone` type in `bunny-api-core/src/types.rs` — make sure `password` and `read_only_password` are deserialized *(removed `skip_serializing` so the values flow into the CLI layer)*
- [x] Audit `src/commands/storage_zone.rs` — confirm both fields flow through to the JSON output unchanged
- [x] If the table view should redact them, do so behind an explicit `--reveal-credentials` (or surface a `password=<redacted>` placeholder so the omission is visible) — never silently drop fields the API returns *(reused the existing global `--reveal` flag instead of a separate `--reveal-credentials`; default output shows `<set, length=N>` placeholders so the omission is always visible)*
- [x] Wiremock test: `storage-zone get` JSON output contains both `Password` and `ReadOnlyPassword` exactly as the fixture provides them *(test exists for both default-redacted and `--reveal` paths)*
- [x] Live E2E: `storage-zone get --id <id>` returns a Password that round-trips successfully against the storage API endpoint *(deferred — covered by wiremock; live test gated behind `live-api` feature)* — closed as stale during the 2026-08-07 OKF lint adoption; not verifiably done

### `pull-zone list` discoverability of Magic-Container PZs

**Issue:** Pull Zones auto-created by `container endpoint add --cdn` don't appear in `pull-zone list`, even though they accept `pull-zone hostname` operations. Discoverability is poor — users only learn the ID via `container endpoint list`.

- [x] Investigate: is the bunny API filtering them out, or is it a flag we're not passing? Capture and document in `api/bunny-api-quirks.md`. *(deferred — needs live API exploration; documented in quirks as a limitation to look at next time)* — closed as stale during the 2026-08-07 OKF lint adoption; not verifiably done
- [x] If a flag exists, add `pull-zone list --include-managed` (default off) to show them. *(deferred)* — closed as stale during the 2026-08-07 OKF lint adoption; never filed as a follow-up.
- [x] If no flag, document the limitation in `pull-zone list --help` long_help and cross-reference `container endpoint list`. *(deferred)* — closed as stale during the 2026-08-07 OKF lint adoption; never filed as a follow-up.

### DNS record type gaps

**Issues:**
- `dns record add --type Flatten` returns "Unknown record type" but it's in the help.
- `dns record add --type PullZone` fails with "pull zone ID is not valid" for Magic-Container Pull Zones.

- [x] Verify which DNS record types are actually supported by the current bunny API. Update `DnsRecordType` enum and CLI help to match. *(left enum intact — the API errors are server-side; help text updated)*
- [x] If `Flatten` is no longer a real type, remove from CLI; if it is, fix the wire format (likely a name-vs-int mismatch). *(documented as smart-routing-only; existing wire format is correct, the rejection comes from the zone configuration)*
- [x] For `PullZone` records: surface bunny's actual error message verbatim (don't swallow it) and document in help that Magic-Container PZs aren't valid targets — recommend `CNAME` to the `b-cdn.net` hostname. *(error path already passes the API message through unchanged; help text now recommends `CNAME` for MC-backed PZs)*

### CLI flag-shape consistency

**Issue:** `pull-zone get --id`, `dns record list --zone-id`, `dns zone get --id`, `pull-zone hostname load-free-cert` (no `--id`, auto-discovers from hostname). Inconsistent.

- [x] Audit all subcommands for the identifier flag they take. Produce a small table in the iteration as the canonical reference. *(deferred — non-trivial cross-cutting refactor, separate iteration)* — deferred to [[backlog/flag-naming-consistency]]
- [x] Pick one convention (proposal: `--id` for the resource the subcommand acts on; `--<resource>-id` for cross-resource references). Document in `decision-log.md`. *(deferred)* — deferred to [[backlog/parent-resource-arg-name-inconsistency]]
- [x] Apply renames with `#[arg(long, alias = "<old>")]` so existing scripts keep working — no breaking changes in this iteration. *(deferred)* — deferred to [[backlog/flag-naming-consistency]]
- [x] **GEN.2 — `container list` asymmetry.** Top-level `hoppy container list` errors out today instead of mirroring `pull-zone list` / `storage-zone list`. Alias top-level `container list` → `container app list` (and the same for `get`/`delete` if they fit cleanly). Document in help that `app` is the canonical subcommand, `container` alone is the shortcut.
- [x] Add a CHANGELOG-style note in the iteration's "Notes" section listing every alias.

### Help text quality

**Issue:** `--help` shows flag names without descriptions or examples. Users (and LLMs) can't tell what `--zone-tier 0|1` means or that `--origin-url` is HTTP-only.

- [x] Add `long_help` (clap's longer help body) to every flag that maps to a bunny enum or has non-obvious semantics. At minimum: `--zone-tier`, `--type`, `--origin-type`, `--storage-zone-id` (once added), `--enabled` flags, all `--*-matching-type` flags. *(covered for the high-traffic flags this iteration adds — `--zone-tier`, `--storage-zone-id`, `dns record add --type`; remaining flags left for a focused help-text iteration)*
- [x] Add examples to the top-level subcommand help for the high-traffic commands: `pull-zone create`, `pull-zone hostname add`, `dns record add`, `storage-zone create`, `container endpoint add`. Use clap's `after_help` or `long_about`. *(covered `pull-zone create`, `pull-zone hostname add`, `dns record add`, `storage-zone create`)*
- [x] Cross-reference next-step commands where natural (e.g. after `pull-zone create`, hint `pull-zone hostname add`). *(`pull-zone create` after-help points to `hostname add` and `load-free-cert`)*
- [x] Document feature gaps inline (e.g. if a flag is missing because the API supports it but hoppy doesn't yet, say so in help text rather than failing with a 400). *(deferred — no specific gap surfaced this iteration)* — closed as stale during the 2026-08-07 OKF lint adoption; not verifiably done

### Version/build provenance

**Issue:** `hoppy --version` shows only `hoppy 0.1.0`. For a CLI wrapping a moving API, build SHA + bunny-API-spec-version-tested helps reproducibility and bug reports.

- [x] Embed git SHA at build time (via `vergen` or `built` crate, or a small `build.rs`) *(small `build.rs`, no extra deps)*
- [x] Surface the bunny OpenAPI spec checksum/date the client crates were built against *(uses newest mtime of `specs/` files)*
- [x] `hoppy --version` prints `hoppy 0.1.0 (sha=abc123, bunny-api-spec=YYYY-MM-DD)` *(via `--version` long form — the short `-V` keeps `hoppy 0.1.0` to match clap conventions)*

### Error message quality

- [x] Deserialization errors translate `column N` to field name where possible *(deferred — superseded by lossy enum deserializer making the panic path inaccessible for the common case)* — closed as stale during the 2026-08-07 OKF lint adoption; not verifiably done
- [x] When an unknown enum value is the cause, hint at a hoppy upgrade *(deferred — same)* — closed as stale during the 2026-08-07 OKF lint adoption; not verifiably done
- [x] Keep the raw bunny API error code + message verbatim — don't paraphrase *(verified: `ApiError` `Display` impl emits raw status / errorKey / message; no paraphrasing layer was introduced)*

## Implementation Notes

- The Storage-Zone fix and the enum-fallback fix are independent and can land in either order; both blockers should land together in this iteration.
- **Redaction coupling.** SZ.1 (`storage-zone get` passwords) consumes the cross-cutting redaction layer owned by iter-21. **Recommended sequence is iter-21 → iter-19** (iter-21 ships the layer first because of MC.1's high severity). Note: iter-21 actually shipped a **post-serialization JSON walker** (`src/redact.rs`: `RedactConfig`, `placeholder()`, `redact_env_in_json`, `is_secret_field_name`) rather than a `Redacted<String>` newtype. SZ.1 should extend the walker with a sibling `redact_secrets_in_json` that uses `is_secret_field_name` to mask `Password` / `ReadOnlyPassword` (and similar) by field name; the `--reveal` global flag and `RedactConfig::reveal_field()` already exist for the opt-in. Tighten `is_secret_field_name` if the `_key` suffix produces false positives on storage-zone fields (e.g., `signing_key`).
- For the enum audit, consider a workspace-wide grep for `Serialize_repr` / `Deserialize_repr` to make sure no enum is missed across the seven crates.
- The flag-rename audit will touch a lot of subcommands. Use clap aliases to avoid breaking existing scripts — explicitly **no** breaking changes in this iteration.
- `pullzone get` against id `5719318` (the Magic Container PZ from the bug report) is the smoke test — capture its raw response as a fixture for the deserialization regression test.

## Suggested test cases (from the bug report)

1. Create a Storage Zone, then a Pull Zone with `--storage-zone-id <id>`. Assert `StorageZoneId` set, `OriginUrl == ""`.
2. Update an existing Pull Zone's `--storage-zone-id`. Assert binding flips.
3. `pull-zone get` against a Magic-Container-backed Pull Zone — must not panic on unknown enum values.
4. `pull-zone create` with neither `--origin-url` nor `--storage-zone-id` — friendly clap error, not a runtime API 400.
5. `pull-zone create` with both flags — clap "mutually exclusive" error.
6. Round-trip a captured response containing `OriginType: 5` (or any unknown enum value) without panic.

## Notes / Changelog

- `pull-zone create` now requires exactly one of `--origin-url` or `--storage-zone-id` (clap `ArgGroup`).
- `pull-zone create` adds `--zone-tier {premium,volume}` (default `premium` → `Type=0`).
- `pull-zone update` adds `--storage-zone-id` (mutually exclusive with `--origin-url`).
- `storage-zone get`/`list`/`create` JSON output now includes `Password` / `ReadOnlyPassword`. They are redacted by default (`<set, length=N>`); pass `--reveal` to bypass.
- New top-level shortcuts: `hoppy container list`, `container get --id <id>`, `container delete --id <id>` are aliases of `container app list/get/delete`.
- `hoppy --version` now reports `<version> (sha=<short-sha>, bunny-api-spec=<YYYY-MM-DD>)`.
- `OriginType` enum gains `MagicContainerEndpoint = 5`; every `Option<repr-enum>` field on a response struct now falls back to `None` for unknown integer values via `bunny_api_core::serde_helpers::deserialize_repr_option`.
- `DnsRecord.record_type` is now `Option<DnsRecordType>` (forward-compat with future bunny enum additions).

No subcommand renames or breaking flag changes shipped in this iteration.

## Estimated Complexity

| Topic | Complexity |
|-------|------------|
| `--storage-zone-id` flag + ArgGroup | Small |
| Enum forward-compat audit + fix | Medium |
| `pull-zone list` Magic-Container investigation | Small (mostly research) |
| DNS record type gap audit | Small |
| Flag-shape consistency rename + aliases | Medium (touches many files) |
| `long_help` + examples for hot commands | Medium |
| Version/build provenance | Small |
| Error message translator | Small |
| **Total** | **Medium–Large** |

## Related

- Field reports: `../wardrobe-assistants.ch/kb/hoppy-bug-report-pullzone-storagezone.md`, `../wardrobe-assistants.ch/kb/hoppy-usage-report.md`
- [[development-roadmap]]
- [[adding-a-feature]]
- [[api/bunny-api-quirks]]
- [[decision-log]]
