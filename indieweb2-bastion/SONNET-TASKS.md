# SONNET-TASKS: indieweb2-bastion

**Date:** 2026-02-12
**Auditor:** Claude Opus 4.6
**Honest Completion:** ~40%

The PROJECT_STATUS.adoc itself admits "Implementation: 0% (no source code)" but that is
outdated -- there IS real code now: a Rust GraphQL DNS API, a Rust webmention rate limiter,
a Deno consent API, Go oDNS proxy/resolver, ReScript policy gate, Motoko IC canisters,
Solidity/Vyper contracts, and a WordPress plugin. However, significant portions are stubs,
placeholders, or have critical gaps: DNSSEC signing is a placeholder, the oDNS components
are in a BANNED language (Go), browser extensions are identical 7-line stubs, mobile apps
are empty, the Justfile is a no-op skeleton, SQL injection exists in the DB layer, the
identity extraction in resolvers is hardcoded "identity:unknown", and the ERC20.sol contract
is a bare-bones token with no identity/registry logic matching the project's stated purpose.

---

## GROUND RULES FOR SONNET

1. Every task has a **Verification** block with runnable commands.
2. Do NOT create new files unless the task explicitly says to.
3. Do NOT refactor code that is not broken.
4. Do NOT change the license headers.
5. If a task says "line N", verify line N still matches before editing.
6. Build commands: `cargo build` (in graphql-dns-api/ or services/webmention-rate-limiter/),
   `npx rescript build` (in root for ReScript), `deno check` (for .ts files).
7. Test commands: `cargo test` (Rust crates), no test runner exists for ReScript or Deno
   consent API yet.
8. The oDNS proxy/resolver are Go code -- per language policy Go is BANNED and must be
   rewritten in Rust. This is a large task and may be deferred.

---

## TASK 1: Fix SQL Injection in `graphql-dns-api/src/db.rs` query_records()

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api/src/db.rs` lines 119-147

**Problem:**
The `query_records` function builds SQL queries using string formatting with unsanitized
user input:

```rust
conditions.push(format!("name = '{}'", name));   // line 130
conditions.push(format!("type = '{:?}'", record_type)); // line 134
```

A GraphQL query with `name: "'; DROP TABLE dns_records; --"` would inject SQL.

**What to do:**
Replace the string-interpolated query with SurrealDB's parameterized query binding
(`.bind()`) for the `name` and `type` fields. SurrealDB supports `$variable` syntax
for safe parameter binding, as already used in `get_dnssec_zone()` on line 171.

The fixed `query_records` should use:
```rust
let mut query = String::from("SELECT * FROM dns_records");
let mut conditions = Vec::new();
let mut bindings: Vec<(&str, serde_json::Value)> = Vec::new();

if let Some(ref name) = name {
    conditions.push("name = $name".to_string());
    bindings.push(("name", serde_json::Value::String(name.clone())));
}
// ... similar for record_type
```

**Verification:**
```bash
cd /var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api
cargo build 2>&1 | tail -5
cargo test 2>&1 | tail -20
# Verify no string formatting with user input in query_records:
grep -n "format!(\".*'{}'" src/db.rs
# Should return zero lines after fix
```

---

## TASK 2: Implement DNSSEC RRSIG Signing (Currently Returns "RRSIG_PLACEHOLDER")

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api/src/dnssec.rs` lines 96-107, 110-113

**Problem:**
`sign_record()` on line 96 returns `Ok("RRSIG_PLACEHOLDER".to_string())` -- it does not
actually sign anything. `verify_signature()` on line 110 returns `Ok(true)` unconditionally
without verifying. This means DNSSEC is non-functional: enabling DNSSEC generates keys but
records are never signed, and verification is a no-op.

**What to do:**
Implement actual Ed25519 signing using the `ring` crate (already a dependency):

1. `sign_record()` should:
   - Accept the ZSK private key bytes
   - Create an `Ed25519KeyPair` from the key bytes
   - Sign the canonical form of the record data
   - Base64-encode the signature
   - Return the signature string (not a full RRSIG record -- that requires trust-dns-server)

2. `verify_signature()` should:
   - Decode the base64 signature
   - Use `ring::signature::UnparsedPublicKey` with `ED25519` algorithm
   - Verify against the record data
   - Return the actual verification result

Also store the ZSK private key alongside the public key in `DNSSECZone` (currently only
public keys are stored, so signing is impossible).

**Verification:**
```bash
cd /var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api
cargo build 2>&1 | tail -5
cargo test -- dnssec 2>&1
# Verify placeholder is gone:
grep -n "RRSIG_PLACEHOLDER" src/dnssec.rs
# Should return zero lines
grep -n "Ok(true)" src/dnssec.rs
# Should not appear in verify_signature
```

---

## TASK 3: Fix Hardcoded "identity:unknown" in GraphQL Resolvers

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api/src/resolvers.rs` lines 162-164, 311-315, 330-332

**Problem:**
Identity extraction in three places falls back to `"identity:unknown"`:

```rust
let identity = ctx.data_opt::<String>()
    .cloned()
    .unwrap_or_else(|| "identity:unknown".to_string());
```

This means:
- `create_dns_record` calls consent check with "identity:unknown" (line 167)
- `propose_mutation` checks privileges for "identity:unknown" (line 317)
- `approve_mutation` approves as "identity:unknown" (line 335)

Anyone can bypass consent and governance because the identity is never actually extracted
from the request.

**What to do:**
1. Add an authentication middleware or extractor that reads identity from:
   - mTLS client certificate CN/SAN (when `require_mtls` is true in policy)
   - Authorization header (Bearer token or API key)
   - X-Identity header (for development only, gated behind a config flag)

2. Store the identity in GraphQL context data so resolvers can retrieve it.

3. When no identity can be determined and the operation requires one, return an
   `Err("Authentication required")` instead of silently using "identity:unknown".

4. At minimum, for `create_dns_record`, `propose_mutation`, and `approve_mutation`,
   fail with an explicit error when identity is "identity:unknown".

**Verification:**
```bash
cd /var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api
cargo build 2>&1 | tail -5
# Verify "identity:unknown" no longer silently used for critical operations:
grep -c "identity:unknown" src/resolvers.rs
# Should be 0 or only in a comment/error message
```

---

## TASK 4: Implement execute_mutation (Currently a No-Op)

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api/src/resolvers.rs` lines 340-354

**Problem:**
The `execute_mutation` resolver contains a TODO comment on line 350:
```rust
// TODO: Actually execute the mutation based on proposal.payload
// For now, just mark as executed
```

The mutation governance system (propose -> approve -> execute) is half-implemented: proposals
can be created and approved, but execution does nothing. The approved payload is ignored.

**What to do:**
Implement payload dispatch based on `proposal.mutation_name`:

1. For `"mutate_dns"`: parse `proposal.payload` as a DNS record operation (create/update/delete)
   and call the appropriate `db` method.
2. For `"rotate_keys"`: call `dnssec_manager.generate_keys()` and update the DNSSEC zone.
3. For unknown mutations: return an error rather than silently marking as executed.

Add a match statement that dispatches based on `mutation_name` and processes the payload.

**Verification:**
```bash
cd /var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api
cargo build 2>&1 | tail -5
# Verify TODO is resolved:
grep -n "TODO.*execute the mutation" src/resolvers.rs
# Should return zero lines
```

---

## TASK 5: Fix IPv6 Reverse DNS Implementation

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api/src/resolvers.rs` lines 395-408

**Problem:**
The `ip_to_reverse_name()` function has a broken IPv6 implementation on line 399:
```rust
Ok(format!("{}.ip6.arpa", ip.replace(':', ".")))
```

This produces `2001.db8..1.ip6.arpa` for `2001:db8::1`, which is completely wrong. The
correct IPv6 reverse DNS format requires:
1. Expanding the address to full 32 hex nibbles (e.g., `20010db8000000000000000000000001`)
2. Reversing the nibbles
3. Separating with dots
4. Appending `.ip6.arpa`

So `2001:db8::1` should become `1.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.8.b.d.0.1.0.0.2.ip6.arpa`.

**What to do:**
1. Parse the IPv6 address using `std::net::Ipv6Addr`.
2. Get the 16 octets via `.octets()`.
3. Convert each octet to two hex nibbles.
4. Reverse the nibble array.
5. Join with dots and append `.ip6.arpa`.

**Verification:**
```bash
cd /var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api
cargo build 2>&1 | tail -5
# Add a unit test or run:
cargo test -- reverse 2>&1
# Manually verify the function no longer uses ip.replace(':', "."):
grep -n "replace.*':'" src/resolvers.rs
# Should return zero lines
```

---

## TASK 6: Replace Stub Justfile with Working Build/Test Commands

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/Justfile` (all lines)
- `/var/mnt/eclipse/repos/indieweb2-bastion/justfile` (all lines -- note: TWO justfiles exist)

**Problem:**
Both `Justfile` (capital J) and `justfile` (lowercase j) exist. The capital-J `Justfile`
is an RSR template stub where every recipe just echoes a message:
```
build:
    @echo "Building..."
test:
    @echo "Testing..."
```

The lowercase `justfile` has the same content (identical file). Neither actually builds
or tests anything. Meanwhile `test/Justfile` has real recipes that reference the actual
toolchain (nickel, rescript, deno).

Additionally, the SPDX header in both Justfiles says `PMPL-1.0-or-later` which violates
the license policy (should be `PMPL-1.0-or-later`).

**What to do:**
1. Delete one of the two duplicate justfiles (keep `justfile` lowercase per convention).
2. Replace the stub recipes with actual commands:
   - `build`: `cd graphql-dns-api && cargo build` + `cd services/webmention-rate-limiter && cargo build`
   - `test`: `cd graphql-dns-api && cargo test` + `cd services/webmention-rate-limiter && cargo test`
   - `lint`: `cd graphql-dns-api && cargo clippy` + `cd services/webmention-rate-limiter && cargo clippy`
   - `fmt`: `cd graphql-dns-api && cargo fmt` + `cd services/webmention-rate-limiter && cargo fmt`
   - `clean`: `cd graphql-dns-api && cargo clean` + `cd services/webmention-rate-limiter && cargo clean`
3. Fix the SPDX header from `PMPL-1.0-or-later` to `PMPL-1.0-or-later`.
4. Integrate the `test/Justfile` recipes into the main justfile as sub-recipes.

**Verification:**
```bash
cd /var/mnt/eclipse/repos/indieweb2-bastion
ls -la justfile Justfile 2>&1
# Should show only one justfile after fix
just --list 2>&1 | head -20
# Should show real recipes, not stubs
grep "SPDX" justfile
# Should show PMPL-1.0-or-later, NOT PMPL-1.0-or-later
```

---

## TASK 7: Rewrite oDNS Proxy/Resolver from Go to Rust (BANNED Language)

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/odns-proxy/main.go` (310 lines)
- `/var/mnt/eclipse/repos/indieweb2-bastion/odns-proxy/go.mod`
- `/var/mnt/eclipse/repos/indieweb2-bastion/odns-resolver/main.go` (278 lines)
- `/var/mnt/eclipse/repos/indieweb2-bastion/odns-resolver/go.mod`

**Problem:**
Go is a BANNED language per the `.claude/CLAUDE.md` language policy. The replacement is Rust.
Both the oDNS proxy and resolver are ~300 lines of Go each. They implement:

- HPKE encryption/decryption (X25519 + ChaCha20-Poly1305) per RFC 9180
- DNS over TLS (DoT) listening on port 853
- DNS query forwarding
- Key generation and rotation

**What to do:**
1. Create `odns-proxy-rs/` and `odns-resolver-rs/` as Rust crates (or a single `odns/` workspace).
2. Use Rust crates: `hpke` (or `rust-hpke`), `trust-dns-client`/`trust-dns-server`, `tokio`,
   `tokio-rustls` for TLS.
3. Port the Go logic 1:1 into Rust, keeping the same CLI flags and behavior.
4. Add `Cargo.toml` with appropriate dependencies.
5. Remove the Go directories after the Rust versions compile and pass basic tests.

This is a large task and may span multiple sessions. At minimum, create the Rust project
skeleton with TODO stubs that compile.

**Verification:**
```bash
# After full port:
cd /var/mnt/eclipse/repos/indieweb2-bastion/odns-proxy-rs
cargo build 2>&1 | tail -5
cd /var/mnt/eclipse/repos/indieweb2-bastion/odns-resolver-rs
cargo build 2>&1 | tail -5
# Verify Go code removed:
find /var/mnt/eclipse/repos/indieweb2-bastion -name "*.go" | wc -l
# Should be 0
```

---

## TASK 8: Implement Browser Extensions (All Four Are Identical 7-Line Stubs)

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/products/extensions/chrome/src/content.js`
- `/var/mnt/eclipse/repos/indieweb2-bastion/products/extensions/firefox/src/content.js`
- `/var/mnt/eclipse/repos/indieweb2-bastion/products/extensions/edge/src/content.js`
- `/var/mnt/eclipse/repos/indieweb2-bastion/products/extensions/safari/src/content.js`

**Problem:**
All four browser extension content scripts are identical 7-line stubs that only check
for a non-existent `window.edgeLLM` or `window.browserLLM` API and log whether it exists:

```javascript
(async () => {
  const nativeLLM = typeof window !== 'undefined' && (window.edgeLLM || window.browserLLM);
  console.log('LLM available:', !!nativeLLM);
})();
```

This has nothing to do with IndieWeb2 or consent management. The extensions should at
minimum:
- Read the page's Webmention `<link>` tag
- Display consent preferences from the consent API
- Allow users to send/receive webmentions

**What to do:**
1. Implement a minimal content script that:
   - Detects `<link rel="webmention">` on the page
   - Queries the consent API for the current user's preferences
   - Shows a small badge/icon indicating webmention support
2. Use a shared source file for common logic, with browser-specific manifest differences.
3. Note: Per language policy, these should use JavaScript (not TypeScript) since they are
   browser extensions where ReScript compilation is not practical for content scripts.

**Verification:**
```bash
# Verify the extensions are no longer identical stubs:
for ext in chrome firefox edge safari; do
  wc -l /var/mnt/eclipse/repos/indieweb2-bastion/products/extensions/$ext/src/content.js
done
# Each should be > 7 lines
# Verify no LLM detection code:
grep -l "edgeLLM" /var/mnt/eclipse/repos/indieweb2-bastion/products/extensions/*/src/content.js
# Should return zero files
```

---

## TASK 9: Implement IPFS Rehydration (Currently a 9-Line Stub)

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/deno/ipfs/rehydrate.js` (9 lines)

**Problem:**
The rehydration script is a stub that prints "Would fetch snapshot from <cid>" and exits:
```javascript
console.log("Would fetch snapshot from", cid);
// TODO: ipfs cat + surreal import
```

This is referenced in `test/Justfile` as part of the test pipeline.

**What to do:**
1. Implement IPFS fetch using `Deno.Command("ipfs", { args: ["cat", cid] })`.
2. Pipe the output to SurrealDB import using `Deno.Command("surreal", { args: ["import", ...] })`.
3. Add error handling for:
   - IPFS daemon not running
   - Invalid CID
   - SurrealDB connection failure
4. Accept optional `--surrealdb-url` flag.

**Verification:**
```bash
# Verify TODO is resolved:
grep -n "TODO" /var/mnt/eclipse/repos/indieweb2-bastion/deno/ipfs/rehydrate.js
# Should return zero lines
# Verify it handles missing args:
cd /var/mnt/eclipse/repos/indieweb2-bastion
deno run deno/ipfs/rehydrate.js 2>&1
# Should print usage and exit 2
```

---

## TASK 10: Fix Stub ERC20 Contract -- Should Be Identity Registry

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/contracts/ERC20.sol` (22 lines)

**Problem:**
The Solidity contract is a minimal ERC20 token ("IndieWeb2 Token") with only `transfer()`.
Per PROJECT_STATUS.adoc, the project needs an `IdentityRegistry.sol` for on-chain identity
management. The current contract:
- Has no `approve`, `transferFrom`, or `allowance` (incomplete ERC20)
- Has no identity registration, lookup, or revocation
- Has no events (violates ERC20 spec)
- Missing `Transfer` event emission
- Missing reentrancy protection

The Vyper `Registry.vy` is equally minimal -- 19 lines with owner-only `set_record`.

**What to do:**
Either:
A) If an ERC20 token is needed, use OpenZeppelin's ERC20 as a base and add proper events,
   approve/transferFrom, and access control.
B) If an identity registry is needed (per PROJECT_STATUS.adoc), replace with an
   `IdentityRegistry.sol` that implements:
   - `register(address, string calldata dnsRecord)` -- register identity
   - `lookup(address) -> string` -- look up identity
   - `revoke(address)` -- revoke identity
   - Events: `IdentityRegistered`, `IdentityRevoked`
   - Access control via OpenZeppelin Ownable or custom roles

Option B aligns with the project's stated purpose.

**Verification:**
```bash
# After implementation, check for events and proper functions:
grep -c "event" /var/mnt/eclipse/repos/indieweb2-bastion/contracts/ERC20.sol
# Should be >= 2
grep -c "function" /var/mnt/eclipse/repos/indieweb2-bastion/contracts/ERC20.sol
# Should be >= 4
```

---

## TASK 11: Fix CURPS Policy Capabilities (All Set to "stub")

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/policy/curps/policy.ncl` lines 12-14

**Problem:**
All three capability definitions are set to the string literal `"stub"`:
```nickel
capabilities = {
    maintainer          = "stub",
    trusted_contributor = "stub",
    default-consent     = "stub",
},
```

The capability documents in `docs/capabilities/` are also stubs (single-line files).

**What to do:**
1. Replace `"stub"` values with actual capability URIs or file references:
   - `maintainer = "docs/capabilities/maintainer.adoc"`
   - `trusted_contributor = "docs/capabilities/trusted_contributor.adoc"`
   - `default-consent = "docs/capabilities/default-consent.adoc"`
2. Flesh out the three capability documents in `docs/capabilities/` with actual definitions:
   - What permissions each capability grants
   - How capabilities are assigned/revoked
   - Integration with the RBAC system in `policy.rs`

**Verification:**
```bash
grep -c '"stub"' /var/mnt/eclipse/repos/indieweb2-bastion/policy/curps/policy.ncl
# Should be 0
# Verify capability docs are no longer stubs:
for f in maintainer.adoc trusted_contributor.adoc default-consent.adoc; do
  wc -l /var/mnt/eclipse/repos/indieweb2-bastion/docs/capabilities/$f
done
# Each should be > 1 line
```

---

## TASK 12: Fix PWA Service Worker (2-Line No-Op)

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/products/pwa/src/service-worker.js` (2 lines)

**Problem:**
The service worker is:
```javascript
self.addEventListener('install', e => self.skipWaiting());
self.addEventListener('activate', e => clients.claim());
```

It registers and activates but does nothing -- no caching, no offline support, no
push notifications. For a PWA to be functional, the service worker needs a caching strategy.

**What to do:**
1. Add a cache-first strategy for static assets (HTML, CSS, JS, images).
2. Add a network-first strategy for API calls (`/graphql`, `/consent`).
3. Handle the `fetch` event.
4. Define a cache version string for cache invalidation.
5. Add an `activate` handler that cleans up old caches.

**Verification:**
```bash
grep -c "addEventListener.*fetch" /var/mnt/eclipse/repos/indieweb2-bastion/products/pwa/src/service-worker.js
# Should be >= 1
wc -l /var/mnt/eclipse/repos/indieweb2-bastion/products/pwa/src/service-worker.js
# Should be > 2
```

---

## TASK 13: Fix SBOM Generator (Returns Hardcoded Stub JSON)

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/sbom/generate.sh`

**Problem:**
The SBOM generator outputs a hardcoded stub:
```bash
echo '{"sbom":"stub"}' > "$OUT"
```

This produces a fake SBOM that provides no supply chain information.

**What to do:**
1. Use `cargo sbom` or `cyclonedx-rust-cargo` for the Rust crates.
2. Use `deno info --json` for the Deno consent API dependencies.
3. Merge outputs into a single CycloneDX or SPDX SBOM.
4. Include the ReScript/npm dependencies from `package-lock.json`.

**Verification:**
```bash
grep -c '"stub"' /var/mnt/eclipse/repos/indieweb2-bastion/sbom/generate.sh
# Should be 0
```

---

## TASK 14: Fix Motoko Canister Compilation Errors

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/protocols/icp/canisters/consent/Consent.mo`

**Problem:**
The `Consent.mo` canister has compilation issues:
1. Line 6: `stable var store : Trie = Trie.Empty;` -- `Trie` is defined as a public type
   inside the actor, but `Trie.Empty` is not valid Motoko syntax. In Motoko, variant
   construction uses `#Empty`, not `Trie.Empty`.
2. Line 21: `Array.append` is used but `Array` module is not imported.
3. The custom Trie implementation (lines 15-18) defines `Trie` as a public type with
   `Empty` and `Node` constructors, but the syntax `{ Empty : (); Node : { ... } }` is
   not correct Motoko variant syntax.

**What to do:**
1. Fix the Trie type to use proper Motoko variant syntax:
   ```motoko
   public type Trie = {
     #Empty;
     #Node : { key : Text; value : Manifest; next : Trie };
   };
   ```
2. Fix variant construction: `#Empty` instead of `Trie.Empty`.
3. Import `Array` module: `import Array "mo:base/Array";`
4. Fix `putTrie` and `lookupTrie` to use `#Empty` and `#Node(n)` pattern matching.

**Verification:**
```bash
cd /var/mnt/eclipse/repos/indieweb2-bastion/protocols/icp
# If dfx is installed:
dfx build 2>&1 | tail -10
# Otherwise check syntax manually:
grep -n "Trie.Empty\|Empty : ()" canisters/consent/Consent.mo
# Should return zero lines after fix
```

---

## TASK 15: Add Tests for Consent API (Deno Service Has Zero Tests)

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/services/consent-api/mod.ts` (255 lines)

**Problem:**
The consent API is a complete Deno HTTP service with SurrealDB integration but has
zero test files. The handler function, consent storage, operation checking, and revocation
are all untested.

**What to do:**
1. Create `/var/mnt/eclipse/repos/indieweb2-bastion/services/consent-api/mod_test.ts`.
2. Test the HTTP handler for each route:
   - POST /consent -- stores preferences, returns 201
   - GET /consent/:identity -- returns consent record or 404
   - POST /consent/:identity/check -- checks operation permission
   - DELETE /consent/:identity -- revokes consent
   - GET /health -- returns OK
3. Test `checkConsentForOperation()` default behavior (no consent record).
4. Test validation: missing required fields returns 400.
5. Use Deno's built-in test runner and `Deno.test()`.
6. Mock SurrealDB or use in-memory for tests.

**Verification:**
```bash
cd /var/mnt/eclipse/repos/indieweb2-bastion/services/consent-api
deno test mod_test.ts 2>&1 | tail -20
```

---

## TASK 16: Fix WordPress Plugin `add_user_meta_field` Call (Undefined Function)

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/wordpress/indieweb2-consent.php` line 65-68

**Problem:**
The `register_settings()` method calls `add_user_meta_field()` on lines 65-68:
```php
add_user_meta_field('indieweb2_telemetry', 'off');
add_user_meta_field('indieweb2_indexing', 'on');
add_user_meta_field('indieweb2_webmentions', 'on');
add_user_meta_field('indieweb2_dns_operations', 'off');
```

`add_user_meta_field` is not a WordPress API function. This will cause a fatal PHP error
when the plugin is activated. The correct approach is to use `register_meta()` or handle
defaults in `get_user_meta()` calls.

**What to do:**
Replace the four `add_user_meta_field()` calls with either:
- `register_meta('user', 'indieweb2_telemetry', ['default' => 'off', 'type' => 'string', ...])` (WP 4.6+)
- Or simply remove them, since defaults are already handled in `render_user_consent_page()`
  via the `?: 'off'` / `?: 'on'` fallbacks on lines 180-183.

**Verification:**
```bash
grep -n "add_user_meta_field" /var/mnt/eclipse/repos/indieweb2-bastion/wordpress/indieweb2-consent.php
# Should return zero lines after fix
# Verify PHP syntax:
php -l /var/mnt/eclipse/repos/indieweb2-bastion/wordpress/indieweb2-consent.php 2>&1
# Should say "No syntax errors detected"
```

---

## TASK 17: Fix CorsLayer::permissive() in Production GraphQL Server

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api/src/main.rs` line 147

**Problem:**
The GraphQL server uses `CorsLayer::permissive()` which allows ANY origin to make
requests. For a DNS API with mutation capabilities and blockchain anchoring, this is
a security vulnerability. Any website could send GraphQL mutations to the API.

**What to do:**
1. Read allowed origins from environment variable `CORS_ORIGINS` (comma-separated).
2. Default to `http://localhost:8080` for development.
3. Use `CorsLayer::new().allow_origin(origins).allow_methods([Method::GET, Method::POST]).allow_headers([header::CONTENT_TYPE])`.
4. Keep `CorsLayer::permissive()` only when `CORS_ORIGINS` is set to `"*"` explicitly.

**Verification:**
```bash
cd /var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api
cargo build 2>&1 | tail -5
grep -n "permissive" src/main.rs
# Should return zero lines (or only in a conditional "*" branch)
```

---

## TASK 18: Fix `base64::encode` Deprecation in dnssec.rs

**Files:**
- `/var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api/src/dnssec.rs` lines 35, 44

**Problem:**
`base64::encode()` was deprecated in base64 crate 0.22 (which is the version in
Cargo.toml). The current code:
```rust
let ksk_public = base64::encode(ksk_pair.public_key().as_ref());
let zsk_public = base64::encode(zsk_pair.public_key().as_ref());
```

This will produce a deprecation warning and may fail in future base64 versions.

**What to do:**
Replace with the new API:
```rust
use base64::{Engine, engine::general_purpose::STANDARD};
let ksk_public = STANDARD.encode(ksk_pair.public_key().as_ref());
let zsk_public = STANDARD.encode(zsk_pair.public_key().as_ref());
```

**Verification:**
```bash
cd /var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api
cargo build 2>&1 | grep -i "deprecat"
# Should return zero lines
grep "base64::encode" src/dnssec.rs
# Should return zero lines
```

---

---

## FINAL VERIFICATION

After completing all tasks, run the following comprehensive check:

```bash
echo "=== 1. Rust crates build ==="
cd /var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api && cargo build 2>&1 | tail -3
cd /var/mnt/eclipse/repos/indieweb2-bastion/services/webmention-rate-limiter && cargo build 2>&1 | tail -3

echo "=== 2. Rust tests pass ==="
cd /var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api && cargo test 2>&1 | tail -5
cd /var/mnt/eclipse/repos/indieweb2-bastion/services/webmention-rate-limiter && cargo test 2>&1 | tail -5

echo "=== 3. No SQL injection ==="
grep -rn "format!(.*'{}'.*)" /var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api/src/db.rs | grep -v "^$" || echo "PASS: No format-string SQL"

echo "=== 4. No stubs in production code ==="
grep -rn '"stub"\|RRSIG_PLACEHOLDER\|identity:unknown' \
  /var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api/src/ \
  /var/mnt/eclipse/repos/indieweb2-bastion/policy/curps/policy.ncl \
  2>/dev/null || echo "PASS: No stubs"

echo "=== 5. No banned languages ==="
find /var/mnt/eclipse/repos/indieweb2-bastion -name "*.go" -not -path "*/.git/*" | wc -l
# Should be 0 after TASK 7

echo "=== 6. License headers correct ==="
grep -rn "AGPL-3.0" /var/mnt/eclipse/repos/indieweb2-bastion/justfile /var/mnt/eclipse/repos/indieweb2-bastion/Justfile 2>/dev/null || echo "PASS: No AGPL in justfiles"

echo "=== 7. WordPress plugin syntax ==="
php -l /var/mnt/eclipse/repos/indieweb2-bastion/wordpress/indieweb2-consent.php 2>&1

echo "=== 8. No TODO/FIXME in critical paths ==="
grep -rn "TODO\|FIXME" \
  /var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api/src/resolvers.rs \
  /var/mnt/eclipse/repos/indieweb2-bastion/graphql-dns-api/src/dnssec.rs \
  /var/mnt/eclipse/repos/indieweb2-bastion/deno/ipfs/rehydrate.js \
  2>/dev/null || echo "PASS: No TODOs in critical paths"

echo "=== 9. Deno consent API tests ==="
cd /var/mnt/eclipse/repos/indieweb2-bastion/services/consent-api
ls mod_test.ts 2>/dev/null && deno test mod_test.ts 2>&1 | tail -5 || echo "FAIL: No test file"

echo "=== DONE ==="
```
