#!/usr/bin/env bash
#
# cleanup.sh — delete bunny.net resources left behind by hoppy's live-API test
# suite and by manual dogfooding sessions.
#
# WHY THIS EXISTS
#   `cargo test --features live-api` creates real pull zones, storage zones,
#   DNS zones, edge scripts, video libraries and Magic Containers apps. When a
#   test panics before its cleanup stack unwinds, those resources survive —
#   container apps in particular bill by the hour. This script sweeps them.
#
# SAFETY MODEL — CLOSED ALLOWLIST
#   Only resources whose name starts with one of these test prefixes are ever
#   considered for deletion:
#
#       hoppy-test-          pull zones, storage zones, video libraries
#       hoppytest-           DNS zones (live tests create <name>.test domains)
#       hoppy-edge-rule-     pull zones (edge-rule lifecycle test)
#       hoppy-shield-test-   pull zones (shield lifecycle test)
#       hpmc-                Magic Containers apps
#       hpst-                storage zones
#       hpsc- hpscs- hpscv-  edge scripts (script / secret / variable tests)
#
#   `--prefix=<p>` NARROWS the sweep to a single prefix. It can never widen it:
#   any `<p>` that does not itself begin with one of the prefixes above is
#   refused. An empty `--prefix=` is refused too.
#
# DRY RUN IS THE DEFAULT
#   With no arguments the script only lists what it would delete, one line per
#   resource (surface, id, name). Pass `--yes` (or `-y`) to actually delete.
#
# AUTH
#   Reads BUNNY_API_KEY from the environment; the key is never echoed. Point it
#   at the throwaway test account, e.g.:
#
#       BUNNY_API_KEY="$TEST_BUNNY_API_KEY" hoppy-knowledgebase/dogfooding/cleanup.sh
#       BUNNY_API_KEY="$TEST_BUNNY_API_KEY" hoppy-knowledgebase/dogfooding/cleanup.sh --yes
#
# EXIT CODES
#   0  nothing matched, or every delete succeeded
#   1  at least one delete (or list) failed — details in the summary
#   2  misuse: missing binary, missing jq, missing key, bad prefix
#
# NOTE ON TOOLING POLICY
#   CLAUDE.md requires all build and runtime code to be Rust. This file is
#   dogfooding tooling under hoppy-knowledgebase/ and is explicitly exempt: it
#   is an auditable shell snippet, not part of the build.
#
# PORTABILITY
#   Plain bash + jq, kept compatible with the bash 3.2 that ships on macOS:
#   no associative arrays, no `mapfile`, no globstar.

set -euo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

# Closed allowlist. Space separated; every entry must be a prefix that only
# test/dogfooding resources can plausibly carry.
ALLOWED_PREFIXES="hoppy-test- hoppytest- hoppy-edge-rule- hoppy-shield-test- hpmc- hpst- hpsc- hpscs- hpscv-"

HOPPY="${HOPPY_BIN:-./target/release/hoppy}"
DRY_RUN=1
PREFIX_OVERRIDE=""
HAVE_OVERRIDE=0

for arg in "$@"; do
    case "$arg" in
        --yes|-y) DRY_RUN=0 ;;
        --prefix=*) PREFIX_OVERRIDE="${arg#--prefix=}"; HAVE_OVERRIDE=1 ;;
        --help|-h)
            sed -n '2,51p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *) echo "unknown arg: $arg (try --help)" >&2; exit 2 ;;
    esac
done

# ---------------------------------------------------------------------------
# Pre-flight guards
# ---------------------------------------------------------------------------

if [[ ! -x "$HOPPY" ]]; then
    echo "hoppy binary not found at $HOPPY — run: cargo build --release" >&2
    echo "(override the path with HOPPY_BIN=/path/to/hoppy)" >&2
    exit 2
fi

if ! command -v jq >/dev/null 2>&1; then
    echo "jq not found on PATH — required to parse hoppy's JSON output" >&2
    exit 2
fi

if [[ -z "${BUNNY_API_KEY:-}" ]]; then
    echo "BUNNY_API_KEY is not set — refusing to run." >&2
    echo "Export the key for your throwaway test account before invoking this script." >&2
    exit 2
fi

# Resolve the active prefix set: the whole allowlist, or the single narrowed
# prefix the caller asked for.
ACTIVE_PREFIXES="$ALLOWED_PREFIXES"
if [[ "$HAVE_OVERRIDE" -eq 1 ]]; then
    if [[ -z "$PREFIX_OVERRIDE" ]]; then
        echo "refusing to run with an empty --prefix= — it would match every resource" >&2
        exit 2
    fi
    allowed=0
    for p in $ALLOWED_PREFIXES; do
        case "$PREFIX_OVERRIDE" in
            "$p"*) allowed=1; break ;;
        esac
    done
    if [[ "$allowed" -eq 0 ]]; then
        echo "refusing --prefix='$PREFIX_OVERRIDE': it does not start with an allowlisted test prefix." >&2
        echo "--prefix may only narrow the sweep. Allowlist: $ALLOWED_PREFIXES" >&2
        exit 2
    fi
    ACTIVE_PREFIXES="$PREFIX_OVERRIDE"
fi

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

MATCHED=0
DELETED=0
FAILED=0
FAILURES=""

record_failure() {
    FAILED=$((FAILED + 1))
    FAILURES="${FAILURES}  - $1
"
}

# True when $1 starts with one of the active prefixes.
matches_prefix() {
    local name="$1" p
    for p in $ACTIVE_PREFIXES; do
        case "$name" in
            "$p"*) return 0 ;;
        esac
    done
    return 1
}

# list_json <surface-label> <hoppy args...>
#
# Runs a list command and leaves its stdout in the file named by $LIST_OUT.
# Returns non-zero (and records a failure) when the command fails. stderr goes
# into the failure message so hints and progress noise never reach the JSON
# parser.
#
# The output goes to a file rather than stdout on purpose: callers would have to
# wrap this in a command substitution, and the resulting subshell would discard
# every counter this function updates.
LIST_OUT=""
TMP_DIR="$(mktemp -d)"

# shellcheck disable=SC2329  # invoked indirectly by the EXIT trap below
cleanup_tmp_dir() {
    rm -rf "$TMP_DIR"
}
trap cleanup_tmp_dir EXIT

list_json() {
    local label="$1"
    shift
    local err_file rc=0
    LIST_OUT="$(mktemp "$TMP_DIR/list.XXXXXX")"
    err_file="$(mktemp "$TMP_DIR/err.XXXXXX")"
    "$HOPPY" "$@" --format json >"$LIST_OUT" 2>"$err_file" || rc=$?
    if [[ "$rc" -ne 0 ]]; then
        record_failure "$label: list failed: $(tr '\n' ' ' <"$err_file")"
        return 1
    fi
    return 0
}

# delete_one <surface> <id> <name> -- <hoppy args...>
delete_one() {
    local surface="$1" id="$2" name="$3"
    shift 3
    MATCHED=$((MATCHED + 1))
    if [[ "$DRY_RUN" -eq 1 ]]; then
        printf '  [dry-run] %s\t%s\t%s\n' "$surface" "$id" "$name"
        return 0
    fi
    printf '  deleting  %s\t%s\t%s\n' "$surface" "$id" "$name"
    local out rc=0
    out="$("$HOPPY" "$@" 2>&1)" || rc=$?
    if [[ "$rc" -ne 0 ]]; then
        record_failure "$surface $id ($name): $(printf '%s' "$out" | tr '\n' ' ')"
    else
        DELETED=$((DELETED + 1))
    fi
}

# ---------------------------------------------------------------------------
# Surfaces
#
# Deletion order matters:
#   1. container apps  — `--cascade` also removes their auto-managed pull zones,
#                        so they must go before the pull-zone sweep.
#   2. video libraries — bunny.net tears down the library's own pull zone and
#                        storage zone with it.
#   3. edge scripts    — independent, but cheap to clear early.
#   4. pull zones      — whatever the cascades above did not already remove.
#   5. storage zones   — deleting one also deletes any still-linked pull zone
#                        (upstream default), hence after the pull-zone sweep.
#   6. DNS zones       — independent.
#   7. shield zones    — report only; there is no delete endpoint.
# ---------------------------------------------------------------------------

cleanup_container_apps() {
    echo "== container apps =="
    local json id name
    list_json "container apps" container app list --all || return 0
    json="$LIST_OUT"
    while IFS=$'\t' read -r id name; do
        [[ -n "$id" ]] || continue
        matches_prefix "$name" || continue
        delete_one "container-app" "$id" "$name" \
            container app delete --id "$id" --cascade --yes
    done < <(jq -r '.items[]? | "\(.id)\t\(.name)"' "$json")
}

cleanup_stream_libraries() {
    echo "== stream video libraries =="
    local json id name
    list_json "stream libraries" stream library list --all || return 0
    json="$LIST_OUT"
    while IFS=$'\t' read -r id name; do
        [[ -n "$id" ]] || continue
        matches_prefix "$name" || continue
        delete_one "stream-library" "$id" "$name" \
            stream library delete --id "$id" --yes
    done < <(jq -r '.Items[]? | "\(.Id)\t\(.Name)"' "$json")
}

cleanup_scripts() {
    echo "== edge scripts =="
    local json id name
    list_json "edge scripts" script list --all || return 0
    json="$LIST_OUT"
    while IFS=$'\t' read -r id name; do
        [[ -n "$id" ]] || continue
        matches_prefix "$name" || continue
        # Deliberately no --delete-linked-pull-zones: a script's own pull zone
        # carries the script's (test-prefixed) name and is caught by the
        # pull-zone sweep below, so we never reach outside the allowlist.
        delete_one "edge-script" "$id" "$name" \
            script delete --id "$id" --yes
    done < <(jq -r '.Items[]? | "\(.Id)\t\(.Name)"' "$json")
}

cleanup_pull_zones() {
    echo "== pull zones =="
    local json id name
    list_json "pull zones" pull-zone list --all || return 0
    json="$LIST_OUT"
    while IFS=$'\t' read -r id name; do
        [[ -n "$id" ]] || continue
        matches_prefix "$name" || continue
        delete_one "pull-zone" "$id" "$name" \
            pull-zone delete --id "$id" --yes
    done < <(jq -r '.Items[]? | "\(.Id)\t\(.Name)"' "$json")
}

cleanup_storage_zones() {
    echo "== storage zones =="
    local json id name
    list_json "storage zones" storage-zone list --all || return 0
    json="$LIST_OUT"
    while IFS=$'\t' read -r id name; do
        [[ -n "$id" ]] || continue
        matches_prefix "$name" || continue
        # Upstream default also deletes pull zones still linked to the zone;
        # by this point the pull-zone sweep has already cleared the test ones.
        delete_one "storage-zone" "$id" "$name" \
            storage-zone delete --id "$id" --yes
    done < <(jq -r '.Items[]? | "\(.Id)\t\(.Name)"' "$json")
}

cleanup_dns_zones() {
    echo "== dns zones =="
    local json id domain
    list_json "dns zones" dns zone list --all || return 0
    json="$LIST_OUT"
    while IFS=$'\t' read -r id domain; do
        [[ -n "$id" ]] || continue
        matches_prefix "$domain" || continue
        delete_one "dns-zone" "$id" "$domain" \
            dns zone delete --id "$id" --yes
    done < <(jq -r '.Items[]? | "\(.Id)\t\(.Domain)"' "$json")
}

# Shield zones cannot be deleted: the public API has no
# DELETE /shield/shield-zone/{id} (a speculative call returns 405). Test runs
# therefore leave inert server-side residue behind. All we can do is report how
# many shield zones point at a pull zone that no longer exists.
report_orphan_shield_zones() {
    echo "== shield zones (report only) =="
    local pz_json sz_json existing orphans=0 shield_id pz_id
    list_json "shield orphan check" pull-zone list --all || return 0
    pz_json="$LIST_OUT"
    list_json "shield zones" shield zone list --per-page 1000 || return 0
    sz_json="$LIST_OUT"
    existing="$(jq -r '.Items[]?.Id' "$pz_json")"
    while IFS=$'\t' read -r shield_id pz_id; do
        [[ -n "$shield_id" ]] || continue
        if ! printf '%s\n' "$existing" | grep -qx -- "$pz_id"; then
            orphans=$((orphans + 1))
        fi
    done < <(jq -r '.Items[]? | "\(.shieldZoneId)\t\(.pullZoneId)"' "$sz_json")
    echo "  $orphans orphaned shield zones (no delete API — ignore)"
}

# ---------------------------------------------------------------------------
# Run
# ---------------------------------------------------------------------------

if [[ "$DRY_RUN" -eq 1 ]]; then
    mode="dry-run"
else
    mode="DELETE"
fi
echo "dogfooding cleanup — mode=$mode"
echo "prefixes: $ACTIVE_PREFIXES"
echo

cleanup_container_apps
cleanup_stream_libraries
cleanup_scripts
cleanup_pull_zones
cleanup_storage_zones
cleanup_dns_zones
report_orphan_shield_zones

echo
if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "DRY RUN — $MATCHED resource(s) matched. Re-run with --yes to delete them."
else
    echo "deleted $DELETED of $MATCHED matched resource(s)."
fi

if [[ "$FAILED" -gt 0 ]]; then
    echo
    echo "$FAILED failure(s):"
    printf '%s' "$FAILURES"
    exit 1
fi

exit 0
