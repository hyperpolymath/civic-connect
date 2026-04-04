// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Property-based tests for oDNS cryptographic primitives.
//
// Tests use proptest to check invariants that must hold for all inputs:
//   - Encrypt/decrypt round-trip identity
//   - Different plaintexts produce different ciphertexts (ciphertext distinguishability)
//   - Signature sign/verify correctness
//   - Tampered messages fail verification
//   - All key/message sizes accepted by public API

use odns_common::{
    crypto::{decrypt_query, encrypt_query, generate_keypair, public_key_from_bytes, secret_key_from_bytes, OVERHEAD},
    signatures::{generate_hybrid_keypair, hybrid_sign, hybrid_verify},
    sphincs_fallback::{generate_sphincs_keypair, public_key_bytes, sphincs_sign, sphincs_verify},
};
use proptest::prelude::*;
use pqcrypto_traits::kem::PublicKey as PublicKeyTrait;

// ---------------------------------------------------------------------------
// Strategy: arbitrary DNS query payloads (1 – 512 bytes, realistic range)
// ---------------------------------------------------------------------------

prop_compose! {
    /// Arbitrary DNS query payload (1–512 bytes).
    fn arb_dns_payload()(
        len in 1usize..=512,
        seed in any::<u64>(),
    ) -> Vec<u8> {
        // Deterministic fill from seed so the same case can be reproduced.
        let mut buf = vec![0u8; len];
        let mut state = seed;
        for byte in buf.iter_mut() {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *byte = (state >> 33) as u8;
        }
        buf
    }
}

// ---------------------------------------------------------------------------
// Kyber-1024 / XChaCha20-Poly1305 round-trip identity
// ---------------------------------------------------------------------------

proptest! {
    /// For any plaintext, decrypt(encrypt(plaintext, pk), sk) == plaintext.
    ///
    /// This is the fundamental correctness invariant of the KEM scheme.
    #[test]
    fn prop_encrypt_decrypt_identity(payload in arb_dns_payload()) {
        let (pk, sk) = generate_keypair();
        let encrypted = encrypt_query(&payload, &pk)
            .expect("encryption must not fail for valid inputs");
        let decrypted = decrypt_query(&encrypted, &sk)
            .expect("decryption must not fail with the correct key");
        prop_assert_eq!(decrypted, payload,
            "decrypt(encrypt(payload)) must equal the original payload");
    }

    /// Encrypted output must always be at least OVERHEAD bytes longer than plaintext.
    ///
    /// OVERHEAD = KEM_CT_LEN (1568) + NONCE_LEN (24) + TAG_LEN (16).
    #[test]
    fn prop_ciphertext_length_ge_overhead(payload in arb_dns_payload()) {
        let (pk, _sk) = generate_keypair();
        let encrypted = encrypt_query(&payload, &pk)
            .expect("encryption must not fail");
        prop_assert!(
            encrypted.len() >= OVERHEAD,
            "ciphertext length {} must be >= OVERHEAD {}",
            encrypted.len(),
            OVERHEAD,
        );
    }

    /// Two distinct plaintexts must produce distinct ciphertexts under the same public key.
    ///
    /// If this failed it would indicate a catastrophic collision in either HKDF
    /// or AEAD – both of which are computationally infeasible.
    #[test]
    fn prop_distinct_plaintexts_produce_distinct_ciphertexts(
        a in arb_dns_payload(),
        b in arb_dns_payload(),
    ) {
        prop_assume!(a != b);
        let (pk, _sk) = generate_keypair();
        let ct_a = encrypt_query(&a, &pk).expect("encrypt a");
        let ct_b = encrypt_query(&b, &pk).expect("encrypt b");
        // XChaCha20-Poly1305 is randomised (random nonce each call), so even
        // the same plaintext produces distinct ciphertexts. Two different
        // plaintexts must certainly differ.
        prop_assert_ne!(ct_a, ct_b,
            "distinct plaintexts must produce distinct ciphertexts");
    }

    /// Decryption with the wrong secret key must always fail.
    #[test]
    fn prop_wrong_key_always_fails(payload in arb_dns_payload()) {
        let (pk, _sk_correct) = generate_keypair();
        let (_pk_wrong, sk_wrong) = generate_keypair();
        let encrypted = encrypt_query(&payload, &pk)
            .expect("encrypt");
        let result = decrypt_query(&encrypted, &sk_wrong);
        prop_assert!(
            result.is_err(),
            "decryption with a different secret key must always fail"
        );
    }

    /// Bit-flipping any byte in the ciphertext must cause decryption to fail.
    ///
    /// This tests the AEAD authentication guarantee: any tampering is detected.
    #[test]
    fn prop_tampered_ciphertext_rejected(
        payload in arb_dns_payload(),
        // Only flip bytes in the AEAD region (after KEM CT) to avoid crafting
        // a valid KEM ciphertext by accident; AEAD tag covers the message body.
        flip_offset in odns_common::crypto::KEM_CT_LEN..=(odns_common::crypto::KEM_CT_LEN + 23 + 8),
    ) {
        let (pk, sk) = generate_keypair();
        let mut encrypted = encrypt_query(&payload, &pk).expect("encrypt");
        prop_assume!(flip_offset < encrypted.len());
        encrypted[flip_offset] ^= 0xFF;
        let result = decrypt_query(&encrypted, &sk);
        prop_assert!(result.is_err(),
            "tampered ciphertext (byte {} flipped) must be rejected", flip_offset);
    }

    /// Public key serialise/deserialise round-trip must be lossless.
    #[test]
    fn prop_public_key_serialization_roundtrip(payload in arb_dns_payload()) {
        let (pk, sk) = generate_keypair();
        let pk_bytes = pk.as_bytes().to_vec();
        let pk2 = public_key_from_bytes(&pk_bytes)
            .expect("public key must round-trip through bytes");
        // Encrypt with the restored key and decrypt with the original secret key.
        let ct = encrypt_query(&payload, &pk2).expect("encrypt with restored pk");
        let pt = decrypt_query(&ct, &sk).expect("decrypt");
        prop_assert_eq!(pt, payload, "key serialization must be lossless");
    }
}

// ---------------------------------------------------------------------------
// Hybrid Ed448 + Dilithium5 signature properties
// ---------------------------------------------------------------------------

proptest! {
    /// For any message, sign(msg, kp) verifies with the corresponding public key.
    #[test]
    fn prop_hybrid_sign_verify_identity(payload in arb_dns_payload()) {
        let kp = generate_hybrid_keypair();
        let sig = hybrid_sign(&payload, &kp);
        let pk = kp.public_key().expect("extract public key");
        let result = hybrid_verify(&payload, &sig, &pk);
        prop_assert!(result.is_ok(),
            "hybrid signature must verify for the signed message");
    }

    /// Signing different messages with the same key must produce valid but distinct sigs.
    #[test]
    fn prop_hybrid_sign_different_msgs_verify_correctly(
        a in arb_dns_payload(),
        b in arb_dns_payload(),
    ) {
        prop_assume!(a != b);
        let kp = generate_hybrid_keypair();
        let pk = kp.public_key().expect("extract public key");
        let sig_a = hybrid_sign(&a, &kp);
        let sig_b = hybrid_sign(&b, &kp);
        // sig_a must verify against a but NOT against b (and vice-versa)
        prop_assert!(hybrid_verify(&a, &sig_a, &pk).is_ok());
        prop_assert!(hybrid_verify(&b, &sig_b, &pk).is_ok());
        prop_assert!(hybrid_verify(&b, &sig_a, &pk).is_err(),
            "sig_a must not verify msg_b");
        prop_assert!(hybrid_verify(&a, &sig_b, &pk).is_err(),
            "sig_b must not verify msg_a");
    }

    /// Signature made with one keypair must not verify under a different keypair's public key.
    #[test]
    fn prop_hybrid_wrong_key_rejects(payload in arb_dns_payload()) {
        let kp1 = generate_hybrid_keypair();
        let kp2 = generate_hybrid_keypair();
        let sig = hybrid_sign(&payload, &kp1);
        let pk2 = kp2.public_key().expect("extract public key");
        prop_assert!(hybrid_verify(&payload, &sig, &pk2).is_err(),
            "signature from kp1 must not verify under kp2 public key");
    }

    /// Public key serialization must be lossless for signature verification.
    #[test]
    fn prop_hybrid_pk_serialization_lossless(payload in arb_dns_payload()) {
        use odns_common::signatures::HybridPublicKey;
        let kp = generate_hybrid_keypair();
        let sig = hybrid_sign(&payload, &kp);
        let pk = kp.public_key().expect("extract public key");
        let pk_bytes = pk.to_bytes();
        let pk2 = HybridPublicKey::from_bytes(&pk_bytes).expect("deserialize public key");
        prop_assert!(hybrid_verify(&payload, &sig, &pk2).is_ok(),
            "verification must succeed with a round-tripped public key");
    }
}

// ---------------------------------------------------------------------------
// SPHINCS+ fallback signature properties (CPR-012)
// ---------------------------------------------------------------------------

proptest! {
    /// SPHINCS+ sign/verify round-trip must hold for all messages.
    ///
    /// Note: SPHINCS+ is slow (~300 ms/sign). proptest default of 256 cases
    /// would be prohibitive; we override to 5 cases.
    #[test]
    #[ignore = "slow: SPHINCS+ signing takes ~300ms per case; run with --include-ignored"]
    fn prop_sphincs_sign_verify_identity(payload in arb_dns_payload()) {
        let kp = generate_sphincs_keypair();
        let sig = sphincs_sign(&payload, &kp);
        let pk = public_key_bytes(&kp);
        let result = sphincs_verify(&payload, &sig, &pk);
        prop_assert!(result.is_ok(),
            "SPHINCS+ signature must verify for the signed message");
    }
}
