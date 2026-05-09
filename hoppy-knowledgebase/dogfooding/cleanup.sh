#!/usr/bin/env bash
# Dogfooding cleanup SKELETON — currently a placeholder. Each surface block
# below is a stub that prints the manual command to run; no deletes happen.
# When implemented, it will list and (with --yes) delete every bunny.net
# resource whose name starts with the dogfooding prefix, idempotently.
#
# Default mode is dry-run. Pass --yes to actually delete (refused today —
# see the "not yet implemented" guard below).
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

# Until real deletion logic exists, refuse --yes so operators don't think
# cleanup ran when nothing was deleted.
if [[ "$DRY_RUN" == 0 ]]; then
    echo "cleanup.sh is a placeholder — --yes is not yet supported." >&2
    echo "Run \`hoppy <noun> list\` manually and delete matches via the dashboard or" >&2
    echo "via \`hoppy <noun> delete\`. Track implementation in iteration-25 / backlog." >&2
    exit 2
fi

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
    #   ids=$("$HOPPY" pull-zone list --output json | jq -r '.[] | select(.Name | startswith("'"$PREFIX"'")) | .Id')
    #   for id in $ids; do
    #       if [[ "$DRY_RUN" == 1 ]]; then
    #           echo "  [dry-run] would delete pull zone $id"
    #       else
    #           "$HOPPY" pull-zone delete --id "$id" --yes
    #       fi
    #   done
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
