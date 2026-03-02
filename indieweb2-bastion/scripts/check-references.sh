#!/usr/bin/env bash
set -euo pipefail

FAIL=0

# 1) Nickel capabilities referenced by roles or routes
CAPS=$(grep -Po '(?<=name = ")[^"]+' policy/curps/policy.ncl || true)
for cap in $CAPS; do
  REF=$(grep -E "privileges.*${cap}|mutations.*${cap}" -r policy/curps/policy.ncl || true)
  STUB=$(grep -r "# stub" policy/curps/policy.ncl || true)
  if [[ -z "$REF" && -z "$STUB" ]]; then
    echo "❌ Capability '$cap' is unreferenced (and not stubbed)."
    FAIL=1
  fi
done

# 2) Cap'n Proto fields mapped in SurrealDB (basic check by field names)
FIELDS=$(grep -Po '^\s+\w+\s+@\d+\s+:' surrealdb/provenance.capnp | awk '{print $1}' || true)
for field in $FIELDS; do
  MAP=$(grep -r "$field" surrealdb/schema.surql || true)
  if [[ -z "$MAP" ]]; then
    echo "❌ Field '$field' not mapped in SurrealDB schema."
    FAIL=1
  fi
done

# 3) Docs linked in README.adoc
for doc in docs/*.adoc; do
  base=$(basename "$doc")
  LINK=$(grep -F "$base" README.adoc || true)
  if [[ -z "$LINK" ]]; then
    echo "❌ Doc '$base' not linked in README.adoc."
    FAIL=1
  fi
done

# Verdict
if [[ "$FAIL" -eq 1 ]]; then
  echo "❌ Reference check failed."
  exit 1
fi

echo "✅ Reference check passed."
