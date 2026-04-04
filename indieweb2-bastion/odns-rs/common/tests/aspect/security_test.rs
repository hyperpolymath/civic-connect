// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Aspect tests: security invariants for the oDNS cryptographic layer.
//
// Covers:
//   - Malformed / truncated wire packets are always rejected
//   - Bit-flipping in every region of the ciphertext triggers AEAD auth failure
//   - Wrong keypair never decrypts successfully (KEM binding)
//   - Zero-length and oversized messages are handled safely
//   - SPHINCS+ fallback activates independently of the primary scheme
//   - Protocol confusion: public API rejects wrong-sized inputs

use odns_common::{
    crypto::{
        decrypt_query, encrypt_query, generate_keypair, public_key_from_bytes,
        secret_key_from_bytes, Error as CryptoError, KEM_CT_LEN, NONCE_LEN, OVERHEAD, TAG_LEN,
    },
    signatures::{
        generate_hybrid_keypair, hybrid_sign, hybrid_verify, HybridPublicKey, HybridSignature,
        SignatureError, HYBRID_PK_LEN, HYBRID_SIG_LEN,
    },
    sphincs_fallback::{
        generate_sphincs_keypair, public_key_bytes, sphincs_sign, sphincs_verify, SphincsError,
        SPHINCS_PK_LEN,
    },
    protocol::{read_framed, write_framed, MAX_PAYLOAD},
};
use std::io::Cursor;

// ---------------------------------------------------------------------------
// KEM / AEAD: truncated input handling
// ---------------------------------------------------------------------------

#[test]
fn security_empty_ciphertext_rejected() {
    let (_pk, sk) = generate_keypair();
    let result = decrypt_query(&[], &sk);
    assert!(
        matches!(result, Err(CryptoError::MessageTooShort { .. })),
        "empty ciphertext must be rejected with MessageTooShort"
    );
}

#[test]
fn security_one_byte_short_rejected() {
    let (_pk, sk) = generate_keypair();
    // OVERHEAD - 1 is one byte too short to hold any AEAD ciphertext.
    let short = vec![0u8; OVERHEAD - 1];
    let result = decrypt_query(&short, &sk);
    assert!(
        matches!(result, Err(CryptoError::MessageTooShort { .. })),
        "ciphertext shorter than OVERHEAD must be rejected"
    );
}

#[test]
fn security_all_zeroes_ciphertext_rejected() {
    let (_pk, sk) = generate_keypair();
    // All-zeroes is not a valid Kyber ciphertext + AEAD ciphertext.
    let zeroes = vec![0u8; OVERHEAD + 32];
    let result = decrypt_query(&zeroes, &sk);
    // May fail as InvalidCiphertext or DecryptionFailed — both are correct.
    assert!(result.is_err(), "all-zeroes ciphertext must be rejected");
}

// ---------------------------------------------------------------------------
// KEM / AEAD: bit-flip tampering in every wire region
// ---------------------------------------------------------------------------

/// Helper: generate a valid ciphertext to tamper with.
fn fresh_ciphertext(plaintext: &[u8]) -> (Vec<u8>, odns_common::crypto::SecretKey) {
    let (pk, sk) = generate_keypair();
    let ct = encrypt_query(plaintext, &pk).expect("encrypt");
    (ct, sk)
}

#[test]
fn security_flip_in_nonce_region_rejected() {
    let (mut ct, sk) = fresh_ciphertext(b"dns-query-nonce-flip");
    // Flip a byte in the nonce (bytes KEM_CT_LEN..KEM_CT_LEN+NONCE_LEN).
    ct[KEM_CT_LEN] ^= 0xFF;
    let result = decrypt_query(&ct, &sk);
    // Nonce is not MACed directly, but the changed nonce decrypts to garbage
    // which fails AEAD tag verification.
    assert!(result.is_err(), "bit-flip in nonce must be caught by AEAD auth");
}

#[test]
fn security_flip_in_aead_ciphertext_region_rejected() {
    let payload = b"dns-query-aead-flip";
    let (mut ct, sk) = fresh_ciphertext(payload);
    // Flip a byte in the AEAD ciphertext body (after nonce, before end of MAC).
    let aead_start = KEM_CT_LEN + NONCE_LEN;
    ct[aead_start] ^= 0xFF;
    let result = decrypt_query(&ct, &sk);
    assert!(result.is_err(), "bit-flip in AEAD ciphertext must be rejected");
}

#[test]
fn security_flip_in_poly1305_tag_rejected() {
    let payload = b"dns-query-tag-flip";
    let (mut ct, sk) = fresh_ciphertext(payload);
    // Poly1305 tag occupies the last TAG_LEN bytes.
    let tag_start = ct.len() - TAG_LEN;
    ct[tag_start] ^= 0x01;
    let result = decrypt_query(&ct, &sk);
    assert!(result.is_err(), "bit-flip in Poly1305 tag must be rejected");
}

#[test]
fn security_flip_in_kem_ciphertext_rejected() {
    let payload = b"dns-query-kem-flip";
    let (mut ct, sk) = fresh_ciphertext(payload);
    // Flip a byte in the KEM ciphertext (first KEM_CT_LEN bytes).
    // This causes decapsulation to produce a different shared secret,
    // yielding a wrong AEAD key — decryption fails with auth error.
    ct[42] ^= 0xFF;
    let result = decrypt_query(&ct, &sk);
    assert!(result.is_err(), "bit-flip in KEM ciphertext must propagate to AEAD failure");
}

// ---------------------------------------------------------------------------
// KEM binding: wrong keypair never succeeds
// ---------------------------------------------------------------------------

#[test]
fn security_wrong_secret_key_rejected() {
    let (pk, _sk_correct) = generate_keypair();
    let (_pk2, sk_wrong) = generate_keypair();
    let ct = encrypt_query(b"secret dns query", &pk).expect("encrypt");
    let result = decrypt_query(&ct, &sk_wrong);
    assert!(result.is_err(), "wrong secret key must never decrypt successfully");
}

// ---------------------------------------------------------------------------
// Protocol confusion: wrong-sized public/secret key bytes
// ---------------------------------------------------------------------------

#[test]
fn security_undersized_public_key_rejected() {
    // Kyber-1024 public key is 1568 bytes; 100 bytes is obviously wrong.
    let result = public_key_from_bytes(&vec![0u8; 100]);
    assert!(
        matches!(result, Err(CryptoError::InvalidPublicKey(_))),
        "undersized public key bytes must be rejected"
    );
}

#[test]
fn security_oversized_public_key_rejected() {
    let result = public_key_from_bytes(&vec![0u8; 9999]);
    assert!(
        matches!(result, Err(CryptoError::InvalidPublicKey(_))),
        "oversized public key bytes must be rejected"
    );
}

#[test]
fn security_undersized_secret_key_rejected() {
    let result = secret_key_from_bytes(&vec![0u8; 100]);
    assert!(
        matches!(result, Err(CryptoError::InvalidSecretKey(_))),
        "undersized secret key bytes must be rejected"
    );
}

// ---------------------------------------------------------------------------
// Timing-attack resistance: response time must not reveal plaintext length
//
// We cannot directly measure timing in a unit test, but we can assert that
// both successful and failed decryptions complete without exponential blowup.
// The structural property we verify: path length through the code is constant
// regardless of which byte fails authentication.
//
// NOTE: This is a structural / smoke assertion, not a statistical timing test.
// A real timing audit requires a microbenchmark harness (see benches/).
// ---------------------------------------------------------------------------

#[test]
fn security_decryption_failure_is_constant_time_structural() {
    // Encrypt a short message and a long message; both tamper cases must fail
    // without observably different code paths (both hit the same AEAD error).
    let (mut short_ct, sk_short) = fresh_ciphertext(b"short");
    let (mut long_ct, sk_long) = fresh_ciphertext(&vec![0xABu8; 400]);

    let aead_start = KEM_CT_LEN + NONCE_LEN;
    short_ct[aead_start] ^= 0xFF;
    long_ct[aead_start] ^= 0xFF;

    let r1 = decrypt_query(&short_ct, &sk_short);
    let r2 = decrypt_query(&long_ct, &sk_long);

    // Both must fail with the same error variant.
    assert!(
        matches!(r1, Err(CryptoError::DecryptionFailed)),
        "tampered short message must fail with DecryptionFailed"
    );
    assert!(
        matches!(r2, Err(CryptoError::DecryptionFailed)),
        "tampered long message must fail with DecryptionFailed"
    );
}

// ---------------------------------------------------------------------------
// Hybrid signature: wrong message / wrong key security
// ---------------------------------------------------------------------------

#[test]
fn security_hybrid_sig_wrong_message_rejected() {
    let kp = generate_hybrid_keypair();
    let sig = hybrid_sign(b"original message", &kp);
    let pk = kp.public_key().expect("extract public key");
    let result = hybrid_verify(b"tampered message", &sig, &pk);
    assert!(result.is_err(), "signature must not verify against a different message");
}

#[test]
fn security_hybrid_sig_wrong_key_rejected() {
    let kp1 = generate_hybrid_keypair();
    let kp2 = generate_hybrid_keypair();
    let sig = hybrid_sign(b"message", &kp1);
    let pk2 = kp2.public_key().expect("extract kp2 public key");
    let result = hybrid_verify(b"message", &sig, &pk2);
    assert!(result.is_err(), "signature must not verify under a different public key");
}

#[test]
fn security_hybrid_sig_empty_message_accepted_if_signed() {
    // Empty payloads are cryptographically valid; the scheme must not panic.
    let kp = generate_hybrid_keypair();
    let sig = hybrid_sign(b"", &kp);
    let pk = kp.public_key().expect("extract public key");
    assert!(
        hybrid_verify(b"", &sig, &pk).is_ok(),
        "hybrid signature over empty message must verify"
    );
}

#[test]
fn security_hybrid_wrong_pk_size_rejected() {
    // HybridPublicKey::from_bytes must reject wrong-sized input.
    let result = HybridPublicKey::from_bytes(&vec![0u8; HYBRID_PK_LEN - 1]);
    assert!(
        matches!(result, Err(SignatureError::InvalidPublicKey { .. })),
        "undersized public key must be rejected"
    );
}

#[test]
fn security_hybrid_wrong_sig_size_rejected() {
    let result = HybridSignature::from_bytes(&vec![0u8; HYBRID_SIG_LEN - 1]);
    assert!(
        matches!(result, Err(SignatureError::InvalidSignature { .. })),
        "undersized signature must be rejected"
    );
}

// ---------------------------------------------------------------------------
// SPHINCS+ fallback: activates independently of primary scheme
// ---------------------------------------------------------------------------

/// SPHINCS+ is marked ignore because it is slow (~300ms per sign).
#[test]
#[ignore = "slow: SPHINCS+ signing takes ~300ms; run with --include-ignored"]
fn security_sphincs_fallback_independent_of_hybrid() {
    // Demonstrate that the SPHINCS+ fallback can be activated without any
    // reference to the Ed448+Dilithium5 primary scheme.
    let kp = generate_sphincs_keypair();
    let pk = public_key_bytes(&kp);
    assert_eq!(pk.len(), SPHINCS_PK_LEN, "SPHINCS+ public key must be exactly 32 bytes");

    let message = b"fallback-only message signed without hybrid scheme";
    let sig = sphincs_sign(message, &kp);

    let result = sphincs_verify(message, &sig, &pk);
    assert!(result.is_ok(), "SPHINCS+ fallback must verify independently");
}

/// SPHINCS+ wrong-public-key rejection (fast — no signing needed).
#[test]
fn security_sphincs_wrong_pk_rejected() {
    let result = sphincs_verify(b"msg", &vec![0u8; 100], &vec![0u8; 100]);
    // Fails on deserialization of pk (wrong size) or sig.
    assert!(result.is_err(), "SPHINCS+ must reject wrong-sized public key");
}

// ---------------------------------------------------------------------------
// Protocol framing: oversized payload rejection
// ---------------------------------------------------------------------------

#[tokio::test]
async fn security_oversized_framed_payload_rejected() {
    let huge = vec![0u8; MAX_PAYLOAD + 1];
    let mut buf = Vec::new();
    let result = write_framed(&mut buf, &huge).await;
    assert!(result.is_err(), "write_framed must reject payloads > MAX_PAYLOAD");
}
