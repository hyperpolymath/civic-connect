// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <jonathan.jewell@open.ac.uk>
//
// Post-quantum KEM encryption for oDNS query transport.
//
// Wire format (proxy → resolver):
//   [1568 bytes: Kyber-1024 KEM ciphertext]
//   [24 bytes:   XChaCha20-Poly1305 nonce]
//   [N bytes:    AEAD ciphertext (DNS query + 16-byte Poly1305 tag)]

use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::XChaCha20Poly1305;
use hkdf::Hkdf;
use pqcrypto_kyber::kyber1024;
use pqcrypto_traits::kem::{
    Ciphertext as CiphertextTrait, PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait,
    SharedSecret as SharedSecretTrait,
};
use rand::RngCore;
use sha3::Sha3_512;

pub type PublicKey = kyber1024::PublicKey;
pub type SecretKey = kyber1024::SecretKey;

/// Kyber-1024 KEM ciphertext size (bytes).
pub const KEM_CT_LEN: usize = 1568;
/// XChaCha20-Poly1305 nonce size (bytes).
pub const NONCE_LEN: usize = 24;
/// Poly1305 authentication tag size (bytes).
pub const TAG_LEN: usize = 16;
/// Minimum encrypted message overhead: KEM ciphertext + nonce + tag.
pub const OVERHEAD: usize = KEM_CT_LEN + NONCE_LEN + TAG_LEN;

/// HKDF info string — domain-separates oDNS query encryption keys.
const HKDF_INFO: &[u8] = b"odns-kyber1024-xchacha20-v1";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("message too short: need at least {min} bytes, got {got}")]
    MessageTooShort { min: usize, got: usize },

    #[error("invalid KEM ciphertext")]
    InvalidCiphertext,

    #[error("encryption failed")]
    EncryptionFailed,

    #[error("decryption failed: authentication failure or data corruption")]
    DecryptionFailed,

    #[error("HKDF key derivation failed")]
    KeyDerivationFailed,

    #[error("invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("invalid secret key: {0}")]
    InvalidSecretKey(String),
}

/// Generate a new Kyber-1024 keypair.
///
/// Returns `(public_key, secret_key)`. The public key (1568 bytes) is
/// distributed to the proxy; the secret key (3168 bytes) stays on the resolver.
pub fn generate_keypair() -> (PublicKey, SecretKey) {
    kyber1024::keypair()
}

/// Encrypt a DNS query for the resolver.
///
/// Performs Kyber-1024 KEM encapsulation to establish a shared secret,
/// derives a symmetric key via HKDF-SHA3-512, then encrypts the query
/// with XChaCha20-Poly1305.
///
/// Returns the wire-format blob: `KEM_ciphertext || nonce || AEAD_ciphertext`.
pub fn encrypt_query(query: &[u8], public_key: &PublicKey) -> Result<Vec<u8>, Error> {
    // 1. Kyber-1024 KEM encapsulation → shared secret + KEM ciphertext
    let (shared_secret, kem_ct) = kyber1024::encapsulate(public_key);

    // 2. Derive 32-byte symmetric key via HKDF-SHA3-512
    let sym_key = derive_symmetric_key(shared_secret.as_bytes())?;

    // 3. Generate 24-byte random nonce for XChaCha20-Poly1305
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = GenericArray::from_slice(&nonce_bytes);

    // 4. XChaCha20-Poly1305 encrypt
    let cipher = XChaCha20Poly1305::new(GenericArray::from_slice(&sym_key));
    let aead_ct = cipher.encrypt(nonce, query).map_err(|_| Error::EncryptionFailed)?;

    // 5. Assemble wire format
    let mut wire = Vec::with_capacity(kem_ct.as_bytes().len() + NONCE_LEN + aead_ct.len());
    wire.extend_from_slice(kem_ct.as_bytes());
    wire.extend_from_slice(&nonce_bytes);
    wire.extend_from_slice(&aead_ct);

    Ok(wire)
}

/// Decrypt a DNS query from the proxy.
///
/// Splits the wire-format blob, performs Kyber-1024 KEM decapsulation,
/// derives the symmetric key, and decrypts with XChaCha20-Poly1305.
pub fn decrypt_query(encrypted: &[u8], secret_key: &SecretKey) -> Result<Vec<u8>, Error> {
    if encrypted.len() < OVERHEAD {
        return Err(Error::MessageTooShort {
            min: OVERHEAD,
            got: encrypted.len(),
        });
    }

    // 1. Split wire format
    let kem_ct_bytes = &encrypted[..KEM_CT_LEN];
    let nonce_bytes = &encrypted[KEM_CT_LEN..KEM_CT_LEN + NONCE_LEN];
    let aead_ct = &encrypted[KEM_CT_LEN + NONCE_LEN..];

    // 2. Reconstruct KEM ciphertext and decapsulate
    let kem_ct = kyber1024::Ciphertext::from_bytes(kem_ct_bytes)
        .map_err(|_| Error::InvalidCiphertext)?;
    let shared_secret = kyber1024::decapsulate(&kem_ct, secret_key);

    // 3. Derive symmetric key
    let sym_key = derive_symmetric_key(shared_secret.as_bytes())?;

    // 4. XChaCha20-Poly1305 decrypt
    let cipher = XChaCha20Poly1305::new(GenericArray::from_slice(&sym_key));
    let nonce = GenericArray::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, aead_ct)
        .map_err(|_| Error::DecryptionFailed)?;

    Ok(plaintext)
}

/// Deserialize a public key from raw bytes (1568 bytes for Kyber-1024).
pub fn public_key_from_bytes(bytes: &[u8]) -> Result<PublicKey, Error> {
    kyber1024::PublicKey::from_bytes(bytes)
        .map_err(|e| Error::InvalidPublicKey(format!("{:?}", e)))
}

/// Deserialize a secret key from raw bytes (3168 bytes for Kyber-1024).
pub fn secret_key_from_bytes(bytes: &[u8]) -> Result<SecretKey, Error> {
    kyber1024::SecretKey::from_bytes(bytes)
        .map_err(|e| Error::InvalidSecretKey(format!("{:?}", e)))
}

/// Derive a 32-byte symmetric key from a Kyber shared secret using HKDF-SHA3-512.
fn derive_symmetric_key(shared_secret: &[u8]) -> Result<[u8; 32], Error> {
    let hk = Hkdf::<Sha3_512>::new(None, shared_secret);
    let mut okm = [0u8; 32];
    hk.expand(HKDF_INFO, &mut okm)
        .map_err(|_| Error::KeyDerivationFailed)?;
    Ok(okm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let (pk, sk) = generate_keypair();
        let plaintext = b"example.com A query";

        let encrypted = encrypt_query(plaintext, &pk).unwrap();
        assert!(encrypted.len() >= OVERHEAD);

        let decrypted = decrypt_query(&encrypted, &sk).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_wrong_key_fails() {
        let (pk, _sk) = generate_keypair();
        let (_pk2, sk2) = generate_keypair();

        let encrypted = encrypt_query(b"secret query", &pk).unwrap();
        let result = decrypt_query(&encrypted, &sk2);
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_truncated_fails() {
        let result = decrypt_query(&[0u8; 10], &generate_keypair().1);
        assert!(matches!(result, Err(Error::MessageTooShort { .. })));
    }

    #[test]
    fn key_serialization_roundtrip() {
        let (pk, sk) = generate_keypair();

        let pk2 = public_key_from_bytes(pk.as_bytes()).unwrap();
        let sk2 = secret_key_from_bytes(sk.as_bytes()).unwrap();

        let encrypted = encrypt_query(b"test", &pk2).unwrap();
        let decrypted = decrypt_query(&encrypted, &sk2).unwrap();
        assert_eq!(decrypted, b"test");
    }
}
