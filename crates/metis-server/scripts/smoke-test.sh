#!/usr/bin/env bash
# End-to-end smoke test for a Metis server: bootstrap a user + token, then check
# healthz, the unauthenticated 401, and an authenticated request.
#
# Works against any reachable server (a local `metis serve`, or the
# docker-compose stack). Requires the `metis` binary for the admin (bootstrap)
# steps and `curl` for the HTTP checks.
#
#   METIS_URL=http://localhost:7878 DATABASE_URL=postgres://… smoke-test.sh
set -euo pipefail

METIS_URL="${METIS_URL:-http://localhost:7878}"
DATABASE_URL="${DATABASE_URL:?set DATABASE_URL to the server database}"
METIS_BIN="${METIS_BIN:-metis}"
USER="smoke-$$"

fail() { echo "SMOKE FAIL: $1" >&2; exit 1; }

echo "1. healthz"
curl -fsS "$METIS_URL/healthz" >/dev/null || fail "healthz not OK"

echo "2. unauthenticated request is rejected"
code=$(curl -s -o /dev/null -w '%{http_code}' "$METIS_URL/api/v1/whoami")
[ "$code" = "401" ] || fail "expected 401, got $code"

echo "3. bootstrap user + token"
"$METIS_BIN" admin create-user "$USER" --admin --database-url "$DATABASE_URL" >/dev/null
TOKEN=$("$METIS_BIN" admin create-token --user "$USER" --agent-name smoke --database-url "$DATABASE_URL" | tail -1)
[ -n "$TOKEN" ] || fail "no token issued"

echo "4. authenticated request succeeds"
body=$(curl -fsS -H "Authorization: Bearer $TOKEN" "$METIS_URL/api/v1/whoami")
echo "$body" | grep -q '"agent":"smoke"' || fail "unexpected whoami body: $body"

echo "SMOKE OK"
