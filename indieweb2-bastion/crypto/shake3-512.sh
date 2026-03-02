#!/usr/bin/env bash
# SPDX-License-Identifier: PMPL-1.0-or-later
# Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <jonathan.jewell@open.ac.uk>
#
# SHAKE3-512 hash utility (FIPS 202) — per CRYPTO-POLICY.adoc CPR-002
# Outputs 512-bit (64-byte) SHAKE3-512 digest of the given file.
set -euo pipefail

FILE="$1"
if [ -z "$FILE" ]; then
  echo "Usage: shake3-512.sh <file>"
  exit 2
fi

if command -v openssl >/dev/null; then
  # OpenSSL 3.x supports SHAKE via -shake256 with -xoflen
  # SHAKE3-512 = SHAKE256 with 64-byte output (Keccak-based, FIPS 202)
  openssl dgst -shake256 -xoflen 64 "$FILE"
else
  echo "error: openssl 3.x required for SHAKE3-512 support" >&2
  exit 1
fi
