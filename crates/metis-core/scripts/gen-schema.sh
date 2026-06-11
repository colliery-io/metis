#!/usr/bin/env bash
# Regenerate the dual-backend schema from logical DDL.
#
# Reads schema/migrations/ (logical DDL) and writes schema/generated/
# (per-backend migrations + unified schema.rs). The generated tree is committed
# and treated as read-only; rerun this whenever the logical migrations change.
#
# Requires the diesel-dualdb-schema binary. During 3.0 development it is built
# from the sibling checkout; once diesel-dualdb is published, prefer
# `cargo install diesel-dualdb-cli` and call `diesel-dualdb-schema` directly.
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DUALDB_DIR="${DUALDB_DIR:-$CRATE_DIR/../../../diesel-dualdb}"

MIGRATIONS="$CRATE_DIR/schema/migrations"
GENERATED="$CRATE_DIR/schema/generated"

if command -v diesel-dualdb-schema >/dev/null 2>&1; then
    diesel-dualdb-schema "$MIGRATIONS" "$GENERATED"
elif [ -d "$DUALDB_DIR" ]; then
    ( cd "$DUALDB_DIR" && cargo run -q -p diesel-dualdb-cli -- "$MIGRATIONS" "$GENERATED" )
else
    echo "error: diesel-dualdb-schema not found and DUALDB_DIR ($DUALDB_DIR) missing" >&2
    exit 1
fi

echo "schema regenerated at $GENERATED"
