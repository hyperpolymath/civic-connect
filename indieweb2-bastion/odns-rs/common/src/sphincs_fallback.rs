// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <jonathan.jewell@open.ac.uk>
//
//! SPHINCS+ fallback signature scheme (CPR-012 per CRYPTO-POLICY.adoc).
//!
//! Provides stateless hash-based signatures as an algorithmic diversity fallback.
//! Uses SPHINCS+-SHA2-256s-simple (SLH-DSA-SHA2-256s per FIPS 205).
//!
//! This module is independent of the primary Ed448+Dilithium5 hybrid (CPR-005).
//! It exists as a last-resort fallback if both Ed448 and Dilithium5 are compromised.
//!
//! Properties:
//!   - Stateless: no per-signature state to manage
//!   - Hash-based: security relies only on hash function (SHA-256)
//!   - Conservative: well-understood security reduction
//!   - Trade-off: larger signatures (~29 KiB) and slower signing

use pqcrypto_sphincsplus::sphincssha2256ssimple;
use pqcrypto_traits::sign::{
    DetachedSignature as DetachedSigTrait, PublicKey as PkTrait, SecretKey as SkTrait,
};

/// SPHINCS+ public key size (bytes).
pub const SPHINCS_PK_LEN: usize = 32;
/// SPHINCS+ secret key size (bytes).
pub const SPHINCS_SK_LEN: usize = 64;
/// SPHINCS+ signature size (bytes) — SHA2-256s-simple.
pub const SPHINCS_SIG_LEN: usize = 29792;

#[derive(Debug, thiserror::Error)]
pub enum SphincsError {
    #[error("SPHINCS+ signature verification failed")]
    VerifyFailed,

    #[error("invalid SPHINCS+ public key: expected {expected} bytes, got {got}")]
    InvalidPublicKey { expected: usize, got: usize },

    #[error("invalid SPHINCS+ secret key: expected {expected} bytes, got {got}")]
    InvalidSecretKey { expected: usize, got: usize },

    #[error("invalid SPHINCS+ signature: expected {expected} bytes, got {got}")]
    InvalidSignature { expected: usize, got: usize },

    #[error("failed to deserialize SPHINCS+ public key")]
    DeserializePublicKey,

    #[error("failed to deserialize SPHINCS+ secret key")]
    DeserializeSecretKey,

    #[error("failed to deserialize SPHINCS+ signature")]
    DeserializeSignature,
}

/// SPHINCS+ keypair (SHA2-256s-simple / SLH-DSA-SHA2-256s).
pub struct SphincsKeyPair {
    pub pk: sphincssha2256ssimple::PublicKey,
    pub sk: sphincssha2256ssimple::SecretKey,
}

/// Generate a new SPHINCS+ keypair.
pub fn generate_sphincs_keypair() -> SphincsKeyPair {
    let (pk, sk) = sphincssha2256ssimple::keypair();
    SphincsKeyPair { pk, sk }
}

/// Sign a message with SPHINCS+ (detached signature).
pub fn sphincs_sign(message: &[u8], keypair: &SphincsKeyPair) -> Vec<u8> {
    let sig = sphincssha2256ssimple::detached_sign(message, &keypair.sk);
    sig.as_bytes().to_vec()
}

/// Verify a SPHINCS+ detached signature.
pub fn sphincs_verify(
    message: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> Result<(), SphincsError> {
    let pk = sphincssha2256ssimple::PublicKey::from_bytes(public_key)
        .map_err(|_| SphincsError::DeserializePublicKey)?;

    let sig = sphincssha2256ssimple::DetachedSignature::from_bytes(signature)
        .map_err(|_| SphincsError::DeserializeSignature)?;

    sphincssha2256ssimple::verify_detached_signature(&sig, message, &pk)
        .map_err(|_| SphincsError::VerifyFailed)
}

/// Serialize a SPHINCS+ public key to bytes.
pub fn public_key_bytes(keypair: &SphincsKeyPair) -> Vec<u8> {
    keypair.pk.as_bytes().to_vec()
}

/// Serialize a SPHINCS+ secret key to bytes.
pub fn secret_key_bytes(keypair: &SphincsKeyPair) -> Vec<u8> {
    keypair.sk.as_bytes().to_vec()
}

/// Deserialize a SPHINCS+ public key from bytes.
pub fn public_key_from_bytes(
    bytes: &[u8],
) -> Result<sphincssha2256ssimple::PublicKey, SphincsError> {
    sphincssha2256ssimple::PublicKey::from_bytes(bytes)
        .map_err(|_| SphincsError::DeserializePublicKey)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphincs_sign_verify_roundtrip() {
        let kp = generate_sphincs_keypair();
        let msg = b"test message for SPHINCS+ fallback";

        let sig = sphincs_sign(msg, &kp);
        let pk = public_key_bytes(&kp);

        assert!(sphincs_verify(msg, &sig, &pk).is_ok());
    }

    #[test]
    fn sphincs_rejects_wrong_message() {
        let kp = generate_sphincs_keypair();
        let sig = sphincs_sign(b"original", &kp);
        let pk = public_key_bytes(&kp);

        let result = sphincs_verify(b"tampered", &sig, &pk);
        assert!(result.is_err());
    }

    #[test]
    fn sphincs_rejects_wrong_key() {
        let kp1 = generate_sphincs_keypair();
        let kp2 = generate_sphincs_keypair();

        let sig = sphincs_sign(b"test", &kp1);
        let pk2 = public_key_bytes(&kp2);

        let result = sphincs_verify(b"test", &sig, &pk2);
        assert!(result.is_err());
    }

    #[test]
    fn sphincs_key_serialization_roundtrip() {
        let kp = generate_sphincs_keypair();
        let pk_bytes = public_key_bytes(&kp);

        let pk_restored = public_key_from_bytes(&pk_bytes);
        assert!(pk_restored.is_ok());
        assert_eq!(pk_restored.unwrap().as_bytes(), kp.pk.as_bytes());
    }

    #[test]
    fn sphincs_signature_size() {
        let kp = generate_sphincs_keypair();
        let sig = sphincs_sign(b"size check", &kp);
        // SPHINCS+-SHA2-256s-simple signatures are ~29 KiB
        assert!(sig.len() > 20_000, "SPHINCS+ sig should be ~29KiB, got {}", sig.len());
    }
}
