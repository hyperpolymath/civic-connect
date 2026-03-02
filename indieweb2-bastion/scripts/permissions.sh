#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-default}"

case "$MODE" in
  default)
    echo "Setting strict permissions..."
    chmod -R go-rwx .tmp build sbom logging
    chmod -R u+rwX .tmp build sbom logging
    ;;
  relax)
    echo "Relaxing permissions for maintenance..."
    chmod -R u+rwX,g+rwX .tmp build sbom logging
    ;;
  restore)
    echo "Restoring strict permissions..."
    chmod -R go-rwx .tmp build sbom logging
    chmod -R u+rwX .tmp build sbom logging
    ;;
  *)
    echo "Usage: permissions.sh [default|relax|restore]"
    exit 2
    ;;
esac
