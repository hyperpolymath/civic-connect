;; SPDX-License-Identifier: PMPL-1.0-or-later
(meta
  (metadata
    (version "0.3.0")
    (last-updated "2026-02-14"))
  (project-info
    (type multi-service)
    (languages (rust rescript deno nickel guile-scheme))
    (deprecated-languages (go))
    (license "PMPL-1.0-or-later")
    (author "Jonathan D.A. Jewell <jonathan.jewell@open.ac.uk>"))
  (architecture-decisions
    (adr "0001" "Multi-chain blockchain architecture" "accepted")
    (adr "0002" "SurrealDB for DNS record graph storage" "accepted")
    (adr "0003" "oDNS Go-to-Rust rewrite with PQ crypto" "accepted"))
  (security
    (crypto-policy "CRYPTO-POLICY.adoc")
    (crypto-policy-machine-readable "CRYPTO-POLICY.scm")
    (signing "Ed448+Dilithium5 hybrid (CPR-005) — implemented in odns-rs/common/src/signatures.rs")
    (hashing "BLAKE3 for content, SHAKE3-512 for integrity")
    (encryption "XChaCha20-Poly1305 — implemented in odns-rs/common/src/crypto.rs")
    (key-exchange "Kyber-1024 / ML-KEM-1024 — implemented in odns-rs/common/src/crypto.rs"))
  (fleet-enrollment
    (gitbot-fleet "enrolled" "shared-context/findings/indieweb2-bastion/")
    (echidna "configured" ".echidnabot.toml")
    (git-private-farm "enrolled" "farm-manifest.json")))
