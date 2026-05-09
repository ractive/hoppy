#!/usr/bin/env bash
# Dogfooding cleanup — list and (with --yes) delete every bunny.net resource
# whose name starts with the dogfooding prefix. Idempotent; safe to run before
# and after each session.
#
# Default mode is dry-run. Pass --yes to actually delete.
set -euo pipefail

PREFIX="${HOPPY_TEST_PREFIX:-hoppy-test-}"
HOPPY="${HOPPY_BIN:-./target/release/hoppy}"
DRY_RUN=1
for arg in "$@"; do
    case "$arg" in
        --yes|-y) DRY_RUN=0 ;;
        --prefix=*) PREFIX="${arg#--prefix=}" ;;
        *) echo "unknown arg: $arg" >&2; exit 2 ;;
    esac
done

if [[ ! -x "$HOPPY" ]]; then
    echo "hoppy binary not found at $HOPPY — run: cargo build --release" >&2
    exit 2
fi

if [[ -z "$PREFIX" ]]; then
    echo "refusing to run with empty prefix — would match every resource" >&2
    exit 2
fi

echo "dogfooding cleanup — prefix='$PREFIX' dry-run=$DRY_RUN"

# Each surface is its own list+filter+delete loop. Add new surfaces here
# as hoppy grows them. Each block must:
#   1. list resources via JSON
#   2. filter by name starting with "$PREFIX"
#   3. delete each match (or print, in dry-run)

cleanup_pull_zones() {
    echo "== pull zones =="
    # Placeholder: real implementation should:
    #   "$HOPPY" pull-zone list --output json | jq -r '.[] | select(.Name | startswith("'"$PREFIX"'")) | .Id'
    # then "$HOPPY" pull-zone delete --id <id> --yes
    # for each match.
    echo "  (not yet implemented — run \`hoppy pull-zone list\` manually and grep '$PREFIX')"
}

cleanup_storage_zones() {
    echo "== storage zones =="
    echo "  (not yet implemented — run \`hoppy storage-zone list\` manually and grep '$PREFIX')"
}

cleanup_dns_zones() {
    echo "== dns zones =="
    echo "  (not yet implemented — run \`hoppy dns zone list\` manually and grep '$PREFIX')"
}

cleanup_containers() {
    echo "== container apps =="
    echo "  (not yet implemented — run \`hoppy container app list\` manually and grep '$PREFIX')"
}

cleanup_pull_zones
cleanup_storage_zones
cleanup_dns_zones
cleanup_containers

if [[ "$DRY_RUN" == 1 ]]; then
    echo
    echo "DRY RUN — re-run with --yes to actually delete."
fi
