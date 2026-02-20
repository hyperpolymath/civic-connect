;; SPDX-License-Identifier: PMPL-1.0-or-later
;; STATE.scm - Project state for civic-connect
;; Media-Type: application/vnd.state+scm

(state
  (metadata
    (version "0.3.0")
    (schema-version "1.0")
    (created "2026-01-03")
    (updated "2026-02-20")
    (project "civic-connect")
    (repo "github.com/hyperpolymath/civic-connect"))

  (project-context
    (name "civic-connect")
    (tagline "Privacy-first civic engagement platform with government API integration")
    (tech-stack
      ("Rust" "Core services: odns-rs, graphql-dns-api, webmention-limiter")
      ("ReScript" "Policy gate, frontend UI components")
      ("Deno" "Consent API, signing, publishing, static server")
      ("Nickel" "Policy contracts and validation")
      ("SurrealDB" "Provenance graph and audit trail")
      ("IPFS" "Content-addressed storage and snapshot rehydration")))

  (current-position
    (phase "active-development")
    (overall-completion 70)
    (components
      (component
        (name "indieweb2-bastion")
        (description "Hardened ingress, oDNS resolver, GraphQL DNS API")
        (completion 60)
        (sub-components
          (sub "graphql-dns-api" 70 "Rust/Axum, DNSSEC signing, BLAKE3 hashing")
          (sub "odns-rs-proxy" 100 "Rust/Tokio, Kyber-1024 KEM, post-quantum")
          (sub "odns-rs-resolver" 100 "Rust, XChaCha20-Poly, HKDF-SHA3-512")
          (sub "odns-rs-signatures" 100 "Ed448+Dilithium5 hybrid signatures")
          (sub "sphincs-fallback" 100 "SPHINCS+/SLH-DSA fallback (CPR-012)")
          (sub "consent-api" 80 "Deno, port 443")
          (sub "webmention-limiter" 60 "Rust rate limiter")
          (sub "policy-gate" 80 "ReScript + Deno, 9 validators")
          (sub "nickel-policies" 100 "crypto, schema, webmention policies")
          (sub "smart-contracts" 60 "Solidity/Hardhat + Motoko canisters")
          (sub "container-stack" 100 "stapeln: cerro-torre, selur, vordr")
          (sub "dnssec-hybrid" 100 "Ed448+Dilithium5 hybrid signing")
          (sub "pq-crypto" 80 "Kyber-1024 + Dilithium5 integrated")
          (sub "seccomp-selinux" 80 "Defined, not yet fully enforced")))
      (component
        (name "civic-stream")
        (description "Government API integration and stream processing")
        (completion 50)
        (sub-components
          (sub "core-logic" 60 "Stream processing active")
          (sub "gov-api-integrations" 40 "Initial endpoints connected")))
      (component
        (name "data-layer")
        (description "SurrealDB provenance and IPFS storage")
        (completion 100)
        (sub-components
          (sub "surrealdb-provenance" 100 "Graph schema stable")
          (sub "ipfs-storage" 100 "Snapshot rehydration active")))
      (component
        (name "repo-infrastructure")
        (description "Container toolchain, automation, machine-readable metadata")
        (completion 100)
        (sub-components
          (sub "stapeln-toolchain" 100 "vordr/selur container security verified")
          (sub "justfile-automation" 100 "Bootstrap and build tasks")
          (sub "machine-readable" 100 "STATE/META/ECOSYSTEM tracking"))))
    (working-features
      ("Oblivious DNS proxy and resolver with post-quantum crypto")
      ("GraphQL DNS API with DNSSEC signing")
      ("SurrealDB provenance graph")
      ("IPFS content-addressed storage")
      ("Nickel policy validation")
      ("ReScript policy gate with 9 validators")
      ("Container stack (stapeln) build and signing")
      ("Consent API (Deno)")
      ("Hybrid Ed448+Dilithium5 signatures")
      ("SPHINCS+ fallback signatures")))

  (route-to-mvp
    (milestones
      (milestone
        (name "bastion-complete")
        (description "IndieWeb2 Bastion fully production-ready")
        (target "2026-Q1")
        (completion 60)
        (remaining
          ("Complete graphql-dns-api (QUIC/HTTP3 support, CPR-010)")
          ("Enforce seccomp/SELinux policies in runtime")
          ("Full PQ crypto test coverage")
          ("Accessibility compliance (WCAG 2.3 AAA, CPR-011)")))
      (milestone
        (name "civic-stream-mvp")
        (description "Government API integration MVP")
        (target "2026-Q2")
        (completion 50)
        (remaining
          ("Expand government API endpoint coverage")
          ("Stream processing reliability hardening")
          ("End-to-end integration tests")))
      (milestone
        (name "gamification-layer")
        (description "Civic engagement gamification (from CivicConnect vision)")
        (target "2026-Q3")
        (completion 0)
        (remaining
          ("Design leveling system for civic participation")
          ("Implement verification system for civic activities")
          ("Build reputation and progression mechanics")
          ("E2E encrypted messaging for organizing")))
      (milestone
        (name "formal-verification")
        (description "Coq/Isabelle proofs for crypto primitives (CPR-013)")
        (target "2026-Q4")
        (completion 0)
        (remaining
          ("Formal verification of hybrid signature scheme")
          ("Proof of oDNS privacy guarantees")))))

  (blockers-and-issues
    (critical)
    (high
      ("QUIC/HTTP3 + IPv6 not yet implemented (CPR-010)")
      ("WCAG 2.3 AAA accessibility not started (CPR-011)")
      ("Coq/Isabelle formal verification not started (CPR-013)"))
    (medium
      ("CommandInjection risk in scripts/interactive_tidy.sh")
      ("PanicPath in graphql-dns-api integration tests")
      ("PanicPath in odns-rs/common/src/crypto.rs")
      ("Smart contracts at 60% - need audit")
      ("webmention-limiter at 60% - needs completion"))
    (low
      ("node_modules present in indieweb2-bastion (should use Deno only)")))

  (critical-next-actions
    (immediate
      ("Complete graphql-dns-api to 100%")
      ("Fix panic paths found by panic-attack assail")
      ("Remove node_modules, migrate to Deno-only"))
    (this-week
      ("Expand civic-stream government API endpoints")
      ("Enforce seccomp/SELinux in container runtime")
      ("Full PQ crypto integration test suite"))
    (this-month
      ("QUIC/HTTP3 transport layer (CPR-010)")
      ("WCAG 2.3 AAA accessibility audit (CPR-011)")
      ("Design gamification layer from CivicConnect vision")))

  (session-history
    (entry
      (date "2026-02-20")
      (action "Integrated CivicConnect planning documents")
      (details "Merged gamified organizing vision into project roadmap. Updated STATE, ECOSYSTEM, META with actual project data. Placed planning docs in docs/planning/."))))
