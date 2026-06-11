#!/usr/bin/env bash
# Fail if schema/generated/ is stale with respect to schema/migrations/.
#
# Regenerates into a temp dir and diffs against the committed output, so the
# generated schema can never silently drift from the logical DDL. Intended for
# CI (and a handy pre-commit check locally).
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DUALDB_DIR="${DUALDB_DIR:-$CRATE_DIR/../../../diesel-dualdb}"
MIGRATIONS="$CRATE_DIR/schema/migrations"
COMMITTED="$CRATE_DIR/schema/generated"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

if command -v diesel-dualdb-schema >/dev/null 2>&1; then
    diesel-dualdb-schema "$MIGRATIONS" "$TMP"
elif [ -d "$DUALDB_DIR" ]; then
    ( cd "$DUALDB_DIR" && cargo run -q -p diesel-dualdb-cli -- "$MIGRATIONS" "$TMP" )
else
    echo "error: diesel-dualdb-schema not found and DUALDB_DIR ($DUALDB_DIR) missing" >&2
    exit 1
fi

if ! diff -ru "$COMMITTED" "$TMP"; then
    echo "" >&2
    echo "error: schema/generated is out of date. Run scripts/gen-schema.sh and commit." >&2
    exit 1
fi

echo "schema/generated is up to date."
