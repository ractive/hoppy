#!/usr/bin/env sh
# audit-fixture-coverage.sh — Compare a recorded e2e fixture (a real
# bunny.net API response) to a hand-written Rust struct, and (optionally)
# to a named OpenAPI schema.
#
# The spec is bunny.net's documentation of the API. The fixture is what
# the API *actually* sends today. The struct is what hoppy deserializes
# into. Three sources, three pair-wise diffs — this script reports all
# three so we can spot:
#   - fields the API sends that the spec doesn't document (spec lag),
#   - fields the spec documents that the API doesn't send back (write-only
#     or deprecated),
#   - fields hoppy's struct misses on both axes (the real gap).
#
# Usage:
#   audit-fixture-coverage.sh \
#       --fixture fixtures/core/pullzone_get.json \
#       --struct-file crates/bunny-net-api/src/core/types.rs \
#       --struct-name PullZone \
#       [--spec specs/core-platform.json --schema PullZoneModel] \
#       [--title "Pull zone fixture"] \
#       [--jq-prefix .]  # path expression into the fixture root
#
# Output: markdown report fragment on stdout. Exit code always 0.

set -eu

FIXTURE=""
STRUCT_FILE=""
STRUCT_NAME=""
SPEC=""
SCHEMA=""
TITLE=""
JQ_PREFIX="."

usage() {
    cat >&2 <<EOF
Usage: $0 --fixture <path> --struct-file <path> --struct-name <name>
          [--spec <path> --schema <name>] [--title <text>] [--jq-prefix <jq>]
EOF
    exit 2
}

while [ $# -gt 0 ]; do
    case "$1" in
        --fixture) FIXTURE="$2"; shift 2 ;;
        --struct-file) STRUCT_FILE="$2"; shift 2 ;;
        --struct-name) STRUCT_NAME="$2"; shift 2 ;;
        --spec) SPEC="$2"; shift 2 ;;
        --schema) SCHEMA="$2"; shift 2 ;;
        --title) TITLE="$2"; shift 2 ;;
        --jq-prefix) JQ_PREFIX="$2"; shift 2 ;;
        -h|--help) usage ;;
        *) echo "Unknown arg: $1" >&2; usage ;;
    esac
done

[ -n "$FIXTURE" ] || usage
[ -n "$STRUCT_FILE" ] || usage
[ -n "$STRUCT_NAME" ] || usage
[ -f "$FIXTURE" ] || { echo "Fixture not found: $FIXTURE" >&2; exit 1; }
[ -f "$STRUCT_FILE" ] || { echo "Struct file not found: $STRUCT_FILE" >&2; exit 1; }

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

FIXTURE_TXT="$TMP/fixture.txt"
STRUCT_TXT="$TMP/struct.txt"
SPEC_TXT="$TMP/spec.txt"

# --- Fixture → top-level keys ------------------------------------------------
# Some fixtures wrap the payload (e.g. paginated `{ Items: [...] }`); the
# caller can pass --jq-prefix `.Items[0]` to dive in.
jq -r "$JQ_PREFIX | if type == \"object\" then keys[] elif type == \"array\" then .[0] | keys[] else empty end" \
    "$FIXTURE" 2>/dev/null | sort -u > "$FIXTURE_TXT" || {
    echo "Failed to extract keys from $FIXTURE (prefix $JQ_PREFIX)" >&2
    exit 1
}
FIXTURE_COUNT=$(wc -l < "$FIXTURE_TXT" | tr -d ' ')

# --- Spec → property names (optional) ---------------------------------------
SPEC_COUNT="-"
if [ -n "$SPEC" ] && [ -n "$SCHEMA" ] && [ -f "$SPEC" ]; then
    jq -r --arg s "$SCHEMA" '
        .components.schemas[$s]
        | if . == null then empty
          elif has("properties") then .properties | keys[]
          elif has("allOf") then [.allOf[] | (if has("properties") then .properties | keys else [] end)] | add // [] | .[]
          else empty
          end
    ' "$SPEC" 2>/dev/null | sort -u > "$SPEC_TXT" || :
    if [ -s "$SPEC_TXT" ]; then
        SPEC_COUNT=$(wc -l < "$SPEC_TXT" | tr -d ' ')
    fi
fi

# --- Rust struct → PascalCased field names ----------------------------------
awk -v target="$STRUCT_NAME" '
    function pascalize(s,   parts, n, i, out, p) {
        n = split(s, parts, "_"); out = ""
        for (i = 1; i <= n; i++) {
            p = parts[i]; if (length(p) == 0) continue
            out = out toupper(substr(p, 1, 1)) substr(p, 2)
        }
        return out
    }
    BEGIN { inside = 0; rename = ""; skip_next = 0 }
    !inside {
        if (match($0, "^pub[ \t]+struct[ \t]+" target "([ \t<({].*)?[ \t]*\\{[ \t]*$")) { inside = 1 }
        next
    }
    inside {
        if ($0 ~ /^\}/) { inside = 0; next }
        if (match($0, /#\[serde\([^]]*\)\]/)) {
            attr = substr($0, RSTART, RLENGTH)
            # Mirror audit-spec-coverage.sh: drop fields that serde wholly skips
            # so the two reports compute the struct-side set the same way.
            if (attr ~ /skip[^a-z_]/ && attr !~ /skip_serializing_if/ && attr !~ /skip_serializing[^_]/ && attr !~ /skip_deserializing[^_]/) {
                skip_next = 1
            }
            if (attr ~ /\<flatten\>/) skip_next = 1
            if (match(attr, /rename[ \t]*=[ \t]*"[^"]+"/)) {
                m = substr(attr, RSTART, RLENGTH)
                sub(/^rename[ \t]*=[ \t]*"/, "", m); sub(/"$/, "", m); rename = m
            }
        }
        if (match($0, /^[ \t]+pub[ \t]+[A-Za-z_][A-Za-z0-9_]*[ \t]*:/)) {
            if (skip_next) { skip_next = 0; rename = ""; next }
            fld = $0
            sub(/^[ \t]+pub[ \t]+/, "", fld); sub(/[ \t]*:.*$/, "", fld)
            print (rename != "" ? rename : pascalize(fld))
            rename = ""
        }
    }
' "$STRUCT_FILE" | sort -u > "$STRUCT_TXT"
STRUCT_COUNT=$(wc -l < "$STRUCT_TXT" | tr -d ' ')

# --- Diffs -------------------------------------------------------------------
F_MINUS_S="$TMP/fixture_minus_struct.txt"   # in fixture, not in struct
S_MINUS_F="$TMP/struct_minus_fixture.txt"   # in struct, not in fixture
comm -23 "$FIXTURE_TXT" "$STRUCT_TXT" > "$F_MINUS_S"
comm -13 "$FIXTURE_TXT" "$STRUCT_TXT" > "$S_MINUS_F"
F_MINUS_S_COUNT=$(wc -l < "$F_MINUS_S" | tr -d ' ')
S_MINUS_F_COUNT=$(wc -l < "$S_MINUS_F" | tr -d ' ')

if [ -s "$SPEC_TXT" ]; then
    F_MINUS_SPEC="$TMP/fixture_minus_spec.txt"
    SPEC_MINUS_F="$TMP/spec_minus_fixture.txt"
    comm -23 "$FIXTURE_TXT" "$SPEC_TXT" > "$F_MINUS_SPEC"
    comm -13 "$FIXTURE_TXT" "$SPEC_TXT" > "$SPEC_MINUS_F"
    F_MINUS_SPEC_COUNT=$(wc -l < "$F_MINUS_SPEC" | tr -d ' ')
    SPEC_MINUS_F_COUNT=$(wc -l < "$SPEC_MINUS_F" | tr -d ' ')
fi

# --- Emit markdown -----------------------------------------------------------
[ -n "$TITLE" ] && { echo "## $TITLE"; echo; }
cat <<EOF
- **Fixture:** \`$FIXTURE\` (jq prefix \`$JQ_PREFIX\`)
- **Struct:** \`$STRUCT_NAME\` (\`$STRUCT_FILE\`)
EOF
[ -n "$SPEC" ] && [ -n "$SCHEMA" ] && \
    echo "- **Spec schema:** \`$SCHEMA\` (\`$SPEC\`)"

cat <<EOF

| Metric | Count |
|---|---|
| Keys in fixture payload | $FIXTURE_COUNT |
| Fields in Rust struct | $STRUCT_COUNT |
| Spec properties | $SPEC_COUNT |
| **In fixture, missing from struct** | **$F_MINUS_S_COUNT** |
| In struct, not in fixture (server omitted or write-only) | $S_MINUS_F_COUNT |
EOF

if [ -s "$SPEC_TXT" ]; then
    cat <<EOF
| In fixture, not in spec (spec lag — undocumented API field) | $F_MINUS_SPEC_COUNT |
| In spec, not in fixture (documented but not returned here) | $SPEC_MINUS_F_COUNT |
EOF
fi

echo
echo "### Missing from struct (in fixture)"
echo
if [ "$F_MINUS_S_COUNT" -eq 0 ]; then
    echo "_None — struct sees every field the API returned._"
else
    while IFS= read -r f; do printf -- "- \`%s\`\n" "$f"; done < "$F_MINUS_S"
fi

if [ -s "$SPEC_TXT" ] && [ "${F_MINUS_SPEC_COUNT:-0}" -gt 0 ]; then
    echo
    echo "### In fixture, not in spec (spec lag)"
    echo
    while IFS= read -r f; do printf -- "- \`%s\`\n" "$f"; done < "$TMP/fixture_minus_spec.txt"
fi

exit 0
