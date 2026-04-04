// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// E2E tests for the oDNS cryptographic pipeline.
//
// These tests exercise the full path a DNS query takes through the oDNS system:
//   DNS query construction → KEM encapsulation → AEAD encryption
//   → wire format assembly → wire format parsing → KEM decapsulation
//   → AEAD decryption → DNS message validation
//
// The proxy and resolver binaries require TLS certificates and TCP connections
// that we cannot set up in a unit test environment, so we test the
// cryptographic and framing layers end-to-end using in-memory buffers.
//
// "E2E" here means the complete data-path through every codec layer, not
// a network-level end-to-end test (which lives in integration/ instead).

use odns_common::{
    crypto::{decrypt_query, encrypt_query, generate_keypair, OVERHEAD},
    protocol::{read_framed, write_framed},
    signatures::{
        generate_hybrid_keypair, hybrid_sign, hybrid_verify, HybridPublicKey, HybridSignature,
        HYBRID_PK_LEN, HYBRID_SIG_LEN,
    },
    sphincs_fallback::{
        generate_sphincs_keypair, public_key_bytes, sphincs_sign, sphincs_verify, SPHINCS_SIG_LEN,
    },
};
use std::io::Cursor;

// ---------------------------------------------------------------------------
// Helper: build a minimal DNS query wire-format message (RFC 1035 §4.1)
// ---------------------------------------------------------------------------

/// Build a minimal DNS A-record query for `name`.
///
/// Layout: header (12 bytes) + QNAME + QTYPE (A=1) + QCLASS (IN=1).
/// This is a valid DNS wire-format message — hickory-proto can decode it.
fn build_dns_query(name: &str) -> Vec<u8> {
    let mut msg = Vec::new();

    // Header (12 bytes)
    msg.extend_from_slice(&[
        0x00, 0x01, // ID = 1
        0x01, 0x00, // Flags: standard query, recursion desired
        0x00, 0x01, // QDCOUNT = 1
        0x00, 0x00, // ANCOUNT = 0
        0x00, 0x00, // NSCOUNT = 0
        0x00, 0x00, // ARCOUNT = 0
    ]);

    // QNAME: split on '.', length-prefixed labels, NUL terminator
    for label in name.split('.') {
        let bytes = label.as_bytes();
        msg.push(bytes.len() as u8);
        msg.extend_from_slice(bytes);
    }
    msg.push(0x00); // Root label

    // QTYPE = A (1), QCLASS = IN (1)
    msg.extend_from_slice(&[0x00, 0x01, 0x00, 0x01]);

    msg
}

// ---------------------------------------------------------------------------
// Full pipeline: DNS query → KEM encrypt → framing → framing parse → KEM decrypt
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_odns_full_pipeline_roundtrip() {
    // ---- Resolver side: generate keypair ----
    let (resolver_pk, resolver_sk) = generate_keypair();

    // ---- Proxy side: construct DNS query ----
    let query_bytes = build_dns_query("example.com");
    assert!(!query_bytes.is_empty(), "DNS query must be non-empty");

    // ---- Proxy side: KEM-encrypt the DNS query ----
    let encrypted = encrypt_query(&query_bytes, &resolver_pk)
        .expect("proxy: KEM encryption must succeed");
    assert!(
        encrypted.len() >= OVERHEAD,
        "encrypted blob must be at least OVERHEAD bytes"
    );

    // ---- Proxy side: frame the encrypted query for TCP transport ----
    let mut wire_buffer = Vec::new();
    write_framed(&mut wire_buffer, &encrypted)
        .await
        .expect("proxy: framing must succeed");
    assert!(wire_buffer.len() > 2, "framed buffer must contain length prefix");

    // ---- Network boundary: simulate transit over TLS ----
    // (In production this traverses a TcpStream; here we use a Cursor.)
    let mut cursor = Cursor::new(wire_buffer);

    // ---- Resolver side: deframe the encrypted query ----
    let received_encrypted = read_framed(&mut cursor)
        .await
        .expect("resolver: deframing must succeed");
    assert_eq!(
        received_encrypted, encrypted,
        "deframed bytes must match what the proxy sent"
    );

    // ---- Resolver side: KEM-decrypt the DNS query ----
    let decrypted = decrypt_query(&received_encrypted, &resolver_sk)
        .expect("resolver: KEM decryption must succeed");

    // ---- Verify the decrypted payload matches the original DNS query ----
    assert_eq!(
        decrypted, query_bytes,
        "decrypted payload must exactly match the original DNS query"
    );
}

// ---------------------------------------------------------------------------
// Pipeline with hybrid signature wrapping (CPR-005 proof-of-origin)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_signed_query_pipeline() {
    // Resolver key (for encryption) and proxy signing key (for authenticity)
    let (resolver_pk, resolver_sk) = generate_keypair();
    let proxy_kp = generate_hybrid_keypair();
    let proxy_pk = proxy_kp.public_key().expect("extract proxy public key");

    let query_bytes = build_dns_query("private.example.internal");

    // Proxy: sign the plaintext DNS query before encrypting.
    let sig = hybrid_sign(&query_bytes, &proxy_kp);
    let sig_bytes = sig.to_bytes();
    assert_eq!(
        sig_bytes.len(),
        HYBRID_SIG_LEN,
        "hybrid signature must be HYBRID_SIG_LEN bytes"
    );

    // Proxy: assemble signed payload = sig || DNS query
    let mut signed_payload = Vec::with_capacity(HYBRID_SIG_LEN + query_bytes.len());
    signed_payload.extend_from_slice(&sig_bytes);
    signed_payload.extend_from_slice(&query_bytes);

    // Proxy: encrypt the signed payload
    let encrypted = encrypt_query(&signed_payload, &resolver_pk)
        .expect("encrypt signed payload");

    // Resolver: decrypt
    let decrypted = decrypt_query(&encrypted, &resolver_sk)
        .expect("decrypt signed payload");
    assert_eq!(decrypted, signed_payload);

    // Resolver: split sig || body
    assert!(
        decrypted.len() >= HYBRID_SIG_LEN,
        "decrypted payload must be at least HYBRID_SIG_LEN bytes"
    );
    let recovered_sig = HybridSignature::from_bytes(&decrypted[..HYBRID_SIG_LEN])
        .expect("deserialize hybrid signature");
    let recovered_body = &decrypted[HYBRID_SIG_LEN..];

    // Resolver: verify signature
    hybrid_verify(recovered_body, &recovered_sig, &proxy_pk)
        .expect("signature verification must succeed on the recovered body");

    assert_eq!(
        recovered_body, query_bytes.as_slice(),
        "recovered DNS query must equal the original"
    );
}

// ---------------------------------------------------------------------------
// Multiple sequential queries (simulates an ODNS session)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_multiple_queries_same_keypair() {
    let (pk, sk) = generate_keypair();

    let queries = vec![
        build_dns_query("example.com"),
        build_dns_query("mail.example.org"),
        build_dns_query("cdn.example.net"),
        build_dns_query("api.example.io"),
        build_dns_query("static.example.co.uk"),
    ];

    for (i, query) in queries.iter().enumerate() {
        // Each query uses a fresh KEM encapsulation → fresh shared secret → fresh AEAD key.
        let encrypted = encrypt_query(query, &pk)
            .unwrap_or_else(|e| panic!("query {i}: encryption failed: {e}"));
        let decrypted = decrypt_query(&encrypted, &sk)
            .unwrap_or_else(|e| panic!("query {i}: decryption failed: {e}"));
        assert_eq!(
            decrypted, *query,
            "query {i}: decrypted payload must match original"
        );
    }
}

// ---------------------------------------------------------------------------
// Framing protocol edge cases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_framing_rejects_zero_length() {
    // Zero-length frame (length prefix = 0x0000) is invalid per protocol.rs.
    let wire = [0x00u8, 0x00];
    let mut cursor = Cursor::new(wire.to_vec());
    let result = read_framed(&mut cursor).await;
    assert!(
        result.is_err(),
        "zero-length framed message must be rejected"
    );
}

#[tokio::test]
async fn e2e_framing_rejects_truncated_payload() {
    // Length prefix says 100 bytes but only 10 are available.
    let mut wire = Vec::new();
    wire.extend_from_slice(&100u16.to_be_bytes()); // claim 100 bytes
    wire.extend(std::iter::repeat(0xABu8).take(10)); // only 10 bytes follow
    let mut cursor = Cursor::new(wire);
    let result = read_framed(&mut cursor).await;
    assert!(
        result.is_err(),
        "truncated payload must cause a framing error"
    );
}

// ---------------------------------------------------------------------------
// SPHINCS+ fallback pipeline (CPR-012)
// ---------------------------------------------------------------------------

/// Verify that the SPHINCS+ fallback can be activated and produces valid sigs.
///
/// This test is marked `ignore` because SPHINCS+ signing is very slow (~300ms).
/// Run with `cargo test -- --include-ignored` to exercise it.
#[test]
#[ignore = "slow: SPHINCS+ signing takes ~300ms; run with --include-ignored"]
fn e2e_sphincs_fallback_pipeline() {
    let resolver_kp = generate_sphincs_keypair();
    let pk_bytes = public_key_bytes(&resolver_kp);
    assert_eq!(
        pk_bytes.len(),
        odns_common::sphincs_fallback::SPHINCS_PK_LEN,
        "SPHINCS+ public key must be SPHINCS_PK_LEN bytes"
    );

    let payload = build_dns_query("fallback.example.internal");

    // Sign
    let sig = sphincs_sign(&payload, &resolver_kp);
    assert_eq!(
        sig.len(),
        SPHINCS_SIG_LEN,
        "SPHINCS+ signature must be exactly SPHINCS_SIG_LEN bytes"
    );

    // Verify
    sphincs_verify(&payload, &sig, &pk_bytes)
        .expect("SPHINCS+ signature verification must succeed");

    // Tampered message must fail
    let mut tampered = payload.clone();
    tampered[0] ^= 0xFF;
    let result = sphincs_verify(&tampered, &sig, &pk_bytes);
    assert!(
        result.is_err(),
        "SPHINCS+ verification must fail on tampered message"
    );
}
