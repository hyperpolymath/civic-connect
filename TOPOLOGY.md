<!-- SPDX-License-Identifier: PMPL-1.0-or-later -->
<!-- TOPOLOGY.md — Project architecture map and completion dashboard -->
<!-- Last updated: 2026-02-20 -->

# Civic-Connect — Project Topology

## System Architecture

```
                        ┌─────────────────────────────────────────┐
                        │              USERS / CITIZENS           │
                        │        (GUI / PWA / API Clients)        │
                        └───────────────────┬─────────────────────┘
                                            │
                                            ▼
                        ┌─────────────────────────────────────────┐
                        │           INDIEWEB2 BASTION             │
                        │    (Hardened Ingress, oDNS Resolver)    │
                        │  ┌───────────┐  ┌───────────────────┐  │
                        │  │  GraphQL  │  │  Oblivious DNS    │  │
                        │  │  DNS API  │  │  (odns-rs)        │  │
                        │  └─────┬─────┘  └────────┬──────────┘  │
                        │        │     ┌───────────┘             │
                        │  ┌─────┴─────┴───────────────────────┐ │
                        │  │ Consent API   Policy Gate         │ │
                        │  │ (Deno)        (ReScript + Deno)   │ │
                        │  └───────────────────────────────────┘ │
                        └────────────────────┬───────────────────┘
                                             │
                                             ▼
                        ┌─────────────────────────────────────────┐
                        │           CIVIC-STREAM CORE             │
                        │    (Government API Integration)         │
                        └───────────────────┬─────────────────────┘
                                            │
                                            ▼
                        ┌─────────────────────────────────────────┐
                        │             DATA LAYER                  │
                        │  ┌───────────┐  ┌───────────────────┐  │
                        │  │ SurrealDB │  │  IPFS             │  │
                        │  │ (Graph)   │  │ (Content Address) │  │
                        │  └───────────┘  └───────────────────┘  │
                        └─────────────────────────────────────────┘

                        ┌─────────────────────────────────────────┐
                        │          BLOCKCHAIN ANCHORS             │
                        │  Ethereum  │  Polygon  │  Internet Comp │
                        └─────────────────────────────────────────┘

                        ┌─────────────────────────────────────────┐
                        │          REPO INFRASTRUCTURE            │
                        │  Stapeln Toolchain  .machine_readable/  │
                        │  Justfile / Nix     Nickel Policies     │
                        └─────────────────────────────────────────┘

                        ┌─────────────────────────────────────────┐
                        │      PLANNED: CIVICCONNECT LAYER        │
                        │  Gamification │ Events │ Messaging      │
                        │  Leveling     │ QR Verify │ Mentoring   │
                        └─────────────────────────────────────────┘
```

## Completion Dashboard

```
COMPONENT                          STATUS              NOTES
─────────────────────────────────  ──────────────────  ─────────────────────────────────
INDIEWEB2 BASTION
  Hardened Ingress (Bastion)        ██████████ 100%    Config stable
  Oblivious DNS proxy (odns-rs)     ██████████ 100%    Kyber-1024 + XChaCha20-Poly
  Oblivious DNS resolver            ██████████ 100%    HKDF-SHA3-512, Ed448+Dilithium5
  SPHINCS+ Fallback                 ██████████ 100%    SLH-DSA (CPR-012)
  DNSSEC Hybrid Signing             ██████████ 100%    Ed448+Dilithium5
  GraphQL DNS API                   ███████░░░  70%    Standard endpoints, needs QUIC
  Consent API (Deno)                ████████░░  80%    Port 443, production
  Policy Gate (ReScript)            ████████░░  80%    9 validators, crypto compliance
  Webmention Rate Limiter           ██████░░░░  60%    Rust, needs completion
  PQ Crypto Integration             ████████░░  80%    Kyber + Dilithium5 integrated
  Smart Contracts                   ██████░░░░  60%    Solidity + Motoko functional
  seccomp/SELinux                   ████████░░  80%    Defined, not fully enforced

CIVIC-STREAM
  Core Logic                        ██████░░░░  60%    Stream processing active
  Gov API Integrations              ████░░░░░░  40%    Initial endpoints connected

DATA & POLICIES
  SurrealDB Provenance              ██████████ 100%    Graph schema stable
  IPFS Storage Integration          ██████████ 100%    Snapshot rehydration active
  Nickel Policies                   ██████████ 100%    Validation rituals stable
  Crypto Policy (13 CPRs)           ████████░░  80%    CPR-010,011,013 not started

REPO INFRASTRUCTURE
  Stapeln (vordr/selur)             ██████████ 100%    Container security verified
  Justfile Automation               ██████████ 100%    Bootstrap & build tasks
  .machine_readable/                ██████████ 100%    STATE/META/ECOSYSTEM populated
  Fleet Enrollment                  ██████████ 100%    gitbot-fleet integrated

PLANNED: CIVICCONNECT LAYER
  Gamification / Leveling           ░░░░░░░░░░   0%    Vision in docs/planning/
  Event System / QR Verify          ░░░░░░░░░░   0%    Planned Q3 2026
  E2E Messaging                     ░░░░░░░░░░   0%    Signal protocol planned
  Mentor Matching                   ░░░░░░░░░░   0%    Depends on leveling system

─────────────────────────────────────────────────────────────────────────────
OVERALL:                            ███████░░░  ~70%   Bastion maturing, stream growing
```

## Key Dependencies

```
Nickel Policy ───► Cerro-Torre ───► Bastion Ingress ───► Civic-Stream
     │                 │                 │                  │
     └─────────────────┴────────┬────────┴──────────────────┘
                                ▼
                           SurrealDB (Graph)
                                │
                                ▼
                    Blockchain Anchors (ETH/Poly/IC)
```

## Crypto Algorithm Map

```
REQUIREMENT   ALGORITHM                    STATUS
───────────   ──────────────────────────   ──────────────
CPR-001       Argon2id (512MiB/8it/4ln)    Dep added
CPR-002       SHAKE3-512 (FIPS 202)        Dep added
CPR-003       Dilithium5-AES (ML-DSA-87)   odns-rs sigs
CPR-004       Kyber-1024 (ML-KEM-1024)     odns-rs crypto
CPR-005       Ed448 + Dilithium5 hybrid    odns-rs + DNSSEC
CPR-006       XChaCha20-Poly1305           odns-rs crypto
CPR-007       HKDF-SHA3-512                odns-rs crypto
CPR-008       ChaCha20-DRBG                OsRng used
CPR-009       BLAKE3 + SHAKE3-512          graphql-api
CPR-010       QUIC + HTTP/3 + IPv6         NOT STARTED
CPR-011       WCAG 2.3 AAA                 NOT STARTED
CPR-012       SPHINCS+ fallback            odns-rs module
CPR-013       Coq/Isabelle verification    NOT STARTED
```

## Update Protocol

This file is maintained by both humans and AI agents. When updating:

1. **After completing a component**: Change its bar and percentage
2. **After adding a component**: Add a new row in the appropriate section
3. **After architectural changes**: Update the ASCII diagram
4. **Date**: Update the `Last updated` comment at the top of this file

Progress bars use: `█` (filled) and `░` (empty), 10 characters wide.
Percentages: 0%, 10%, 20%, ... 100% (in 10% increments).
