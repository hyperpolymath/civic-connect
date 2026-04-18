// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! Cryptographic operations
//!
//! Security-critical module for:
//! - Password hashing (Argon2)
//! - Digital signatures (ed25519)
//! - JWT token generation

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::RngCore;

use crate::error::{ApiError, Result};

/// Hash a password using Argon2id
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Password hashing failed: {}", e)))?;

    Ok(hash.to_string())
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Invalid password hash: {}", e)))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Generate a new ed25519 keypair
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// Sign a message with ed25519
pub fn sign_message(signing_key: &SigningKey, message: &[u8]) -> Signature {
    signing_key.sign(message)
}

/// Verify an ed25519 signature
pub fn verify_signature(
    verifying_key: &VerifyingKey,
    message: &[u8],
    signature: &Signature,
) -> bool {
    verifying_key.verify(message, signature).is_ok()
}

/// Generate a random nonce (32 bytes, hex encoded)
pub fn generate_nonce() -> String {
    let mut nonce = [0u8; 32];
    OsRng.fill_bytes(&mut nonce);
    hex::encode(nonce)
}

/// Hash email for zero-knowledge storage
pub fn hash_email(email: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(email.to_lowercase().as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "SecurePassword123!";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_ed25519_signatures() {
        let (signing_key, verifying_key) = generate_keypair();
        let message = b"Test message for signing";

        let signature = sign_message(&signing_key, message);
        assert!(verify_signature(&verifying_key, message, &signature));

        // Wrong message should fail
        assert!(!verify_signature(&verifying_key, b"Wrong message", &signature));
    }

    #[test]
    fn test_nonce_generation() {
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();

        // Should be 64 hex characters (32 bytes)
        assert_eq!(nonce1.len(), 64);
        assert_eq!(nonce2.len(), 64);

        // Should be unique
        assert_ne!(nonce1, nonce2);
    }

    #[test]
    fn test_email_hashing() {
        let email = "User@Example.COM";
        let hash1 = hash_email(email);
        let hash2 = hash_email("user@example.com");

        // Should be case-insensitive
        assert_eq!(hash1, hash2);

        // Should be 64 hex characters (SHA256)
        assert_eq!(hash1.len(), 64);
    }
}
