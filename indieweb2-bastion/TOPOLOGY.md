# TOPOLOGY.md - indieweb2-bastion

## System Architecture

```
                          +---------------------------+
                          |     indieweb2-bastion      |
                          |  Multi-chain IndieWeb      |
                          |  Identity Platform         |
                          +---------------------------+
                                      |
               +----------------------+----------------------+
               |                      |                      |
    +----------v----------+ +---------v---------+ +---------v---------+
    |   graphql-dns-api   | |     odns-rs/      | |   services/       |
    |   (Rust + Axum)     | |   proxy + resolver| |                   |
    |   Port 8443         | |   (Rust + Tokio)  | |  consent-api      |
    |                     | |   Port 853 / 8853 | |  (Deno, port 443) |
    |  GraphQL endpoint   | |                   | |                   |
    |  DNSSEC signing     | |  Kyber-1024 KEM   | |  webmention-rate  |
    |  BLAKE3 hashing     | |  XChaCha20-Poly   | |  -limiter (Rust)  |
    +----------+----------+ |  HKDF-SHA3-512    | +---------+---------+
               |            |  Ed448+Dilithium5 |           |
               |            +---------+---------+           |
               |                      |                     |
    +----------v----------+ +---------v---------+ +---------v---------+
    |     SurrealDB       | |   Upstream DNS    | |   Blockchain      |
    |   (kv-mem / rocksdb)| |   (UDP, IPv6)     | |   Providers       |
    |                     | |   [2606:4700::]   | |   Ethereum +      |
    |  dns_records        | +-------------------+ |   Polygon +       |
    |  dnssec_zones       |                       |   Internet Comp.  |
    |  blockchain_prov    |                       +-------------------+
    +---------------------+

              +---------------------------------------------+
              |           Policy Layer                       |
              |                                             |
              |  policy-gate (ReScript + Deno)              |
              |    9 validators, crypto compliance          |
              |  policy/curps/schema.ncl   (Nickel)         |
              |  policy/curps/policy.ncl   (crypto policy)  |
              |  policy/curps/webmention.ncl                |
              |  .machine_readable/CRYPTO-POLICY.scm        |
              +---------------------------------------------+

              +---------------------------------------------+
              |           Container Stack (stapeln)          |
              |                                             |
              |  cerro-torre build   Containerfile           |
              |  cerro-torre sign    .ctp bundles            |
              |  selur seal          IPC bridge              |
              |  selur-compose       compose.toml            |
              |  vordr run           verified runtime        |
              |  Base: cgr.dev/chainguard/wolfi-base         |
              +---------------------------------------------+

              +---------------------------------------------+
              |           Smart Contracts                    |
              |                                             |
              |  contracts/          Solidity (Hardhat)      |
              |  Internet Computer   Motoko canisters        |
              +---------------------------------------------+
```

### Data Flow

```
Client (DoT :853) --> odns-proxy --> [Kyber-1024 encrypt] --> odns-resolver
                                                                    |
                                                            [decrypt + resolve]
                                                                    |
                                                            Upstream DNS (UDP)

Client (HTTPS :8443) --> graphql-dns-api --> SurrealDB
                              |
                        DNSSEC sign (Ed25519 interim)
                              |
                        Blockchain anchor (Ethereum/Polygon/IC)
```

### Hybrid Signature Flow (CPR-005)

```
Message --> Ed448 sign (classical)  --> HybridSignature
        |                               [114 + 4627 bytes]
        +-> Dilithium5 sign (PQ)    -+
                                     |
Verify:  Both MUST pass --> Ok(())   |
         Either fails  --> Err(...)  +
```

## Completion Dashboard

| Component               | Status       | Progress                   | Pct  |
|--------------------------|-------------|----------------------------|------|
| graphql-dns-api (Rust)   | Partial     | `███████░░░` | 70%  |
| odns-rs/proxy (Rust)     | Complete    | `██████████` | 100% |
| odns-rs/resolver (Rust)  | Complete    | `██████████` | 100% |
| odns-rs/signatures       | Complete    | `██████████` | 100% |
| odns-rs/sphincs_fallback | Complete    | `██████████` | 100% |
| consent-api (Deno)       | Production  | `████████░░` | 80%  |
| webmention-limiter       | Partial     | `██████░░░░` | 60%  |
| policy-gate (ReScript)   | Functional  | `████████░░` | 80%  |
| crypto-policy (scheme)   | Defined     | `██████████` | 100% |
| Nickel policy configs    | Complete    | `██████████` | 100% |
| smart contracts          | Functional  | `██████░░░░` | 60%  |
| container (stapeln)      | Complete    | `██████████` | 100% |
| DNSSEC (hybrid)          | Complete    | `██████████` | 100% |
| PQ crypto (Kyber+Dilith) | Integrated  | `████████░░` | 80%  |
| SPHINCS+ fallback        | Complete    | `██████████` | 100% |
| seccomp/SELinux          | Defined     | `████████░░` | 80%  |
| fleet enrollment         | Complete    | `██████████` | 100% |
| **Overall**              | **Active**  | `██████░░░░` | **60%** |

### Legend

- Partial = functional but missing PQ crypto or full test coverage
- Functional = working with validators/tests, may need v13 migration
- Defined = policy/config written, not yet enforced in code

## Key Dependencies

| Dependency           | Version | Purpose                              |
|----------------------|---------|--------------------------------------|
| pqcrypto-kyber       | 0.8     | Kyber-1024 / ML-KEM-1024 (FIPS 203) |
| pqcrypto-dilithium   | 0.5     | Dilithium5 / ML-DSA-87 (FIPS 204)   |
| ed448-goldilocks-plus| 0.16    | Ed448 classical signatures           |
| chacha20poly1305     | 0.10    | XChaCha20-Poly1305 symmetric         |
| hkdf + sha3          | 0.12    | HKDF-SHA3-512 key derivation         |
| blake3               | 1.5     | Content hashing (CPR-009)            |
| hickory-proto        | 0.24    | DNS protocol (replaced trust-dns)    |
| surrealdb            | 1.5     | Graph DB (kv-mem default)            |
| async-graphql        | 7.0     | GraphQL framework                    |
| axum                 | 0.8     | HTTP framework                       |
| ethers               | 2.0     | Ethereum/Polygon blockchain          |
| pqcrypto-sphincsplus | 0.7     | SPHINCS+ / SLH-DSA fallback (CPR-012)|
| tokio                | 1.x     | Async runtime                        |
| rustls               | 0.23    | TLS 1.3 (proxy)                      |
| rescript             | 12.1    | Policy-gate (compiled via Deno)      |
| nickel               | -       | Policy contracts                     |
| deno                 | 2.x     | Consent API + policy-gate runtime    |

### Crypto Algorithm Map (CRYPTO-POLICY.adoc)

| Requirement | Algorithm                  | Status          |
|-------------|----------------------------|-----------------|
| CPR-001     | Argon2id (512MiB/8it/4ln)  | Dep added       |
| CPR-002     | SHAKE3-512 (FIPS 202)      | Dep added       |
| CPR-003     | Dilithium5-AES (ML-DSA-87) | odns-rs sigs    |
| CPR-004     | Kyber-1024 (ML-KEM-1024)   | odns-rs crypto  |
| CPR-005     | Ed448 + Dilithium5 hybrid  | odns-rs + DNSSEC|
| CPR-006     | XChaCha20-Poly1305         | odns-rs crypto  |
| CPR-007     | HKDF-SHA3-512              | odns-rs crypto  |
| CPR-008     | ChaCha20-DRBG              | OsRng used      |
| CPR-009     | BLAKE3 + SHAKE3-512        | graphql-api     |
| CPR-010     | QUIC + HTTP/3 + IPv6       | Not started     |
| CPR-011     | WCAG 2.3 AAA               | Not started     |
| CPR-012     | SPHINCS+ fallback          | odns-rs module  |
| CPR-013     | Coq/Isabelle verification  | Not started     |

### Scan Results (panic-attack assail, 2026-02-14)

| Severity | Category             | Location                               |
|----------|----------------------|----------------------------------------|
| Medium   | CommandInjection     | scripts/interactive_tidy.sh            |
| Medium   | PanicPath            | graphql-dns-api/tests/integration_test.rs |
| Medium   | PanicPath            | odns-rs/common/src/crypto.rs           |
