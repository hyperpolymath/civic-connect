# Test & Benchmark Requirements

## Current State
- Unit tests: ~9 Rust test files (integration_test.rs, security_test.rs + test harness files)
- Integration tests: 2 (webmention-rate-limiter integration + graphql-dns-api integration)
- E2E tests: NONE
- Benchmarks: NONE
- panic-attack scan: NEVER RUN

## What's Missing
### Point-to-Point (P2P)
#### ODNS-RS Common (Rust — 5 files)
- crypto.rs — likely tested via integration tests
- protocol.rs — likely undertested
- signatures.rs — needs comprehensive test coverage (crypto correctness critical)
- sphincs_fallback.rs — post-quantum fallback needs edge case testing

#### ODNS-RS Proxy (Rust)
- main.rs — no dedicated unit tests

#### ODNS-RS Resolver (Rust)
- main.rs — no dedicated unit tests

#### Webmention Rate Limiter (Rust — 6 source files)
- config.rs — no dedicated tests
- handlers.rs — integration test exists but unit coverage unclear
- limiter.rs — rate limiting logic needs thorough testing
- validator.rs — input validation needs edge case tests
- Security test exists (good)
- Test harness (attacks.rs, generators.rs, metrics.rs) is well-structured

#### GraphQL DNS API (Rust — 9 source files)
- blockchain.rs — no dedicated tests
- consent.rs — no dedicated tests
- db.rs — no dedicated tests
- dnssec.rs — no dedicated tests
- error.rs — no dedicated tests
- models.rs — no dedicated tests
- policy.rs — no dedicated tests
- resolvers.rs — integration test exists
- schema.rs — no dedicated tests

#### ReScript UI (17 files)
- ZERO test files

#### Idris2 ABI (6 files) + Zig FFI (6 files)
- 2 Zig integration tests only

### End-to-End (E2E)
- Full DNS resolution: query -> ODNS proxy -> resolver -> DNSSEC validation -> response
- Webmention flow: receive mention -> validate -> rate limit -> process
- GraphQL API: query -> resolve -> blockchain verification -> consent check -> response
- Attack simulation: DDoS -> rate limiter -> graceful degradation
- SPHINCS+ post-quantum signature verification

### Aspect Tests
- [ ] Security (DNS spoofing, DNSSEC bypass, rate limiter evasion, GraphQL injection, SSRF, auth bypass)
- [ ] Performance (DNS resolution latency, rate limiter throughput, GraphQL query complexity limits)
- [ ] Concurrency (concurrent DNS queries, rate limiter fairness under load)
- [ ] Error handling (DNSSEC validation failure, blockchain unavailability, network partitions)
- [ ] Accessibility (ReScript UI if user-facing)

### Build & Execution
- [ ] cargo build for all Rust components — not verified
- [ ] cargo test — not verified
- [ ] ReScript build — not verified
- [ ] Self-diagnostic — none

### Benchmarks Needed
- DNS resolution latency (encrypted vs unencrypted)
- Rate limiter throughput and fairness
- SPHINCS+ signature verification speed vs RSA/ECDSA
- GraphQL query response time
- Concurrent query handling capacity

### Self-Tests
- [ ] panic-attack assail on own repo
- [ ] DNS resolution self-check
- [ ] DNSSEC chain validation self-test

## Priority
- **HIGH** — Security-critical project (DNS privacy, rate limiting, DNSSEC, post-quantum crypto) with 30 Rust + 17 ReScript + 6 Zig + 6 Idris2 files. The security test harness for webmention-rate-limiter is well-done, but the GraphQL DNS API (9 source files) has only 1 integration test, and the ReScript UI has ZERO tests. For a security-focused project, this needs significantly more testing, especially around crypto correctness and attack resistance.

## FAKE-FUZZ ALERT

- `tests/fuzz/placeholder.txt` is a scorecard placeholder inherited from rsr-template-repo — it does NOT provide real fuzz testing
- Replace with an actual fuzz harness (see rsr-template-repo/tests/fuzz/README.adoc) or remove the file
- Priority: P2 — creates false impression of fuzz coverage
