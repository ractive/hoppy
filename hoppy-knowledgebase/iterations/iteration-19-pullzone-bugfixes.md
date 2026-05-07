---
title: "Iteration 19 — Pull Zone bug fixes & CLI UX polish"
type: iteration
date: 2026-05-05
tags:
  - iteration
  - bugfix
  - cdn
  - ux
  - dx
status: planned
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

- [ ] Add `--storage-zone-id <i64>` to `pull-zone create` and `pull-zone update`
- [ ] Make `--origin-url` and `--storage-zone-id` a clap `ArgGroup` — exactly one required on `create`, either allowed on `update`
- [ ] Expose bunny's `Type` enum (`premium=0` / `volume=1`) as `--zone-tier` on create (currently hard-coded to default 0)
- [ ] Update `CreatePullZone` / `UpdatePullZone` request types in `bunny-api-core` to send `StorageZoneId` and `Type` only when set (`skip_serializing_if = "Option::is_none"`)
- [ ] Wiremock test: `pull-zone create --name x --storage-zone-id 1234` sends `{"Name":"x","StorageZoneId":1234,"Type":0}`, no `OriginUrl`
- [ ] Wiremock test: `pull-zone create` with neither flag fails clap argument parsing (no API call)
- [ ] Wiremock test: `pull-zone create` with both flags fails clap "mutually exclusive"
- [ ] Live E2E: create SZ → create PZ bound to SZ → assert `OriginUrl == ""` and `StorageZoneId` matches → cleanup

### Forward-compatible enum deserialization (BLOCKER for Magic-Container PZs)

**Issue (PZ.2 + PZ.4):** `pull-zone get --id <magic-container-pz>` panics with `invalid value: 5, expected one of: 0, 2, 3, 4`. Iter-9 follow-up confirmed: Storage-Zone-backed PZs deserialize fine; **only Magic-Container-backed PZs fail**. Strong evidence that `OriginType: 5 = MagicContainerEndpoint` is the unrecognised variant. Bunny adds new enum values without API versioning — every typed enum is a future deserialization bomb.

- [ ] Audit every `Serialize_repr/Deserialize_repr` enum in `bunny-api-core/src/types.rs` (OriginType, PullZoneType, EdgeRuleActionType, TriggerType, MatchingType, DnsRecordType, etc.)
- [ ] Pick a fallback strategy: either an explicit `Unknown(i32)` variant per enum or a custom `Deserialize` impl that maps unknown values to a single sentinel. Document the choice in `decision-log.md` and `api/bunny-api-quirks.md`.
- [ ] Apply the strategy to all repr-based enums
- [ ] Add `OriginType::MagicContainerEndpoint = 5` (the value confirmed empirically: only Magic-Container-backed PZs trip the deserializer; Storage-Zone-backed PZs deserialize fine)
- [ ] Capture the live response for PZ `5719318` (legacy MC auto-PZ from the field report) as a regression fixture in `fixtures/core/pullzone_get_magic_container.json`
- [ ] Round-trip tests: deserialize a payload with an unknown enum value, ensure it doesn't panic, serializes back losslessly when echoed
- [ ] Update error message: when deserialization still fails, translate `column N` to a field name and produce a user-actionable hint (e.g. "this Pull Zone uses a feature added after this hoppy version — try upgrading")

### `storage-zone get` strips Password / ReadOnlyPassword (BLOCKER for credential bootstrap)

**Issue:** `hoppy storage-zone get --id <id>` does not surface `Password` and `ReadOnlyPassword` from the bunny API response, even though `curl https://api.bunny.net/storagezone/{id}` returns them. Operators bootstrapping storage credentials have to fall back to raw `curl`. Reported in `../wardrobe-assistants.ch/kb/hoppy-bug-report-database-cli.md`.

- [ ] Audit `StorageZone` type in `bunny-api-core/src/types.rs` — make sure `password` and `read_only_password` are deserialized
- [ ] Audit `src/commands/storage_zone.rs` — confirm both fields flow through to the JSON output unchanged
- [ ] If the table view should redact them, do so behind an explicit `--reveal-credentials` (or surface a `password=<redacted>` placeholder so the omission is visible) — never silently drop fields the API returns
- [ ] Wiremock test: `storage-zone get` JSON output contains both `Password` and `ReadOnlyPassword` exactly as the fixture provides them
- [ ] Live E2E: `storage-zone get --id <id>` returns a Password that round-trips successfully against the storage API endpoint

### `pull-zone list` discoverability of Magic-Container PZs

**Issue:** Pull Zones auto-created by `container endpoint add --cdn` don't appear in `pull-zone list`, even though they accept `pull-zone hostname` operations. Discoverability is poor — users only learn the ID via `container endpoint list`.

- [ ] Investigate: is the bunny API filtering them out, or is it a flag we're not passing? Capture and document in `api/bunny-api-quirks.md`.
- [ ] If a flag exists, add `pull-zone list --include-managed` (default off) to show them.
- [ ] If no flag, document the limitation in `pull-zone list --help` long_help and cross-reference `container endpoint list`.

### DNS record type gaps

**Issues:**
- `dns record add --type Flatten` returns "Unknown record type" but it's in the help.
- `dns record add --type PullZone` fails with "pull zone ID is not valid" for Magic-Container Pull Zones.

- [ ] Verify which DNS record types are actually supported by the current bunny API. Update `DnsRecordType` enum and CLI help to match.
- [ ] If `Flatten` is no longer a real type, remove from CLI; if it is, fix the wire format (likely a name-vs-int mismatch).
- [ ] For `PullZone` records: surface bunny's actual error message verbatim (don't swallow it) and document in help that Magic-Container PZs aren't valid targets — recommend `CNAME` to the `b-cdn.net` hostname.

### CLI flag-shape consistency

**Issue:** `pull-zone get --id`, `dns record list --zone-id`, `dns zone get --id`, `pull-zone hostname load-free-cert` (no `--id`, auto-discovers from hostname). Inconsistent.

- [ ] Audit all subcommands for the identifier flag they take. Produce a small table in the iteration as the canonical reference.
- [ ] Pick one convention (proposal: `--id` for the resource the subcommand acts on; `--<resource>-id` for cross-resource references). Document in `decision-log.md`.
- [ ] Apply renames with `#[arg(long, alias = "<old>")]` so existing scripts keep working — no breaking changes in this iteration.
- [ ] **GEN.2 — `container list` asymmetry.** Top-level `hoppy container list` errors out today instead of mirroring `pull-zone list` / `storage-zone list`. Alias top-level `container list` → `container app list` (and the same for `get`/`delete` if they fit cleanly). Document in help that `app` is the canonical subcommand, `container` alone is the shortcut.
- [ ] Add a CHANGELOG-style note in the iteration's "Notes" section listing every alias.

### Help text quality

**Issue:** `--help` shows flag names without descriptions or examples. Users (and LLMs) can't tell what `--zone-tier 0|1` means or that `--origin-url` is HTTP-only.

- [ ] Add `long_help` (clap's longer help body) to every flag that maps to a bunny enum or has non-obvious semantics. At minimum: `--zone-tier`, `--type`, `--origin-type`, `--storage-zone-id` (once added), `--enabled` flags, all `--*-matching-type` flags.
- [ ] Add examples to the top-level subcommand help for the high-traffic commands: `pull-zone create`, `pull-zone hostname add`, `dns record add`, `storage-zone create`, `container endpoint add`. Use clap's `after_help` or `long_about`.
- [ ] Cross-reference next-step commands where natural (e.g. after `pull-zone create`, hint `pull-zone hostname add`).
- [ ] Document feature gaps inline (e.g. if a flag is missing because the API supports it but hoppy doesn't yet, say so in help text rather than failing with a 400).

### Version/build provenance

**Issue:** `hoppy --version` shows only `hoppy 0.1.0`. For a CLI wrapping a moving API, build SHA + bunny-API-spec-version-tested helps reproducibility and bug reports.

- [ ] Embed git SHA at build time (via `vergen` or `built` crate, or a small `build.rs`)
- [ ] Surface the bunny OpenAPI spec checksum/date the client crates were built against
- [ ] `hoppy --version` prints `hoppy 0.1.0 (sha=abc123, bunny-api-spec=YYYY-MM-DD)`

### Error message quality

- [ ] Deserialization errors translate `column N` to field name where possible
- [ ] When an unknown enum value is the cause, hint at a hoppy upgrade
- [ ] Keep the raw bunny API error code + message verbatim — don't paraphrase

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
