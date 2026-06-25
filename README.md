<!--
SPDX-License-Identifier: CC-BY-SA-4.0
SPDX-FileCopyrightText: 2025-2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->

Privacy-first civic engagement platform with government API integration,
post-quantum secure DNS, and provenance-grade audit infrastructure.

[![RSR Certified](https://img.shields.io/badge/RSR-Certified-gold)](https://github.com/hyperpolymath/rhodium-standard-repositories)
[![License: MPL-2.0](https://img.shields.io/badge/License-MPL--2.0-blue.svg)](https://www.mozilla.org/MPL/2.0/)

<div id="toc">

</div>

# Overview

Civic-Connect combines hardened ingress infrastructure, oblivious DNS
resolution, GraphQL DNS APIs, and SurrealDB provenance graphs to enable
privacy-preserving civic engagement with government data.

## Key Components

- **IndieWeb2 Bastion** — Hardened ingress gateway with consent-first
  architecture

- **Oblivious DNS (odns-rs)** — Post-quantum encrypted DNS
  proxy/resolver (Kyber-1024 + Dilithium5)

- **GraphQL DNS API** — DNSSEC-signed DNS queries via GraphQL
  (Rust/Axum)

- **Civic-Stream** — Government API integration and stream processing

- **SurrealDB Provenance** — Graph database for audit trails and data
  provenance

- **IPFS Storage** — Content-addressed snapshots with rehydration

- **Nickel Policies** — Machine-checkable consent and crypto compliance
  contracts

## Security Features

- Post-quantum cryptography: Kyber-1024 (KEM), Dilithium5 (signatures),
  SPHINCS+ (fallback)

- Hybrid signatures: Ed448 + Dilithium5 dual-sign for
  quantum-resistant + classical assurance

- Zero-knowledge DNS: Oblivious DNS prevents query correlation

- Provenance anchoring: Ethereum, Polygon, and Internet Computer for
  tamper-evidence

# Architecture

See <a href="TOPOLOGY.md" class="md">TOPOLOGY</a> for a visual
architecture map and completion dashboard.

    Users / Citizens (GUI / PWA / API Clients)
                │
                ▼
        IndieWeb2 Bastion (Hardened Ingress, oDNS)
                │
                ▼
        Civic-Stream (Government API Integration)
                │
                ▼
        Data Layer (SurrealDB Graph + IPFS)

# Quick Start

```bash
cd indieweb2-bastion
just bootstrap
just all
```

This bootstraps dependencies (jq, capnproto, nickel, deno), validates
Nickel policies, runs the ReScript policy gate, and starts the GUI/PWA
server on port 8443.

## Prerequisites

- **Rust** (nightly) — Core services

- **Deno** — Consent API, policy gate runtime

- **Nickel** — Policy contracts

- **SurrealDB** — Provenance graph

- **IPFS** — Content-addressed storage

- **stapeln toolchain** — cerro-torre, selur, vordr, selur-compose

# Technology Stack

| Language | Purpose                                         |
|----------|-------------------------------------------------|
| Rust     | odns-rs, graphql-dns-api, webmention-limiter    |
| ReScript | Policy gate (9 validators), UI components       |
| Deno     | Consent API, signing, publishing, static server |
| Nickel   | Policy contracts and validation                 |
| Solidity | Smart contracts (Ethereum/Polygon)              |
| Motoko   | Internet Computer canisters                     |

# Documentation

- [Architecture Topology & Completion Dashboard](TOPOLOGY.md)

- [CivicConnect Vision & Handover](docs/planning/HANDOVER_SUMMARY.md)

- [RSR Compliance Guide](docs/planning/RSR_COMPLIANCE.md)

- [Cryptographic Policy (13 CPR
  requirements)](indieweb2-bastion/CRYPTO-POLICY.adoc)

- [IndieWeb2 Bastion Documentation](indieweb2-bastion/README.adoc)

# License

SPDX-License-Identifier: CC-BY-SA-4.0

Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath)
\<[j.d.a.jewell@open.ac](j.d.a.jewell@open.ac).uk\>
