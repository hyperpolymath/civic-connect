## Machine-Readable Artefacts

The following files in `.machine_readable/` contain structured project metadata:

- `STATE.scm` - Current project state and progress
- `META.scm` - Architecture decisions and development practices
- `ECOSYSTEM.scm` - Position in the ecosystem and related projects
- `AGENTIC.scm` - AI agent interaction patterns
- `NEUROSYM.scm` - Neurosymbolic integration config
- `PLAYBOOK.scm` - Operational runbook

---

# CLAUDE.md - Civic-Connect Development Instructions

## Project Overview

Civic-Connect is a privacy-first civic engagement platform combining:
- **IndieWeb2 Bastion**: Hardened ingress with Oblivious DNS, GraphQL DNS API, SurrealDB provenance
- **Civic-Stream**: Government API integration and stream processing
- **CivicConnect Vision**: Gamified civic participation (planned, see `docs/planning/`)

## Architecture

```
Users (GUI / PWA / API Clients)
        │
        ▼
IndieWeb2 Bastion
  ├── GraphQL DNS API (Rust/Axum, port 8443)
  ├── Oblivious DNS (Rust/Tokio, port 853/8853)
  ├── Consent API (Deno, port 443)
  └── Policy Gate (ReScript + Deno, 9 validators)
        │
        ▼
Civic-Stream (Government API Integration)
        │
        ▼
Data Layer
  ├── SurrealDB (provenance graph, kv-mem/rocksdb)
  ├── IPFS (content-addressed storage)
  └── Blockchain (Ethereum, Polygon, Internet Computer)
```

## Critical Security Requirements

1. **Post-quantum crypto is mandatory** — Kyber-1024 for KEM, Dilithium5 for signatures, SPHINCS+ fallback
2. **Hybrid signatures** — Ed448 + Dilithium5 dual-sign (both MUST pass)
3. **Zero-knowledge DNS** — Server never sees plaintext DNS queries
4. **Consent-first** — All data processing requires explicit, auditable consent
5. **Self-hosted only** — No cloud provider dependencies

### Crypto Policy Requirements (CPR)

13 requirements defined in `indieweb2-bastion/CRYPTO-POLICY.adoc`:
- CPR-001: Argon2id (512MiB/8it/4ln)
- CPR-002: SHAKE3-512 (FIPS 202)
- CPR-003: Dilithium5-AES (ML-DSA-87)
- CPR-004: Kyber-1024 (ML-KEM-1024)
- CPR-005: Ed448+Dilithium5 hybrid
- CPR-006: XChaCha20-Poly1305
- CPR-007: HKDF-SHA3-512
- CPR-008: ChaCha20-DRBG
- CPR-009: BLAKE3 + SHAKE3-512
- CPR-010: QUIC + HTTP/3 + IPv6 (**not started**)
- CPR-011: WCAG 2.3 AAA (**not started**)
- CPR-012: SPHINCS+ fallback
- CPR-013: Coq/Isabelle verification (**not started**)

## Development Workflow

### Building

```bash
# IndieWeb2 Bastion
cd indieweb2-bastion
just bootstrap    # Install deps (jq, capnproto, nickel, deno)
just all          # Validate policies, run gate, start server

# Civic-Stream
cd civic-stream
just build
```

### Testing

```bash
# Rust tests (odns-rs, graphql-dns-api)
cargo test

# ReScript policy gate
deno test

# Integration tests
just test
```

### Container Operations

```bash
# Build with stapeln
cerro-torre build -f Containerfile .
cerro-torre sign <image>       # Ed25519 signatures

# Run with vordr
vordr run <image>

# Orchestrate
selur-compose -f compose.toml up
```

## Language Policy (Hyperpolymath Standard)

### ALLOWED Languages & Tools

| Language/Tool | Use Case | Notes |
|---------------|----------|-------|
| **Rust** | Core services, crypto, DNS, API | Primary for performance-critical code |
| **ReScript** | Policy gate, UI components | Compiles to JS, type-safe |
| **Deno** | Runtime, consent API, signing | Replaces Node/npm/bun |
| **Nickel** | Policy contracts | Complex config validation |
| **Guile Scheme** | State/meta files | STATE.scm, META.scm, ECOSYSTEM.scm |
| **Solidity** | Smart contracts | Ethereum/Polygon |
| **Motoko** | Smart contracts | Internet Computer canisters |
| **Bash/POSIX Shell** | Scripts, automation | Keep minimal |

### BANNED - Do Not Use

| Banned | Replacement |
|--------|-------------|
| TypeScript | ReScript |
| Node.js | Deno |
| npm/Bun/pnpm/yarn | Deno |
| Go | Rust |
| Python | Julia/Rust/ReScript |
| Java/Kotlin | Rust |
| Docker | Podman + stapeln |

### Package Management

- **Primary**: Guix (guix.scm)
- **Fallback**: Nix (flake.nix)
- **JS deps**: Deno (deno.json imports)

### Security Requirements

- No MD5/SHA1 for security (use SHA256+, BLAKE3, SHAKE3-512)
- HTTPS only (no HTTP URLs)
- No hardcoded secrets
- SHA-pinned dependencies
- SPDX license headers on all files
- Post-quantum algorithms for all new crypto code

## Known Issues (from panic-attack assail, 2026-02-14)

| Severity | Category | Location |
|----------|----------|----------|
| Medium | CommandInjection | scripts/interactive_tidy.sh |
| Medium | PanicPath | graphql-dns-api/tests/integration_test.rs |
| Medium | PanicPath | odns-rs/common/src/crypto.rs |

## Priority Actions

1. Complete graphql-dns-api to 100% (currently 70%)
2. Fix panic paths from security scan
3. Remove node_modules from indieweb2-bastion (use Deno only)
4. Expand civic-stream government API endpoints (40% → 80%)
5. QUIC/HTTP3 transport (CPR-010)
6. WCAG 2.3 AAA accessibility (CPR-011)

## CivicConnect Vision (Future)

Planning documents in `docs/planning/` describe a gamified civic engagement layer:
- Leveling system (Levels 0-5) for civic participation
- Verified event attendance (QR + ed25519 signatures)
- E2E encrypted messaging (Signal protocol)
- Mentor matching and cross-regional coordination
- Privacy-preserving location (H3 geohashing)

This is currently in PLANNING status and depends on the bastion and civic-stream reaching production readiness.
