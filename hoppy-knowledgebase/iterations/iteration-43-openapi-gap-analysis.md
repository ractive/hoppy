---
title: Iter-43 — OpenAPI gap analysis (read-only audit, no code changes)
type: iteration
date: 2026-05-31
tags:
  - iteration
  - audit
  - research
  - openapi
  - no-code
status: planned
branch: iter-43/openapi-gap-analysis
---

# Iter-43 — OpenAPI gap analysis

## Why

The 2026-05-31 dogfooding round surfaced that `pull-zone update`
exposes only 12 of the ~45 toggle fields the API supports — and that
the same gap exists in the deserialization struct (`PullZone` has
56 of the 164 properties defined in `PullZoneModel`). Spot-checks
suggest this is **not** unique to pull-zone: the original "hand-written
clients" decision (iter-0.5, see [[../decision-log]]) preserved hoppy's
control over UX, but the audit step that should have kept the structs
in sync with the spec was never institutionalised. Every resource is
probably under-covered, and we don't know by how much.

This iteration produces the data needed to size the problem. **No code
changes**, **no struct edits**, **no CLI additions**. Just a structured
report per resource so future iterations can land the fixes in a
defensible order.

## Scope

### 1. Build a reusable audit script
Output: `hoppy-knowledgebase/scripts/audit-spec-coverage.sh`

A shell script (jq + comm, no Rust) that takes a spec file, a schema
name, and a Rust struct location, and emits a markdown report
fragment with:

- Total properties in the spec schema.
- Total fields in the Rust struct.
- Fields in the spec but missing from the struct (the gap).
- Optional: fields in the struct but missing from the spec (likely
  stale).

Recipe (extracted from the 2026-05-31 dogfooding investigation):

```sh
# spec → property names (PascalCase per bunny's API convention)
jq -r '.components.schemas.<SchemaName>.properties | keys[]' <spec>.json | sort -u > spec.txt

# Rust struct → field names converted to PascalCase
awk '/^pub struct <StructName> {/{flag=1; next} flag && /^}/{exit} flag' <types>.rs \
  | grep -E '^\s+pub\b' | awk '{print $2}' | sed 's/:.*//' \
  | awk -F_ '{out=""; for(i=1;i<=NF;i++) out=out toupper(substr($i,1,1)) substr($i,2); print out}' \
  | sort -u > struct.txt

# diff
comm -23 spec.txt struct.txt    # in spec, missing from struct
comm -13 spec.txt struct.txt    # in struct, missing from spec
```

The script must:

- [ ] Take `--spec <path>`, `--schema <name>`, `--struct-file <path>`,
      `--struct-name <name>` flags.
- [ ] Emit a markdown table (counts + sorted gap lists) to stdout.
- [ ] Handle the bunny.net spec casing quirks (e.g.
      `EnableWebPVary` in `PullZoneModel` vs `EnableWebpVary` in
      `PullZoneSettingsModel`) — report case-insensitive matches as a
      "casing mismatch" section, not as separate gaps.
- [ ] Be invokable from CI eventually (exit code 0 even when gaps
      exist; gaps are data, not failures).
- [ ] Cross-platform-safe (POSIX shell + jq + awk — no GNU-only flags).

### 2. Run the audit on every read-shape struct + write-payload schema

Output: one markdown file per resource at
`hoppy-knowledgebase/research/spec-coverage/<resource>.md`

For each of these resource pairs, generate the report:

| Resource | Read schema | Write schema | Struct file | Read struct | Write struct |
|---|---|---|---|---|---|
| pull-zone | `PullZoneModel` | `PullZoneSettingsModel` | `crates/bunny-net-api/src/core/types.rs` | `PullZone` | `UpdatePullZone` |
| storage-zone | `StorageZoneModel`+`StorageZoneSettingsModel` | (find via spec paths) | same | `StorageZone` | `UpdateStorageZone` |
| dns zone | `DnsZoneModel` | (find via spec) | same | `DnsZone` | `UpdateDnsZone` |
| video library | `VideoLibraryModel` | (find via spec) | same | `VideoLibrary` | `UpdateVideoLibrary` |
| container app | (in `specs/...` — locate) | — | `crates/bunny-net-api/src/containers/...` | `ContainerApp` | … |
| stream video library | `Library` (in `specs/stream.json`) | — | `crates/bunny-net-api/src/stream/...` | `StreamLibrary` | … |
| edge script | (in `specs/edge-scripting.json`) | — | `crates/bunny-net-api/src/compute/...` | `Script` | … |
| storage zone | (in `specs/storage.json`) | — | … | … | … |
| shield zone | (in `specs/shield.json`) | — | `crates/bunny-net-api/src/shield/...` | … | … |
| database | (in `specs/database.json`) | — | `crates/bunny-net-api/src/database/...` | … | … |

The script should be invoked once per (resource, struct, schema) tuple.
Where the spec uses a different schema name than the struct name,
discover it by:

- `jq -r '.components.schemas | keys[]' specs/<file>.json | grep -i <resource>`
- Following `$ref` from the relevant path (`/pullzone/{id}` POST →
  request body → schema).

**Don't guess** — if a struct/schema mapping is ambiguous, write a TODO
in the per-resource report and move on. The point of this iteration is
to surface the data, not paper over uncertainty.

- [ ] Per-resource markdown report at
      `hoppy-knowledgebase/research/spec-coverage/<resource>.md`,
      following the same template:
      - Schema name + spec path + struct name + struct path.
      - Total field counts.
      - The full gap list (in spec, not in struct), with each field
        on its own line.
      - A "casing mismatches" section if any.
      - A "reverse gap" section (in struct, not in spec) if any —
        these are usually stale fields the API stopped sending.
- [ ] An index file at `hoppy-knowledgebase/research/spec-coverage/README.md`
      summarising:
      - One row per resource: spec fields / struct fields / gap count.
      - The total across all resources (the "size of the systemic
        problem" number).

### 3. Categorise pull-zone's 33 toggle gaps by severity

Output: append to `hoppy-knowledgebase/research/spec-coverage/pull-zone.md`

Use the categorisation already in
[[../backlog/pull-zone-update-toggle-coverage-gap]] (security,
caching, vary headers, origin/edge, blocking, logging) but extend it to
**all** PullZoneSettingsModel fields the struct is missing, not just
toggles. Numbers (e.g. `CacheControlMaxAgeOverride`), strings (e.g.
`OriginHostHeader`), and enums also need triage.

- [ ] Group each missing field under one of:
      - **security/compliance** — blocks PCI/SOC2 requirements.
      - **performance** — caching, optimisation, vary.
      - **routing/origin** — edge config, origin selection.
      - **firewall** — blocking, allowlists, security keys.
      - **observability** — logging, statistics.
      - **niche** — fields very few users will ever touch.
- [ ] Mark a "next iteration scope" recommendation: smallest
      surgical PR vs broader bundles.

## Out of scope

- **All code changes.** No edits to `crates/bunny-net-api/src/`. No
  edits to `crates/hoppy-cli/src/cli.rs`. No edits to handlers. This
  iteration produces a report and a script; the actual struct/CLI
  expansion happens in iter-44+ informed by the data here.
- Wider tooling like a `progenitor`-driven codegen (already evaluated
  and rejected in iter-0.5, see [[../decision-log]]). Just data this
  round.
- Tests for the audit script. It's a one-shot investigation tool, not
  shipped product code.
- Specs hoppy doesn't currently touch (e.g. anything bunny.net offers
  that hoppy doesn't model at all today).

## Acceptance

- [ ] `hoppy-knowledgebase/scripts/audit-spec-coverage.sh` exists,
      runs against `specs/core-platform.json` + `PullZoneModel` +
      `PullZone`, and reproduces the 33-gap finding from the 2026-05-31
      dogfooding round.
- [ ] `hoppy-knowledgebase/research/spec-coverage/` has one report per
      resource listed in scope §2.
- [ ] `hoppy-knowledgebase/research/spec-coverage/README.md` totals
      the gap counts across all resources — i.e. a single number for
      "how big is the systemic coverage gap."
- [ ] Pull-zone report has the severity grouping from §3.
- [ ] No `crates/` files modified. Confirm with
      `git diff origin/main..HEAD -- crates/ | wc -l` returning 0.
- [ ] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings &&
      cargo test --workspace --quiet` clean (should be trivially true
      since no code changed).
- [ ] [[../backlog/pull-zone-update-toggle-coverage-gap]] updated with
      a link to the new pull-zone report.

## Related

- [[../backlog/pull-zone-update-toggle-coverage-gap]] — the dogfood
  finding that motivated this audit.
- [[../decision-log]] — iter-0.5's hand-written-clients decision, which
  this audit is the missing companion process to.
- [[iteration-0.5-codegen-experiment]] — the codegen evaluation that
  rejected the spec-driven approach. This iteration produces the data
  needed to revisit that decision later if the gap is large enough.
- Dogfooding investigation: 2026-05-31 (post-iter-42).
