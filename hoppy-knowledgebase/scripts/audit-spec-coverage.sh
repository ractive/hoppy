#!/usr/bin/env sh
# audit-spec-coverage.sh — Compare a bunny.net OpenAPI schema's properties
# against a hand-written Rust struct's fields and emit a markdown report.
#
# This is a one-shot investigation tool, not shipped product code. It is
# intentionally implemented in POSIX shell + jq + awk so it can run in CI
# eventually without pulling in Rust tooling. Exit code is always 0 even
# when gaps exist — gaps are data, not failures.
#
# Usage:
#   audit-spec-coverage.sh \
#       --spec specs/core-platform.json \
#       --schema PullZoneModel \
#       --struct-file crates/bunny-net-api/src/core/types.rs \
#       --struct-name PullZone \
#       [--title "Pull zone (read shape)"]
#
# Output: markdown report fragment on stdout. Diagnostics on stderr.

set -eu

SPEC=""
SCHEMA=""
STRUCT_FILE=""
STRUCT_NAME=""
TITLE=""

usage() {
    cat >&2 <<EOF
Usage: $0 --spec <path> --schema <name> --struct-file <path> --struct-name <name> [--title <text>]

Emits a markdown report fragment comparing the named OpenAPI schema's
properties to the named Rust struct's fields. Exit 0 even when gaps exist.
EOF
    exit 2
}

while [ $# -gt 0 ]; do
    case "$1" in
        --spec) SPEC="$2"; shift 2 ;;
        --schema) SCHEMA="$2"; shift 2 ;;
        --struct-file) STRUCT_FILE="$2"; shift 2 ;;
        --struct-name) STRUCT_NAME="$2"; shift 2 ;;
        --title) TITLE="$2"; shift 2 ;;
        -h|--help) usage ;;
        *) echo "Unknown arg: $1" >&2; usage ;;
    esac
done

[ -n "$SPEC" ] || usage
[ -n "$SCHEMA" ] || usage
[ -n "$STRUCT_FILE" ] || usage
[ -n "$STRUCT_NAME" ] || usage

[ -f "$SPEC" ] || { echo "Spec file not found: $SPEC" >&2; exit 1; }
[ -f "$STRUCT_FILE" ] || { echo "Struct file not found: $STRUCT_FILE" >&2; exit 1; }

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

SPEC_TXT="$TMP/spec.txt"
STRUCT_TXT="$TMP/struct.txt"

# --- Spec → property names ---------------------------------------------------
# Resolve $ref if the schema is a simple alias. We only support a direct
# properties-bearing schema; allOf composition is reported via diagnostics.
jq -r --arg s "$SCHEMA" '
    .components.schemas[$s]
    | if . == null then
        error("schema not found: " + $s)
      elif has("properties") then
        .properties | keys[]
      elif has("allOf") then
        [.allOf[] | (if has("properties") then .properties | keys else [] end)]
        | add // []
        | .[]
      else
        empty
      end
' "$SPEC" 2>/dev/null | sort -u > "$SPEC_TXT" || {
    echo "Failed to extract properties for schema '$SCHEMA' from $SPEC" >&2
    exit 1
}

SPEC_COUNT=$(wc -l < "$SPEC_TXT" | tr -d ' ')

# --- Rust struct → field names ----------------------------------------------
# Strategy:
#   1. Find the line "pub struct <Name> {" and scan until matching "}" at col 1.
#   2. For each line, capture the most recent #[serde(rename = "X")] override.
#   3. For "pub field: …" lines without an override, convert snake_case → PascalCase.
#   4. Skip fields marked #[serde(skip)] or #[serde(flatten)].
awk -v target="$STRUCT_NAME" '
    function pascalize(s,   parts, n, i, out, p) {
        n = split(s, parts, "_")
        out = ""
        for (i = 1; i <= n; i++) {
            p = parts[i]
            if (length(p) == 0) continue
            out = out toupper(substr(p, 1, 1)) substr(p, 2)
        }
        return out
    }
    BEGIN { inside = 0; rename = ""; skip_next = 0 }
    {
        line = $0
    }
    !inside {
        # Match: pub struct Name { ... } or pub struct Name<T> {
        if (match(line, "^pub[ \t]+struct[ \t]+" target "([ \t<({].*)?[ \t]*\\{[ \t]*$")) {
            inside = 1
            next
        }
        next
    }
    inside {
        # End of struct
        if (line ~ /^\}/) { inside = 0; next }

        # Capture serde attributes
        if (match(line, /#\[serde\([^]]*\)\]/)) {
            attr = substr(line, RSTART, RLENGTH)
            if (attr ~ /skip[^a-z_]/ && attr !~ /skip_serializing_if/ && attr !~ /skip_serializing[^_]/ && attr !~ /skip_deserializing[^_]/) {
                # serde(skip) entirely skips the field from both sides
                skip_next = 1
            }
            if (attr ~ /\<flatten\>/) {
                skip_next = 1
            }
            if (match(attr, /rename[ \t]*=[ \t]*"[^"]+"/)) {
                m = substr(attr, RSTART, RLENGTH)
                sub(/^rename[ \t]*=[ \t]*"/, "", m)
                sub(/"$/, "", m)
                rename = m
            }
        }

        # pub field declaration
        if (match(line, /^[ \t]+pub[ \t]+[A-Za-z_][A-Za-z0-9_]*[ \t]*:/)) {
            if (skip_next) { skip_next = 0; rename = ""; next }
            fld = line
            sub(/^[ \t]+pub[ \t]+/, "", fld)
            sub(/[ \t]*:.*$/, "", fld)
            if (rename != "") {
                print rename
            } else {
                print pascalize(fld)
            }
            rename = ""
        }
    }
' "$STRUCT_FILE" | sort -u > "$STRUCT_TXT"

STRUCT_COUNT=$(wc -l < "$STRUCT_TXT" | tr -d ' ')

# --- Diff --------------------------------------------------------------------

# Gaps (in spec, missing from struct) — exact-case
MISSING="$TMP/missing.txt"
comm -23 "$SPEC_TXT" "$STRUCT_TXT" > "$MISSING"

# Reverse gap (in struct, missing from spec) — exact-case
REVERSE="$TMP/reverse.txt"
comm -13 "$SPEC_TXT" "$STRUCT_TXT" > "$REVERSE"

# Casing mismatches: lowercase intersection, but exact-case differ.
# Detected as: name appears in both MISSING and REVERSE when compared case-insensitively.
CASING="$TMP/casing.txt"
awk '{print tolower($0) "\t" $0}' "$MISSING"   | sort > "$TMP/m.lc"
awk '{print tolower($0) "\t" $0}' "$REVERSE"   | sort > "$TMP/r.lc"
join -t '	' "$TMP/m.lc" "$TMP/r.lc" \
    | awk -F'\t' '{ printf "%s ⇄ %s\n", $2, $3 }' \
    > "$CASING"

# Filter out casing-mismatched entries from MISSING and REVERSE.
if [ -s "$CASING" ]; then
    awk -F'\t' '{print $1}' "$TMP/m.lc" > "$TMP/mlc.names"
    awk -F'\t' '{print $1}' "$TMP/r.lc" > "$TMP/rlc.names"
    comm -12 "$TMP/mlc.names" "$TMP/rlc.names" > "$TMP/both.lc"
    awk 'NR==FNR{both[$1]=1; next} !(tolower($0) in both)' "$TMP/both.lc" "$MISSING" > "$TMP/missing.clean"
    awk 'NR==FNR{both[$1]=1; next} !(tolower($0) in both)' "$TMP/both.lc" "$REVERSE" > "$TMP/reverse.clean"
    mv "$TMP/missing.clean" "$MISSING"
    mv "$TMP/reverse.clean" "$REVERSE"
fi

MISSING_COUNT=$(wc -l < "$MISSING" | tr -d ' ')
REVERSE_COUNT=$(wc -l < "$REVERSE" | tr -d ' ')
CASING_COUNT=$(wc -l < "$CASING" | tr -d ' ')

# --- Emit markdown -----------------------------------------------------------
if [ -n "$TITLE" ]; then
    echo "# $TITLE"
    echo
fi

cat <<EOF
- **Spec file:** \`$SPEC\`
- **Schema:** \`$SCHEMA\`
- **Struct file:** \`$STRUCT_FILE\`
- **Struct:** \`$STRUCT_NAME\`

| Metric | Count |
|---|---|
| Properties in spec schema | $SPEC_COUNT |
| Fields in Rust struct | $STRUCT_COUNT |
| Missing from struct (gap) | $MISSING_COUNT |
| Casing mismatches | $CASING_COUNT |
| In struct, missing from spec (reverse gap) | $REVERSE_COUNT |

## Missing from struct (in spec, not in struct)

EOF

if [ "$MISSING_COUNT" -eq 0 ]; then
    echo "_None — struct covers every spec property._"
else
    while IFS= read -r line; do
        printf -- "- \`%s\`\n" "$line"
    done < "$MISSING"
fi

echo
echo "## Casing mismatches"
echo
if [ "$CASING_COUNT" -eq 0 ]; then
    echo "_None._"
else
    echo "These names match case-insensitively. Spec ⇄ Struct:"
    echo
    while IFS= read -r line; do
        printf -- "- %s\n" "$line"
    done < "$CASING"
fi

echo
echo "## Reverse gap (in struct, not in spec)"
echo
if [ "$REVERSE_COUNT" -eq 0 ]; then
    echo "_None._"
else
    echo "These fields may be stale or renamed in the API:"
    echo
    while IFS= read -r line; do
        printf -- "- \`%s\`\n" "$line"
    done < "$REVERSE"
fi

exit 0
