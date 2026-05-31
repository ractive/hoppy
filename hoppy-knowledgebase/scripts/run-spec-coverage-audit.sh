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
mkdir -p "$OUT_DIR"

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
    title=$1; spec=$2; schema=$3; struct_file=$4; struct_name=$5
    if "$SCRIPT" --spec "$spec" --schema "$schema" \
                 --struct-file "$struct_file" --struct-name "$struct_name" \
                 --title "$title" 2>/dev/null; then
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
(which calls \`audit-spec-coverage.sh\` per tuple).

EOF
        echo "$TUPLES" | awk -F'|' -v want="$slug" '$1 == want' | while IFS='|' read -r _slug title spec schema struct_file struct_name; do
            run_one "$title" "$spec" "$schema" "$struct_file" "$struct_name"
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
