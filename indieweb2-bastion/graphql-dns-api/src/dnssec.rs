// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <jonathan.jewell@open.ac.uk>
//
//! DNSSEC key generation and zone signing — Hybrid Ed448 + Dilithium5 (CPR-005)
//!
//! Implements:
//! - KSK (Key Signing Key) generation: hybrid Ed448 + Dilithium5
//! - ZSK (Zone Signing Key) generation: hybrid Ed448 + Dilithium5
//! - DS record generation for parent zone (BLAKE3 digest per CPR-009)
//! - RRSIG generation and verification using hybrid signatures
//!
//! Algorithm number 253 (private-use per RFC 4034 §A.1.1) for hybrid scheme.
//! Wire format: [Ed448 (57/114 bytes)] [Dilithium5 (2592/4627 bytes)]

use crate::error::{AppError, Result};
use base64::Engine as _;
use ed448_goldilocks_plus::{SigningKey, VerifyingKey};
use pqcrypto_dilithium::dilithium5;
use pqcrypto_traits::sign::{
    DetachedSignature as DilDetachedSigTrait, PublicKey as DilPkTrait,
};

/// DNSSEC algorithm number for hybrid Ed448+Dilithium5 (private-use, RFC 4034 §A.1.1).
pub const ALGORITHM_HYBRID_ED448_DIL5: u8 = 253;

/// Ed448 public key size (bytes).
const ED448_PK_LEN: usize = 57;
/// Dilithium5 public key size (bytes).
const DIL5_PK_LEN: usize = 2592;

/// Hybrid DNSSEC keypair (Ed448 + Dilithium5).
pub struct HybridDNSSECKey {
    ed448_sk: SigningKey,
    ed448_vk: VerifyingKey,
    dil5_pk: dilithium5::PublicKey,
    dil5_sk: dilithium5::SecretKey,
}

impl HybridDNSSECKey {
    /// Generate a new hybrid keypair.
    fn generate() -> Self {
        let ed448_sk = SigningKey::generate(&mut rand::rngs::OsRng);
        let ed448_vk = ed448_sk.verifying_key();
        let (dil5_pk, dil5_sk) = dilithium5::keypair();
        Self { ed448_sk, ed448_vk, dil5_pk, dil5_sk }
    }

    /// Serialize the public key: [Ed448 vk (57)] [Dilithium5 pk (2592)].
    fn public_key_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(ED448_PK_LEN + DIL5_PK_LEN);
        bytes.extend_from_slice(&self.ed448_vk.to_bytes());
        bytes.extend_from_slice(self.dil5_pk.as_bytes());
        bytes
    }

    /// Sign data with both Ed448 and Dilithium5.
    /// Returns concatenated signature: [Ed448 sig (114)] [Dilithium5 sig (4627)].
    fn sign(&self, data: &[u8]) -> Vec<u8> {
        let ed448_sig = self.ed448_sk.sign_raw(data);
        let dil5_sig = dilithium5::detached_sign(data, &self.dil5_sk);

        let mut bytes = Vec::with_capacity(114 + 4627);
        bytes.extend_from_slice(&ed448_sig.to_bytes());
        bytes.extend_from_slice(dil5_sig.as_bytes());
        bytes
    }
}

/// Verify a hybrid DNSSEC signature. Both Ed448 AND Dilithium5 must verify.
fn hybrid_verify(data: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool> {
    if public_key.len() != ED448_PK_LEN + DIL5_PK_LEN {
        return Err(AppError::DNSSEC(format!(
            "Invalid hybrid public key length: expected {}, got {}",
            ED448_PK_LEN + DIL5_PK_LEN,
            public_key.len()
        )));
    }
    if signature.len() != 114 + 4627 {
        return Err(AppError::DNSSEC(format!(
            "Invalid hybrid signature length: expected {}, got {}",
            114 + 4627,
            signature.len()
        )));
    }

    // Split public key
    let mut ed448_pk_bytes = [0u8; ED448_PK_LEN];
    ed448_pk_bytes.copy_from_slice(&public_key[..ED448_PK_LEN]);
    let ed448_vk = VerifyingKey::from_bytes(&ed448_pk_bytes)
        .map_err(|_| AppError::DNSSEC("Invalid Ed448 public key".into()))?;

    let dil5_pk = dilithium5::PublicKey::from_bytes(&public_key[ED448_PK_LEN..])
        .map_err(|_| AppError::DNSSEC("Invalid Dilithium5 public key".into()))?;

    // Split signature
    let mut ed448_sig_bytes = [0u8; 114];
    ed448_sig_bytes.copy_from_slice(&signature[..114]);
    let ed448_sig = ed448_goldilocks_plus::Signature::from_bytes(&ed448_sig_bytes)
        .map_err(|_| AppError::DNSSEC("Invalid Ed448 signature".into()))?;

    let dil5_sig = dilithium5::DetachedSignature::from_bytes(&signature[114..])
        .map_err(|_| AppError::DNSSEC("Invalid Dilithium5 signature".into()))?;

    // Verify Ed448
    if ed448_vk.verify_raw(&ed448_sig, data).is_err() {
        return Ok(false);
    }

    // Verify Dilithium5
    if dilithium5::verify_detached_signature(&dil5_sig, data, &dil5_pk).is_err() {
        return Ok(false);
    }

    Ok(true)
}

/// DNSSEC key manager — Hybrid Ed448 + Dilithium5 (CPR-005).
pub struct DNSSECManager {
    // Configuration placeholder
}

impl DNSSECManager {
    /// Create a new DNSSEC manager.
    pub fn new() -> Self {
        Self {}
    }

    /// Generate DNSSEC keys for a zone.
    ///
    /// Returns (KSK public key base64, ZSK public key base64, DS record).
    pub fn generate_keys(&self, zone: &str) -> Result<(String, String, String)> {
        let b64 = &base64::engine::general_purpose::STANDARD;

        // Generate KSK (Key Signing Key) — hybrid Ed448 + Dilithium5
        let ksk = HybridDNSSECKey::generate();
        let ksk_public = b64.encode(ksk.public_key_bytes());

        // Generate ZSK (Zone Signing Key) — hybrid Ed448 + Dilithium5
        let zsk = HybridDNSSECKey::generate();
        let zsk_public = b64.encode(zsk.public_key_bytes());

        // Generate DS record (Delegation Signer for parent zone)
        // Algorithm 253 = private-use hybrid Ed448+Dilithium5
        let ds_record = self.generate_ds_record(zone, &ksk_public)?;

        Ok((ksk_public, zsk_public, ds_record))
    }

    /// Generate DS record for parent zone.
    fn generate_ds_record(&self, zone: &str, ksk_public: &str) -> Result<String> {
        // DNSKEY record format: <zone> IN DNSKEY <flags> <protocol> <algorithm> <public key>
        // Flags: 257 for KSK (bit 0 = Zone Key, bit 15 = Secure Entry Point)
        // Protocol: always 3
        // Algorithm: 253 for hybrid Ed448+Dilithium5
        let dnskey = format!(
            "{} IN DNSKEY 257 3 {} {}",
            zone, ALGORITHM_HYBRID_ED448_DIL5, ksk_public
        );

        // BLAKE3 digest per CPR-009 (replacing SHA-256)
        let digest_value = blake3::hash(dnskey.as_bytes());
        let digest_hex = hex::encode(digest_value.as_bytes());

        let key_tag = self.calculate_key_tag(&dnskey);

        // DS record: <zone> IN DS <key tag> <algorithm> <digest type> <digest>
        // Digest type 253 = private-use BLAKE3 (matching algorithm private-use range)
        let ds_record = format!(
            "{} IN DS {} {} 253 {}",
            zone, key_tag, ALGORITHM_HYBRID_ED448_DIL5, digest_hex
        );

        Ok(ds_record)
    }

    /// Calculate DNSSEC key tag per RFC 4034 §B.1 (simplified).
    ///
    /// Uses BLAKE3 for internal hashing (CPR-009).
    fn calculate_key_tag(&self, dnskey: &str) -> u16 {
        let hash = blake3::hash(dnskey.as_bytes());
        let bytes = hash.as_bytes();
        u16::from_be_bytes([bytes[0], bytes[1]])
    }

    /// Sign a DNS record with hybrid Ed448 + Dilithium5.
    ///
    /// Takes record data and the ZSK hybrid key material (serialized).
    /// Returns base64-encoded hybrid signature.
    pub fn sign_record(&self, record_data: &str, zsk_key: &HybridDNSSECKey) -> Result<String> {
        let sig_bytes = zsk_key.sign(record_data.as_bytes());
        Ok(base64::engine::general_purpose::STANDARD.encode(&sig_bytes))
    }

    /// Verify a hybrid DNSSEC signature.
    ///
    /// Both Ed448 and Dilithium5 signatures must verify independently.
    pub fn verify_signature(
        &self,
        record_data: &str,
        rrsig_b64: &str,
        public_key_bytes: &[u8],
    ) -> Result<bool> {
        let sig_bytes = base64::engine::general_purpose::STANDARD
            .decode(rrsig_b64)
            .map_err(|e| AppError::DNSSEC(format!("Invalid base64 signature: {:?}", e)))?;

        hybrid_verify(record_data.as_bytes(), &sig_bytes, public_key_bytes)
    }
}

impl Default for DNSSECManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keys() {
        let manager = DNSSECManager::new();
        let result = manager.generate_keys("example.com");
        assert!(result.is_ok());

        let (ksk, zsk, ds) = result.expect("generate_keys should succeed");
        assert!(!ksk.is_empty());
        assert!(!zsk.is_empty());
        assert!(ds.contains("example.com"));
        assert!(ds.contains("IN DS"));
        assert!(ds.contains("253")); // algorithm 253
    }

    #[test]
    fn test_key_tag_calculation() {
        let manager = DNSSECManager::new();
        let dnskey = "example.com IN DNSKEY 257 3 253 YWJjZGVm";
        let tag = manager.calculate_key_tag(dnskey);
        assert!(tag > 0);
    }

    #[test]
    fn test_sign_and_verify_roundtrip() {
        let manager = DNSSECManager::new();
        let zsk = HybridDNSSECKey::generate();
        let public_key = zsk.public_key_bytes();

        let record = "example.com. 3600 IN A 192.0.2.1";
        let signature = manager.sign_record(record, &zsk)
            .expect("signing should succeed");

        let verified = manager.verify_signature(record, &signature, &public_key)
            .expect("verification should succeed");
        assert!(verified);
    }

    #[test]
    fn test_verify_rejects_wrong_data() {
        let manager = DNSSECManager::new();
        let zsk = HybridDNSSECKey::generate();
        let public_key = zsk.public_key_bytes();

        let signature = manager.sign_record("original record", &zsk)
            .expect("signing should succeed");

        let verified = manager.verify_signature("tampered record", &signature, &public_key)
            .expect("verification call should succeed");
        assert!(!verified);
    }

    #[test]
    fn test_verify_rejects_wrong_key() {
        let manager = DNSSECManager::new();
        let zsk1 = HybridDNSSECKey::generate();
        let zsk2 = HybridDNSSECKey::generate();

        let record = "example.com. 3600 IN A 192.0.2.1";
        let signature = manager.sign_record(record, &zsk1)
            .expect("signing should succeed");

        let verified = manager.verify_signature(record, &signature, &zsk2.public_key_bytes())
            .expect("verification call should succeed");
        assert!(!verified);
    }
}
