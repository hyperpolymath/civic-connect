// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <jonathan.jewell@open.ac.uk>
//
// Hybrid Ed448 + Dilithium5 signature scheme (CPR-005 per CRYPTO-POLICY.adoc).
//
// Combines a classical (Ed448, RFC 8032) and a post-quantum (Dilithium5/ML-DSA-87,
// FIPS 204) signature. Both must verify for the hybrid to succeed, providing
// security even if one algorithm is compromised.
//
// Wire format:
//   HybridPublicKey:  [57 bytes Ed448 vk] [2592 bytes Dilithium5 pk]
//   HybridSignature:  [114 bytes Ed448 sig] [4627 bytes Dilithium5 sig]

use ed448_goldilocks_plus::{SigningKey, VerifyingKey};
use pqcrypto_dilithium::dilithium5;
use pqcrypto_traits::sign::{
    DetachedSignature as DilDetachedSigTrait, PublicKey as DilPkTrait,
};

/// Ed448 public key size (bytes).
pub const ED448_PK_LEN: usize = 57;
/// Ed448 signature size (bytes).
pub const ED448_SIG_LEN: usize = 114;
/// Dilithium5 public key size (bytes).
pub const DIL5_PK_LEN: usize = 2592;
/// Dilithium5 signature size (bytes).
pub const DIL5_SIG_LEN: usize = 4627;
/// Combined hybrid public key size.
pub const HYBRID_PK_LEN: usize = ED448_PK_LEN + DIL5_PK_LEN;
/// Combined hybrid signature size.
pub const HYBRID_SIG_LEN: usize = ED448_SIG_LEN + DIL5_SIG_LEN;

#[derive(Debug, thiserror::Error)]
pub enum SignatureError {
    #[error("Ed448 signature verification failed")]
    Ed448VerifyFailed,

    #[error("Dilithium5 signature verification failed")]
    Dilithium5VerifyFailed,

    #[error("invalid public key: expected {expected} bytes, got {got}")]
    InvalidPublicKey { expected: usize, got: usize },

    #[error("invalid signature: expected {expected} bytes, got {got}")]
    InvalidSignature { expected: usize, got: usize },

    #[error("invalid Ed448 public key")]
    InvalidEd448Key,

    #[error("invalid Ed448 signature")]
    InvalidEd448Signature,

    #[error("invalid Dilithium5 public key")]
    InvalidDilithiumKey,

    #[error("invalid Dilithium5 signature")]
    InvalidDilithiumSignature,
}

/// Hybrid keypair: Ed448 + Dilithium5.
pub struct HybridKeyPair {
    pub ed448_sk: SigningKey,
    pub ed448_vk: VerifyingKey,
    pub dil5_pk: dilithium5::PublicKey,
    pub dil5_sk: dilithium5::SecretKey,
}

/// Hybrid public key (Ed448 + Dilithium5).
pub struct HybridPublicKey {
    pub ed448_vk: VerifyingKey,
    pub dil5_pk: dilithium5::PublicKey,
}

/// Hybrid signature (Ed448 + Dilithium5).
pub struct HybridSignature {
    pub ed448_sig: ed448_goldilocks_plus::Signature,
    pub dil5_sig: dilithium5::DetachedSignature,
}

/// Generate a new hybrid Ed448 + Dilithium5 keypair.
pub fn generate_hybrid_keypair() -> HybridKeyPair {
    // Ed448 keypair
    let ed448_sk = SigningKey::generate(&mut rand::rngs::OsRng);
    let ed448_vk = ed448_sk.verifying_key();

    // Dilithium5 keypair
    let (dil5_pk, dil5_sk) = dilithium5::keypair();

    HybridKeyPair {
        ed448_sk,
        ed448_vk,
        dil5_pk,
        dil5_sk,
    }
}

/// Sign a message with both Ed448 and Dilithium5.
pub fn hybrid_sign(message: &[u8], keypair: &HybridKeyPair) -> HybridSignature {
    let ed448_sig = keypair.ed448_sk.sign_raw(message);
    let dil5_sig = dilithium5::detached_sign(message, &keypair.dil5_sk);

    HybridSignature {
        ed448_sig,
        dil5_sig,
    }
}

/// Verify a hybrid signature. Both Ed448 AND Dilithium5 signatures must verify.
pub fn hybrid_verify(
    message: &[u8],
    signature: &HybridSignature,
    public_key: &HybridPublicKey,
) -> Result<(), SignatureError> {
    // Verify Ed448
    public_key
        .ed448_vk
        .verify_raw(&signature.ed448_sig, message)
        .map_err(|_| SignatureError::Ed448VerifyFailed)?;

    // Verify Dilithium5
    dilithium5::verify_detached_signature(&signature.dil5_sig, message, &public_key.dil5_pk)
        .map_err(|_| SignatureError::Dilithium5VerifyFailed)?;

    Ok(())
}

impl HybridPublicKey {
    /// Serialize to bytes: `[Ed448 vk (57)] [Dilithium5 pk (2592)]`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HYBRID_PK_LEN);
        let ed448_bytes = self.ed448_vk.to_bytes();
        bytes.extend_from_slice(&ed448_bytes);
        bytes.extend_from_slice(self.dil5_pk.as_bytes());
        bytes
    }

    /// Deserialize from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SignatureError> {
        if bytes.len() != HYBRID_PK_LEN {
            return Err(SignatureError::InvalidPublicKey {
                expected: HYBRID_PK_LEN,
                got: bytes.len(),
            });
        }

        let mut ed448_bytes = [0u8; ED448_PK_LEN];
        ed448_bytes.copy_from_slice(&bytes[..ED448_PK_LEN]);
        let ed448_vk = VerifyingKey::from_bytes(&ed448_bytes)
            .map_err(|_| SignatureError::InvalidEd448Key)?;

        let dil5_pk = dilithium5::PublicKey::from_bytes(&bytes[ED448_PK_LEN..])
            .map_err(|_| SignatureError::InvalidDilithiumKey)?;

        Ok(Self { ed448_vk, dil5_pk })
    }
}

impl HybridSignature {
    /// Serialize to bytes: `[Ed448 sig (114)] [Dilithium5 sig (4627)]`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HYBRID_SIG_LEN);
        let ed448_bytes = self.ed448_sig.to_bytes();
        bytes.extend_from_slice(&ed448_bytes);
        bytes.extend_from_slice(self.dil5_sig.as_bytes());
        bytes
    }

    /// Deserialize from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SignatureError> {
        if bytes.len() != HYBRID_SIG_LEN {
            return Err(SignatureError::InvalidSignature {
                expected: HYBRID_SIG_LEN,
                got: bytes.len(),
            });
        }

        let mut ed448_bytes = [0u8; ED448_SIG_LEN];
        ed448_bytes.copy_from_slice(&bytes[..ED448_SIG_LEN]);
        let ed448_sig = ed448_goldilocks_plus::Signature::from_bytes(&ed448_bytes)
            .map_err(|_| SignatureError::InvalidEd448Signature)?;

        let dil5_sig = dilithium5::DetachedSignature::from_bytes(&bytes[ED448_SIG_LEN..])
            .map_err(|_| SignatureError::InvalidDilithiumSignature)?;

        Ok(Self {
            ed448_sig,
            dil5_sig,
        })
    }
}

impl HybridKeyPair {
    /// Extract the public key from the keypair.
    pub fn public_key(&self) -> Result<HybridPublicKey, SignatureError> {
        let vk_bytes = self.ed448_vk.to_bytes();
        let ed448_vk = VerifyingKey::from_bytes(&vk_bytes)
            .map_err(|_| SignatureError::InvalidEd448Key)?;
        let dil5_pk = dilithium5::PublicKey::from_bytes(self.dil5_pk.as_bytes())
            .map_err(|_| SignatureError::InvalidDilithiumKey)?;
        Ok(HybridPublicKey { ed448_vk, dil5_pk })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hybrid_sign_verify_roundtrip() {
        let kp = generate_hybrid_keypair();
        let msg = b"test message for hybrid signature";

        let sig = hybrid_sign(msg, &kp);
        let pk = kp.public_key().unwrap();

        assert!(hybrid_verify(msg, &sig, &pk).is_ok());
    }

    #[test]
    fn hybrid_rejects_wrong_message() {
        let kp = generate_hybrid_keypair();
        let sig = hybrid_sign(b"original", &kp);
        let pk = kp.public_key().unwrap();

        let result = hybrid_verify(b"tampered", &sig, &pk);
        assert!(result.is_err());
    }

    #[test]
    fn hybrid_rejects_wrong_key() {
        let kp1 = generate_hybrid_keypair();
        let kp2 = generate_hybrid_keypair();

        let sig = hybrid_sign(b"test", &kp1);
        let pk2 = kp2.public_key().unwrap();

        let result = hybrid_verify(b"test", &sig, &pk2);
        assert!(result.is_err());
    }

    #[test]
    fn public_key_serialization_roundtrip() {
        let kp = generate_hybrid_keypair();
        let pk = kp.public_key().unwrap();

        let bytes = pk.to_bytes();
        assert_eq!(bytes.len(), HYBRID_PK_LEN);

        let pk2 = HybridPublicKey::from_bytes(&bytes).unwrap();
        assert_eq!(pk.ed448_vk.to_bytes(), pk2.ed448_vk.to_bytes());
        assert_eq!(pk.dil5_pk.as_bytes(), pk2.dil5_pk.as_bytes());
    }

    #[test]
    fn signature_serialization_roundtrip() {
        let kp = generate_hybrid_keypair();
        let msg = b"serialize this signature";
        let sig = hybrid_sign(msg, &kp);

        let bytes = sig.to_bytes();
        assert_eq!(bytes.len(), HYBRID_SIG_LEN);

        let sig2 = HybridSignature::from_bytes(&bytes).unwrap();
        let pk = kp.public_key().unwrap();
        assert!(hybrid_verify(msg, &sig2, &pk).is_ok());
    }
}
