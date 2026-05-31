#!/usr/bin/env sh
# run-spec-coverage-audit.sh — Orchestrates audit-spec-coverage.sh across
# every (resource, struct, schema) tuple hoppy currently models, and
# writes one report per resource under
# hoppy-knowledgebase/research/spec-coverage/.
#
# Re-runnable: overwrites the per-resource reports and the README index.

set -eu

cd "$(dirname "$0")/../.."   # repo root

SCRIPT="hoppy-knowledgebase/scripts/audit-spec-coverage.sh"
OUT_DIR="hoppy-knowledgebase/research/spec-coverage"
DATA_DIR="$OUT_DIR/.data"
mkdir -p "$OUT_DIR" "$DATA_DIR"
# Wipe sidecars from any prior run so dedup unions are clean.
rm -f "$DATA_DIR"/*.spec.txt "$DATA_DIR"/*.struct.txt "$DATA_DIR"/*.missing.txt 2>/dev/null || true

# Each line: resource-slug | title | spec | schema | struct-file | struct
TUPLES=$(cat <<'EOF'
pull-zone|Pull zone — read shape|specs/core-platform.json|PullZoneModel|crates/bunny-net-api/src/core/types.rs|PullZone
pull-zone|Pull zone — update payload|specs/core-platform.json|PullZoneSettingsModel|crates/bunny-net-api/src/core/types.rs|UpdatePullZone
pull-zone|Pull zone — create payload|specs/core-platform.json|PullZoneAddModel|crates/bunny-net-api/src/core/types.rs|CreatePullZone
storage-zone|Storage zone — read shape|specs/core-platform.json|StorageZoneModel|crates/bunny-net-api/src/core/types.rs|StorageZone
storage-zone|Storage zone — update payload|specs/core-platform.json|StorageZoneSettingsModel|crates/bunny-net-api/src/core/types.rs|UpdateStorageZone
storage-zone|Storage zone — create payload|specs/core-platform.json|StorageZoneModelAdd|crates/bunny-net-api/src/core/types.rs|CreateStorageZone
dns-zone|DNS zone — read shape|specs/core-platform.json|DnsZoneModel|crates/bunny-net-api/src/core/types.rs|DnsZone
dns-zone|DNS zone — update payload|specs/core-platform.json|UpdateDnsZoneModel|crates/bunny-net-api/src/core/types.rs|UpdateDnsZone
dns-zone|DNS zone — create payload|specs/core-platform.json|DnsZoneAddModel|crates/bunny-net-api/src/core/types.rs|CreateDnsZone
video-library|Video library — read shape|specs/core-platform.json|VideoLibraryModel|crates/bunny-net-api/src/core/types.rs|VideoLibrary
video-library|Video library — update payload|specs/core-platform.json|VideoLibraryUpdateModel|crates/bunny-net-api/src/core/types.rs|UpdateVideoLibrary
video-library|Video library — create payload|specs/core-platform.json|VideoLibraryCreateModel|crates/bunny-net-api/src/core/types.rs|CreateVideoLibrary
edge-script|Edge script — read shape|specs/edge-scripting.json|EdgeScriptModel|crates/bunny-net-api/src/compute/types.rs|EdgeScript
edge-script|Edge script — update payload|specs/edge-scripting.json|UpdateEdgeScriptModel|crates/bunny-net-api/src/compute/types.rs|UpdateEdgeScript
edge-script|Edge script — create payload|specs/edge-scripting.json|AddEdgeScriptModel|crates/bunny-net-api/src/compute/types.rs|CreateEdgeScript
stream-video|Stream video — read shape|specs/stream.json|VideoModel|crates/bunny-net-api/src/stream/types.rs|Video
stream-video|Stream video — update payload|specs/stream.json|UpdateVideoModel|crates/bunny-net-api/src/stream/types.rs|UpdateVideo
stream-video|Stream video — create payload|specs/stream.json|CreateVideoModel|crates/bunny-net-api/src/stream/types.rs|CreateVideo
shield-zone|Shield zone — read shape|specs/shield.json|ShieldZoneResponse|crates/bunny-net-api/src/shield/types.rs|ShieldZoneResponse
shield-zone|Shield zone — update payload|specs/shield.json|UpdateShieldZoneRequest|crates/bunny-net-api/src/shield/types.rs|UpdateShieldZoneRequest
shield-zone|Shield zone — create payload|specs/shield.json|CreateShieldZoneRequest|crates/bunny-net-api/src/shield/types.rs|CreateShieldZoneRequest
database|Database (V2) — read shape|specs/database.json|Database|crates/bunny-net-api/src/database/types.rs|Database
database|Database (V2) — create payload|specs/database.json|CreateDatabaseV2Payload|crates/bunny-net-api/src/database/types.rs|CreateDatabasePayload
database|Database (V2) — update payload|specs/database.json|UpdateDatabaseV2Payload|crates/bunny-net-api/src/database/types.rs|UpdateDatabaseGroupPayload
storage-object|Storage object (data plane)|specs/storage.json|StorageObject|crates/bunny-net-api/src/storage/types.rs|StorageObject
EOF
)

# Group tuples by resource slug and concatenate reports.
SLUGS=$(echo "$TUPLES" | awk -F'|' '{print $1}' | awk '!seen[$0]++')

run_one() {
    slug=$1; title=$2; spec=$3; schema=$4; struct_file=$5; struct_name=$6
    slug_data="$DATA_DIR/$slug"
    mkdir -p "$slug_data"
    if "$SCRIPT" --spec "$spec" --schema "$schema" \
                 --struct-file "$struct_file" --struct-name "$struct_name" \
                 --title "$title" --data-dir "$slug_data" 2>/dev/null; then
        return 0
    fi
    # Fallback report when the schema or struct can't be located.
    cat <<MD
## $title

- **Spec file:** \`$spec\`
- **Schema:** \`$schema\`
- **Struct file:** \`$struct_file\`
- **Struct:** \`$struct_name\`

> **TODO:** could not extract — either the schema is absent from the
> spec, the struct is absent from the source file, or the schema uses
> a composition pattern the audit script does not yet support
> (\`oneOf\` / \`discriminator\`). Investigate manually.

MD
}

for slug in $SLUGS; do
    out="$OUT_DIR/$slug.md"
    {
        cat <<EOF
---
title: "Spec coverage audit — $slug"
type: research
date: 2026-05-31
tags:
  - audit
  - openapi
  - spec-coverage
---

# Spec coverage audit — \`$slug\`

Generated by \`hoppy-knowledgebase/scripts/run-spec-coverage-audit.sh\`
(which calls \`audit-spec-coverage.sh\` per tuple). **This file is
overwritten on every re-audit run** — hand-authored commentary
(severity buckets, planning recommendations) belongs in a sibling
\`<slug>-buckets.md\` (or similar) file, not here.

EOF
        echo "$TUPLES" | awk -F'|' -v want="$slug" '$1 == want' | while IFS='|' read -r _slug title spec schema struct_file struct_name; do
            run_one "$slug" "$title" "$spec" "$schema" "$struct_file" "$struct_name"
            echo
        done
    } > "$out"
    echo "wrote $out" >&2
done

# Container app: no spec is present in specs/ — write a stub.
cat > "$OUT_DIR/container-app.md" <<'EOF'
---
title: "Spec coverage audit — container-app"
type: research
date: 2026-05-31
tags:
  - audit
  - openapi
  - spec-coverage
  - todo
---

# Spec coverage audit — `container-app`

> **TODO — no spec file available.** The bunny.net Magic Containers
> (Container App) API is not currently checked into `specs/`. The
> hand-written `containers/types.rs` module models ~15 request/response
> shapes (`ContainerInstance`, `ContainerTemplate`, `ContainerRequest`,
> `ContainerEndpoint`, …) without a reference document to diff against.
>
> Follow-up actions:
>
> - Locate / export the Magic Containers OpenAPI spec from bunny.net.
> - Drop it under `specs/containers.json`.
> - Re-run `run-spec-coverage-audit.sh` after extending the tuple list
>   in the orchestrator to include each `Container*` struct.
EOF
echo "wrote $OUT_DIR/container-app.md" >&2

# --- Deduplicated index ------------------------------------------------------
# Per-resource unique counts: union spec/struct/missing across tuples for the
# same resource slug. This is the load-bearing fix for the iter-43 finding
# that the index over-counted by ~3x because the same missing toggle appears
# in both `*Model` (read) and `*SettingsModel` (update) schemas.
INDEX="$OUT_DIR/README.md"
{
    cat <<'EOF'
---
title: "Spec coverage audit — index"
type: research
date: 2026-05-31
tags:
  - audit
  - openapi
  - spec-coverage
  - index
---

# Spec coverage audit — index

Per-resource property and gap counts, **deduplicated** across the
(read shape, update payload, create payload) tuples we model for each
resource. A field missing from both `<Resource>Model` and
`<Resource>SettingsModel` is counted **once** here — the per-resource
report still lists every tuple's gap individually.

Generated by `hoppy-knowledgebase/scripts/run-spec-coverage-audit.sh`.

| Resource | Unique spec fields | Unique struct fields | Unique gap |
|---|---:|---:|---:|
EOF

    total_spec=0
    total_struct=0
    total_missing=0
    for slug in $SLUGS; do
        slug_dir="$DATA_DIR/$slug"
        if [ ! -d "$slug_dir" ]; then continue; fi
        spec_u=0; struct_u=0; missing_u=0
        if ls "$slug_dir"/*.spec.txt    >/dev/null 2>&1; then
            spec_u=$(cat "$slug_dir"/*.spec.txt    | sort -u | wc -l | tr -d ' ')
        fi
        if ls "$slug_dir"/*.struct.txt  >/dev/null 2>&1; then
            struct_u=$(cat "$slug_dir"/*.struct.txt  | sort -u | wc -l | tr -d ' ')
        fi
        if ls "$slug_dir"/*.missing.txt >/dev/null 2>&1; then
            missing_u=$(cat "$slug_dir"/*.missing.txt | sort -u | wc -l | tr -d ' ')
        fi
        printf "| [%s](%s.md) | %d | %d | **%d** |\n" "$slug" "$slug" "$spec_u" "$struct_u" "$missing_u"
        total_spec=$((total_spec + spec_u))
        total_struct=$((total_struct + struct_u))
        total_missing=$((total_missing + missing_u))
    done

    cat <<EOF

## Totals (deduplicated)

- **Unique spec properties:** $total_spec
- **Unique struct fields:** $total_struct
- **Unique missing fields (the real systemic gap):** **$total_missing**

## Caveats

- Counts are unique *within a resource* (read+update+create unioned).
- The audit script PascalCases Rust field names; APIs that use
  camelCase (Shield, Stream V2) may surface real coverage as
  "casing mismatches" in the per-resource files — investigate
  those before treating them as gaps.
- Schemas with \`oneOf\` / \`discriminator\` composition are not
  flattened by the script and surface as a TODO in the per-resource
  file.
- \`container-app\` has no spec checked in under \`specs/\` — see
  that resource file for follow-up.

See [[../../iterations/iteration-43-openapi-gap-analysis]] for the
motivating audit and [[pull-zone-buckets]] for the severity-bucketed
breakdown that informs sequencing.
EOF
} > "$INDEX"
echo "wrote $INDEX" >&2
