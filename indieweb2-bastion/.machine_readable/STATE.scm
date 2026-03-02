;; SPDX-License-Identifier: PMPL-1.0-or-later
(state
  (metadata
    (version "0.5.0")
    (last-updated "2026-02-14")
    (status active))
  (project-context
    (name "indieweb2-bastion")
    (purpose "Multi-chain blockchain IndieWeb platform with GraphQL DNS, DNSSEC, consent API, and policy enforcement")
    (completion-percentage 60))
  (components
    (component
      (name "graphql-dns-api")
      (language "rust")
      (status "partial")
      (notes "SQL injection fixed, CORS hardened, identity bypass fixed, BLAKE3 hashing, hybrid DNSSEC (Ed448+Dilithium5), kv-mem default, axum 0.8"))
    (component
      (name "odns-rs/proxy")
      (language "rust")
      (status "complete")
      (notes "Kyber-1024 KEM + HKDF-SHA3-512 + XChaCha20-Poly1305; replaces Go odns-proxy (ADR-0003)"))
    (component
      (name "odns-rs/resolver")
      (language "rust")
      (status "complete")
      (notes "Kyber-1024 decapsulation + upstream DNS resolution; replaces Go odns-resolver (ADR-0003)"))
    (component
      (name "odns-rs/signatures")
      (language "rust")
      (status "complete")
      (notes "Hybrid Ed448+Dilithium5 (CPR-005): 11 tests pass, sign/verify/serialize roundtrip"))
    (component
      (name "odns-rs/sphincs_fallback")
      (language "rust")
      (status "complete")
      (notes "SPHINCS+-SHA2-256s-simple (CPR-012/SLH-DSA): 5 tests pass, ~29KiB signatures"))
    (component
      (name "consent-api")
      (language "deno")
      (status "production")
      (notes "CORS hardened, content integrity hashing (SHA-256 interim → BLAKE3), PQ signing scaffold documented"))
    (component
      (name "webmention-rate-limiter")
      (language "rust")
      (status "partial")
      (notes "Rate limiting functional; no direct crypto usage"))
    (component
      (name "policy-gate")
      (language "rescript")
      (status "functional")
      (notes "9 validators, crypto compliance checks, Deno runtime, ES module output"))
    (component
      (name "crypto-policy")
      (language "scheme/nickel/asciidoc")
      (status "defined")
      (notes "CRYPTO-POLICY.scm + CRYPTO-POLICY.adoc + schema.ncl/policy.ncl"))
    (component
      (name "smart-contracts")
      (language "solidity/vyper")
      (status "functional")
      (notes "ERC-20 with events+approve+transferFrom; Registry.vy with events+ownership"))
    (component
      (name "container-stack")
      (language "containerfile/toml")
      (status "complete")
      (notes "stapeln/cerro-torre build, selur seal, vordr run, selur-compose orchestration"))
    (component
      (name "fleet-enrollment")
      (language "json/toml")
      (status "complete")
      (notes "gitbot-fleet findings, echidnabot.toml, git-private-farm manifest")))
  (scan-results
    (scan-date "2026-02-14")
    (tool "panic-attack assail")
    (total-findings 3)
    (finding "medium" "CommandInjection" "scripts/interactive_tidy.sh")
    (finding "medium" "PanicPath" "graphql-dns-api/tests/integration_test.rs")
    (finding "medium" "PanicPath" "odns-rs/common/src/crypto.rs"))
  (blockers-and-issues
    (issue "consent-api uses SHA-256 interim — needs BLAKE3 WASM + hybrid signing")
    (issue "QUIC/HTTP3/IPv6-only transport not started (CPR-010)")
    (issue "Formal verification not started (CPR-013)")
    (issue "WCAG 2.3 AAA not started (CPR-011)"))
  (critical-next-actions
    (action "Build consent-crypto Rust WASM crate for BLAKE3+hybrid signing in Deno")
    (action "Migrate to QUIC/HTTP3/IPv6-only transport (CPR-010)")
    (action "Begin Coq/Isabelle formal verification of crypto protocols (CPR-013)")
    (action "WCAG 2.3 AAA accessibility compliance (CPR-011)")))
