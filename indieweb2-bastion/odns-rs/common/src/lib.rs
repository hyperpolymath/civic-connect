// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <jonathan.jewell@open.ac.uk>
//
// oDNS Common — Shared cryptographic primitives and protocol framing
//
// Cryptographic stack (per CRYPTO-POLICY.adoc):
//   KEM:       Kyber-1024 / ML-KEM-1024 (FIPS 203) — CPR-004
//   KDF:       HKDF-SHA3-512 (RFC 5869 + FIPS 202) — CPR-007
//   Symmetric: XChaCha20-Poly1305 (extended nonce)  — CPR-006
//   RNG:       ChaCha20-DRBG via OsRng              — CPR-008
//   Sigs:      Ed448 + Dilithium5 hybrid            — CPR-005

pub mod crypto;
pub mod protocol;
pub mod signatures;
pub mod sphincs_fallback;

pub use crypto::{decrypt_query, encrypt_query, generate_keypair, PublicKey, SecretKey};
