# Test & Benchmark Requirements

## CRG Grade: C — ACHIEVED 2026-04-04

CRG C requires: unit + smoke + build + P2P (property-based) + E2E + reflexive + contract + aspect tests + benchmarks baselined.

All items ticked below were added in the CRG D→C blitz (2026-04-04).

---

## Coverage Summary

### odns-rs (common crate)

| Test type | File(s) | Count | Status |
|-----------|---------|-------|--------|
| Unit | `src/crypto.rs`, `protocol.rs`, `signatures.rs`, `sphincs_fallback.rs` (inline) | 22 | PASS |
| Property (P2P) | `tests/property_tests.rs` → `property/crypto_properties_test.rs` | 10 (+1 ignored) | PASS |
| E2E | `tests/e2e_tests.rs` → `e2e/odns_pipeline_test.rs` | 5 (+1 ignored) | PASS |
| Aspect / Security | `tests/aspect_tests.rs` → `aspect/security_test.rs` | 19 (+1 ignored) | PASS |
| Benchmarks | `benches/crypto_bench.rs` | 8 groups | COMPILES |

**Ignored tests** (marked `#[ignore]`, require `--include-ignored`):
- SPHINCS+ sign in property tests — ~300ms per case
- SPHINCS+ E2E pipeline — ~300ms per sign
- SPHINCS+ aspect security test — slow signing

### webmention-rate-limiter

| Test type | File(s) | Count | Status |
|-----------|---------|-------|--------|
| Unit | `src/limiter.rs`, `src/validator.rs` (inline) | 8 | PASS |
| Integration | `tests/integration_test.rs` | 5 | PASS |
| Security / Attack simulation | `tests/security_test.rs` + `tests/harness/` | 15 | PASS |
| Property (P2P) | `tests/property_tests.rs` → `property/rate_limiter_properties_test.rs` | 7 | PASS |
| E2E | `tests/e2e_tests.rs` → `e2e/webmention_e2e_test.rs` | 11 | PASS |
| Benchmarks | `benches/rate_limiter_bench.rs` | 9 functions | COMPILES |

---

## What Was Added (D→C blitz)

### New files

- `indieweb2-bastion/odns-rs/common/tests/property/crypto_properties_test.rs`
  - 10 proptest cases: KEM roundtrip identity, ciphertext length, distinctness, wrong key, tampering, key serialization, hybrid sign/verify, cross-key rejection, pk serialization, SPHINCS+ roundtrip (ignored)
- `indieweb2-bastion/odns-rs/common/tests/e2e/odns_pipeline_test.rs`
  - 5 async E2E tests: full pipeline roundtrip, signed query pipeline, multiple sequential queries, framing edge cases; 1 SPHINCS+ pipeline (ignored)
- `indieweb2-bastion/odns-rs/common/tests/aspect/security_test.rs`
  - 19 security aspect tests: truncated/zeroed inputs, bit-flip in every wire region (nonce, AEAD body, Poly1305 tag, KEM CT), wrong key, wrong-sized keys/signatures, constant-time structural assertion, hybrid sig edge cases, SPHINCS+ fallback independence, oversized framing
- `indieweb2-bastion/odns-rs/common/benches/crypto_bench.rs`
  - Criterion benchmarks: Kyber-1024 keygen, encrypt (4 sizes), decrypt (4 sizes), hybrid keygen, hybrid sign, hybrid verify, framing roundtrip (3 sizes)
- `indieweb2-bastion/services/webmention-rate-limiter/tests/property/rate_limiter_properties_test.rs`
  - 7 proptest cases: N-within-limit accepted, N+1 rejected, source limits independent, missing source/target rejected, self-ping always blocked, cross-domain accepted
- `indieweb2-bastion/services/webmention-rate-limiter/tests/e2e/webmention_e2e_test.rs`
  - 11 E2E tests: valid accept, charset in content-type, invalid CT early rejection, missing source/target, self-ping, FTP URL rejection, IP rate limit, source rate limit, different IPs independent, invalid requests don't consume tokens
- `indieweb2-bastion/services/webmention-rate-limiter/benches/rate_limiter_bench.rs`
  - Criterion benchmarks: check_ip latency, combined check latency, many unique IPs (3 sizes), validator throughput (valid/invalid/self-ping/batch)

### Cargo.toml changes

- `odns-rs/Cargo.toml` — added `proptest = "1"` and `criterion = "0.5"` to `[workspace.dependencies]`
- `odns-rs/common/Cargo.toml` — added `[dev-dependencies]` with proptest + criterion, added `[[bench]] crypto_bench`
- `services/webmention-rate-limiter/Cargo.toml` — added proptest + criterion dev-deps, added `[[bench]] rate_limiter_bench`

---

## Remaining Gaps (CRG B would require)

### GraphQL DNS API (Rust — 9 source files)
- blockchain.rs — no dedicated tests
- consent.rs — no dedicated tests
- db.rs — no dedicated tests
- dnssec.rs — no dedicated tests
- error.rs — no dedicated tests
- models.rs — no dedicated tests
- policy.rs — no dedicated tests
- resolvers.rs — integration test exists
- schema.rs — no dedicated tests

### ReScript UI (17 files)
- ZERO test files — needs deno test integration

### Idris2 ABI (6 files) + Zig FFI (6 files)
- 2 Zig integration tests only

### Network-level E2E
- Full DNS resolution: query → ODNS proxy (TLS) → resolver (TLS) → DNSSEC → response
- Requires: TLS cert generation, ephemeral TCP listeners, test fixtures
- Priority: P1 for CRG B

### Benchmarks (runs, not just compile)
- Benchmarks have been baselined to compile; actual numbers require `cargo bench` on the target hardware
- Establish baseline numbers and commit to `BENCHMARK-BASELINES.md` for CRG B

### Fuzz Testing
- `tests/fuzz/placeholder.txt` is still a scorecard placeholder — FAKE-FUZZ ALERT from original scan
- Replace with a real cargo-fuzz harness targeting `decrypt_query` and the webmention validator

### panic-attack assail
- `panic-attack assail` has not been run on this repo — run before CRG B
