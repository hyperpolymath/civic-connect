#!/usr/bin/env bash
set -euo pipefail
OUT=../.tmp/sbom.json
mkdir -p ../.tmp
echo "Generating SBOM (CycloneDX JSON)..."
# Example: use syft if available
if command -v syft >/dev/null; then
  syft . -o cyclonedx-json > "$OUT"
else
  echo '{"sbom":"stub"}' > "$OUT"
fi
echo "SBOM written to $OUT"
