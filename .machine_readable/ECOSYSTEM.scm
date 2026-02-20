;; SPDX-License-Identifier: PMPL-1.0-or-later
;; ECOSYSTEM.scm - Ecosystem position for civic-connect
;; Media-Type: application/vnd.ecosystem+scm

(ecosystem
  (version "1.0")
  (name "civic-connect")
  (type "platform")
  (purpose "Privacy-first civic engagement platform combining government API integration, oblivious DNS, and provenance-grade audit infrastructure")

  (position-in-ecosystem
    (category "civic-technology")
    (subcategory "government-transparency-and-engagement")
    (unique-value
      ("Post-quantum cryptographic privacy (Kyber-1024, Dilithium5, SPHINCS+)")
      ("Zero-knowledge DNS resolution via Oblivious DNS")
      ("Provenance-grade audit trail (SurrealDB graph + IPFS)")
      ("Consent-first architecture with Nickel policy contracts")
      ("Hybrid classical+PQ signature scheme (Ed448+Dilithium5)")))

  (related-projects
    (project
      (name "indieweb2")
      (relationship "core-component")
      (description "Consent-first bastion with oDNS, GraphQL DNS, SurrealDB provenance"))
    (project
      (name "odns-rs")
      (relationship "embedded-component")
      (description "Oblivious DNS proxy/resolver with post-quantum crypto"))
    (project
      (name "stapeln")
      (relationship "infrastructure-dependency")
      (description "Container toolchain: cerro-torre, selur, vordr, selur-compose"))
    (project
      (name "hypatia")
      (relationship "ci-integration")
      (description "Neurosymbolic CI/CD intelligence scanning"))
    (project
      (name "gitbot-fleet")
      (relationship "automation")
      (description "Bot orchestration for automated fixes"))
    (project
      (name "panic-attacker")
      (relationship "security-tooling")
      (description "Vulnerability scanning (panic-attack assail)"))
    (project
      (name "echidna")
      (relationship "security-tooling")
      (description "Proof verification and security proofing"))
    (project
      (name "verisimdb")
      (relationship "data-pipeline")
      (description "Vulnerability similarity database for scan results"))
    (project
      (name "cerro-torre")
      (relationship "build-tool")
      (description "Container image building and Ed25519 signing")))

  (technology-stack
    (languages
      (primary
        (lang "Rust"
          (purpose "Core services, crypto, DNS, API")
          (components "odns-rs, graphql-dns-api, webmention-limiter"))
        (lang "ReScript"
          (purpose "Policy gate, UI components")
          (components "policy-gate with 9 validators"))
        (lang "Deno"
          (purpose "Runtime, consent API, signing, publishing")
          (components "consent-api, static server, crypto utilities")))
      (supporting
        (lang "Nickel" (purpose "Policy contracts and validation"))
        (lang "Guile Scheme" (purpose "Machine-readable metadata (.scm files)"))
        (lang "Solidity" (purpose "Smart contracts (Ethereum/Polygon)"))
        (lang "Motoko" (purpose "Internet Computer canisters"))))

    (infrastructure
      (containerization "Podman + stapeln toolchain")
      (base-images "cgr.dev/chainguard/wolfi-base:latest")
      (orchestration "selur-compose (compose.toml)")
      (runtime "vordr (formally verified container execution)")
      (signing "cerro-torre sign (Ed25519, .ctp bundles)")
      (sealing "selur seal (zero-copy IPC bridge)"))

    (databases
      (primary
        (engine "SurrealDB")
        (mode "kv-mem / rocksdb")
        (purpose "Provenance graph: dns_records, dnssec_zones, blockchain_prov"))
      (content-storage
        (engine "IPFS")
        (purpose "Content-addressed storage and snapshot rehydration")))

    (cryptography
      (post-quantum
        (kem "Kyber-1024 / ML-KEM-1024 (FIPS 203)")
        (signature "Dilithium5 / ML-DSA-87 (FIPS 204)")
        (fallback "SPHINCS+ / SLH-DSA (CPR-012)"))
      (classical
        (signature "Ed448-Goldilocks")
        (symmetric "XChaCha20-Poly1305")
        (kdf "HKDF-SHA3-512")
        (hash "BLAKE3, SHAKE3-512")
        (password "Argon2id (512MiB/8it/4ln)")
        (rng "ChaCha20-DRBG / OsRng"))
      (hybrid "Ed448 + Dilithium5 dual-sign (CPR-005)"))

    (blockchain
      (providers "Ethereum, Polygon, Internet Computer")
      (purpose "Provenance anchoring and decentralized verification"))

    (dns
      (protocol "Oblivious DNS over TLS (port 853 / 8853)")
      (api "GraphQL DNS API (port 8443, Rust/Axum)")
      (dnssec "Hybrid Ed448+Dilithium5 signing")
      (upstream "UDP IPv6 [2606:4700::]")))

  (what-this-is
    ("A privacy-first civic engagement platform")
    ("Government API integration for transparency")
    ("Post-quantum secure DNS infrastructure")
    ("Provenance-grade audit trail for civic data")
    ("Consent-aware data processing framework")
    ("Decentralized verification via blockchain anchoring"))

  (what-this-is-not
    ("Not a social media platform")
    ("Not a general-purpose DNS server")
    ("Not a blockchain project (blockchain is one component)")
    ("Not cloud-dependent (self-hosted by design)")))
