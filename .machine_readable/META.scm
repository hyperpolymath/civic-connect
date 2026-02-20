;; SPDX-License-Identifier: PMPL-1.0-or-later
;; META.scm - Meta-level information for civic-connect
;; Media-Type: application/meta+scheme

(meta
  (architecture-decisions
    (adr
      (id "ADR-001")
      (title "Zero-knowledge DNS resolution via Oblivious DNS")
      (status "accepted")
      (date "2025-12")
      (context "Users querying government APIs should not have DNS queries visible to intermediaries or the server operator")
      (decision "Implement Oblivious DNS with Kyber-1024 KEM encryption between proxy and resolver")
      (consequences "Strong privacy guarantees for DNS queries; adds latency from encryption; requires post-quantum crypto stack"))

    (adr
      (id "ADR-002")
      (title "Post-quantum cryptography for all signing and encryption")
      (status "accepted")
      (date "2025-12")
      (context "Government surveillance is a primary threat; quantum computing threatens classical crypto within the platform's lifetime")
      (decision "Hybrid Ed448+Dilithium5 for signatures, Kyber-1024 for KEM, SPHINCS+ as fallback")
      (consequences "Future-proof against quantum attacks; larger signature sizes (114+4627 bytes for hybrid); more CPU for signing"))

    (adr
      (id "ADR-003")
      (title "SurrealDB for provenance graph storage")
      (status "accepted")
      (date "2025-12")
      (context "Need graph-capable database for DNS record provenance, blockchain anchoring, and audit trails")
      (decision "Use SurrealDB with kv-mem for dev, rocksdb for production")
      (consequences "Graph queries for provenance; less mature than PostgreSQL; flexible schema"))

    (adr
      (id "ADR-004")
      (title "IPFS for content-addressed storage")
      (status "accepted")
      (date "2025-12")
      (context "Civic data snapshots need verifiable integrity and decentralized replication")
      (decision "Use IPFS for content-addressed storage with snapshot rehydration")
      (consequences "Data integrity via content hashing; decentralized availability; requires IPFS daemon"))

    (adr
      (id "ADR-005")
      (title "Nickel policy contracts for validation")
      (status "accepted")
      (date "2025-12")
      (context "Crypto compliance, consent rules, and webmention policies need machine-checkable enforcement")
      (decision "Use Nickel contracts for policy definition and validation")
      (consequences "Declarative policy enforcement; runtime validation; Nickel is a niche language"))

    (adr
      (id "ADR-006")
      (title "Stapeln container toolchain over Docker")
      (status "accepted")
      (date "2025-12")
      (context "Need formally verified container execution with supply chain security")
      (decision "Use stapeln ecosystem: cerro-torre (build/sign), selur (IPC seal), vordr (verified runtime), selur-compose (orchestration)")
      (consequences "Stronger security guarantees; Ed25519 image signing; .ctp bundles; less ecosystem support than Docker"))

    (adr
      (id "ADR-007")
      (title "ReScript for policy gate over TypeScript")
      (status "accepted")
      (date "2025-12")
      (context "Policy gate needs type safety; TypeScript is banned per hyperpolymath standards")
      (decision "Use ReScript compiled via Deno for the 9-validator policy gate")
      (consequences "Type-safe policy validation; ReScript compiles to clean JS; smaller community than TS"))

    (adr
      (id "ADR-008")
      (title "Self-hosted infrastructure for data sovereignty")
      (status "accepted")
      (date "2025-12")
      (context "Civic data about government transparency must not be controlled by cloud providers who may be compelled to hand over data")
      (decision "Self-host all infrastructure; use Chainguard wolfi-base images for supply chain security")
      (consequences "Full data sovereignty; higher operational burden; lower cost than cloud"))

    (adr
      (id "ADR-009")
      (title "Blockchain anchoring for provenance verification")
      (status "accepted")
      (date "2025-12")
      (context "Government API data provenance needs tamper-evident anchoring beyond the platform")
      (decision "Anchor provenance hashes to Ethereum, Polygon, and Internet Computer")
      (consequences "Tamper-evident audit trail; multi-chain redundancy; smart contract complexity"))

    (adr
      (id "ADR-010")
      (title "Gamification layer for civic engagement (planned)")
      (status "proposed")
      (date "2026-02")
      (context "CivicConnect vision proposes RPG-style progression to encourage sustained civic participation")
      (decision "Design a leveling system (Levels 0-5) rewarding verified civic activities: event attendance, mentorship, cross-regional coordination")
      (consequences "Increased engagement; risk of gaming; needs anti-fraud measures (rate limiting, temporal/spatial validation, reputation decay)")))

  (development-practices
    (code-style
      ("Rust: rustfmt + clippy")
      ("ReScript: rescript format")
      ("Deno: deno fmt")
      ("Nickel: manual formatting"))
    (security
      (principle "Defense in depth")
      (crypto-policy "See CRYPTO-POLICY.adoc — 13 CPR requirements")
      (threat-model "Government surveillance, employer retaliation, hostile infiltrators, data breaches")
      (scanning "panic-attack assail for vulnerability detection")
      (container-security "stapeln + chainguard images for supply chain"))
    (testing
      ("Rust: cargo test + proptest for crypto invariants")
      ("ReScript: built-in test framework")
      ("Deno: deno test for consent API")
      ("Integration: container-based E2E tests"))
    (versioning "SemVer")
    (documentation "AsciiDoc primary, Markdown for compatibility")
    (branching "main for stable, feature/ for development")
    (ci-cd "GitHub Actions + Hypatia neurosymbolic scanning"))

  (design-rationale
    (privacy-first "Every feature must consider: could this endanger a citizen or organizer? If yes, redesign or remove")
    (post-quantum "Classical crypto alone is insufficient for a platform handling government transparency data with a multi-decade lifetime")
    (self-hosted "Data sovereignty is non-negotiable; cloud providers can be compelled to hand over data")
    (consent-aware "All data processing requires explicit, auditable consent")
    (decentralized-verification "No central authority for trust; blockchain anchoring provides tamper-evidence")
    (regenerative "RSR methodology: code health improves over time, 20% capacity for tech debt reduction")))
