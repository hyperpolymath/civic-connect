;; SPDX-License-Identifier: PMPL-1.0-or-later
;; Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <jonathan.jewell@open.ac.uk>
;;
;; Canonical cryptographic policy for indieweb2-bastion.
;; Machine-readable Guile Scheme format — all components MUST comply.
;; Human-readable rendering: CRYPTO-POLICY.adoc (repository root)

(crypto-policy
  (metadata
    (version "1.0.0")
    (date "2026-02-14")
    (status active)
    (canonical-doc "CRYPTO-POLICY.adoc"))

  ;; 1. Password Hashing
  (requirement
    (id "CPR-001")
    (name "password-hashing")
    (algorithm "Argon2id")
    (standard "RFC 9106")
    (parameters
      (memory-kib 524288)    ;; 512 MiB
      (iterations 8)
      (parallelism 4)
      (output-length 32))    ;; 256-bit
    (status required))

  ;; 2. General Hashing
  (requirement
    (id "CPR-002")
    (name "general-hashing")
    (algorithm "SHAKE3-512")
    (standard "FIPS 202 / SHA-3")
    (output-length 64)       ;; 512-bit
    (status required))

  ;; 3. Post-Quantum Signatures
  (requirement
    (id "CPR-003")
    (name "pq-signatures")
    (algorithm "Dilithium5-AES")
    (standard "FIPS 204 / ML-DSA-87")
    (status required))

  ;; 4. Post-Quantum Key Exchange
  (requirement
    (id "CPR-004")
    (name "pq-key-exchange")
    (algorithm "Kyber-1024")
    (standard "FIPS 203 / ML-KEM-1024")
    (status required))

  ;; 5. Classical Signatures (hybrid with PQ)
  (requirement
    (id "CPR-005")
    (name "classical-signatures")
    (algorithm "Ed448 + Dilithium5 hybrid")
    (standard "RFC 8032 / FIPS 204")
    (note "Ed448 classical paired with Dilithium5 post-quantum")
    (status required))

  ;; 6. Symmetric Encryption
  (requirement
    (id "CPR-006")
    (name "symmetric-encryption")
    (algorithm "XChaCha20-Poly1305")
    (standard "RFC 8439 extended nonce")
    (nonce-size 192)         ;; 192-bit nonce
    (status required))

  ;; 7. Key Derivation Function
  (requirement
    (id "CPR-007")
    (name "key-derivation")
    (algorithm "HKDF-SHAKE512")
    (standard "RFC 5869 with SHAKE3-512")
    (status required))

  ;; 8. Random Number Generation
  (requirement
    (id "CPR-008")
    (name "rng")
    (algorithm "ChaCha20-DRBG")
    (standard "NIST SP 800-90A compliant")
    (note "OS entropy source seeded")
    (status required))

  ;; 9. Database Hashing
  (requirement
    (id "CPR-009")
    (name "database-hashing")
    (algorithm "BLAKE3 + SHAKE3-512")
    (standard "BLAKE3 spec / FIPS 202")
    (note "BLAKE3 for fast content hashing, SHAKE3-512 for integrity proofs")
    (status required))

  ;; 10. Protocol Stack
  (requirement
    (id "CPR-010")
    (name "protocol-stack")
    (algorithm "QUIC + HTTP/3 + IPv6")
    (standard "RFC 9000 / RFC 9114 / RFC 8200")
    (note "IPv6-only, no IPv4 fallback")
    (status required))

  ;; 11. Accessibility
  (requirement
    (id "CPR-011")
    (name "accessibility")
    (standard "WCAG 2.3 AAA")
    (note "All cryptographic UIs must meet AAA accessibility")
    (status required))

  ;; 12. Fallback Signature
  (requirement
    (id "CPR-012")
    (name "fallback-signature")
    (algorithm "SPHINCS+")
    (standard "FIPS 205 / SLH-DSA")
    (note "Stateless hash-based fallback if lattice-based schemes compromised")
    (status required))

  ;; 13. Formal Verification
  (requirement
    (id "CPR-013")
    (name "formal-verification")
    (tools ("Coq" "Isabelle/HOL"))
    (note "All cryptographic protocol implementations must have formal proofs")
    (status required))

  ;; 14. TLS Configuration
  (requirement
    (id "CPR-014")
    (name "tls-configuration")
    (algorithm "TLS 1.3 only")
    (standard "RFC 8446")
    (ciphersuites ("TLS_AES_256_GCM_SHA384" "TLS_CHACHA20_POLY1305_SHA256"))
    (note "No TLS 1.2 or below")
    (status required))

  ;; 15. Certificate Pinning
  (requirement
    (id "CPR-015")
    (name "certificate-pinning")
    (algorithm "DANE-TLSA")
    (standard "RFC 6698 / RFC 7671")
    (note "DNSSEC-validated TLSA records for all endpoints")
    (status required))

  ;; 16. Blockchain Signatures
  (requirement
    (id "CPR-016")
    (name "blockchain-signatures")
    (algorithm "Ed448 + Dilithium5 hybrid")
    (standard "RFC 8032 / FIPS 204")
    (note "Blockchain-native ECDSA accepted where chain requires; off-chain uses hybrid")
    (status required))

  ;; Terminated Algorithms — MUST NOT be used in new code
  (terminated
    (algorithm (name "Ed25519") (reason "Replaced by Ed448+Dilithium5 hybrid") (deadline "2026-06-01"))
    (algorithm (name "SHA-1") (reason "Collision attacks demonstrated") (deadline "immediate"))
    (algorithm (name "MD5") (reason "Broken — collision attacks trivial") (deadline "immediate"))
    (algorithm (name "RSA") (reason "Quantum-vulnerable, replaced by ML-DSA-87") (deadline "2026-06-01"))
    (algorithm (name "X25519") (reason "Replaced by Kyber-1024 / ML-KEM-1024") (deadline "2026-06-01"))
    (algorithm (name "HKDF-SHA256") (reason "Replaced by HKDF-SHAKE512") (deadline "2026-06-01"))
    (algorithm (name "SHA-256") (reason "Replaced by SHAKE3-512 and BLAKE3") (deadline "2026-09-01"))
    (algorithm (name "ChaCha20-Poly1305") (reason "Replaced by XChaCha20-Poly1305 (extended nonce)") (deadline "2026-06-01"))
    (algorithm (name "HTTP/1.1") (reason "Replaced by HTTP/3 over QUIC") (deadline "2026-06-01"))
    (algorithm (name "IPv4") (reason "IPv6-only policy") (deadline "2026-09-01"))
    (algorithm (name "TLS 1.2") (reason "Replaced by TLS 1.3 only") (deadline "immediate")))

  ;; Migration Timeline
  (migration
    (phase (name "Phase 1: Policy Definition") (deadline "2026-02-14") (status complete))
    (phase (name "Phase 2: BLAKE3 + SHAKE3-512 for hashing") (deadline "2026-03-01") (status in-progress))
    (phase (name "Phase 3: XChaCha20-Poly1305 symmetric") (deadline "2026-04-01") (status planned))
    (phase (name "Phase 4: Ed448+Dilithium5 hybrid sigs") (deadline "2026-06-01") (status planned))
    (phase (name "Phase 5: Kyber-1024 key exchange") (deadline "2026-06-01") (status planned))
    (phase (name "Phase 6: QUIC/HTTP3/IPv6 transport") (deadline "2026-09-01") (status planned))
    (phase (name "Phase 7: Formal verification") (deadline "2026-12-01") (status planned))))
