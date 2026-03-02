// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <jonathan.jewell@open.ac.uk>
//
// DEPRECATED: Ed25519-only signing is deprecated per CRYPTO-POLICY.adoc (CPR-005).
// Target replacement: Rust implementation with Ed448+Dilithium5 hybrid signatures.
// This script remains for interim compatibility until Phase 4 migration (2026-06-01).
//
// Deno — Ed25519 sign policy JSON (pure JS)
// Usage: deno run --allow-read --allow-write --unstable crypto/sign_policy.js policy.json policy.sig

const [input, sigOut] = Deno.args;
if (!input || !sigOut) {
  console.error("Usage: sign_policy.js <policy.json> <policy.sig>");
  Deno.exit(2);
}

const data = await Deno.readFile(input);

// Generate ephemeral keypair (replace with persistent keys via env/keystore)
const keyPair = await crypto.subtle.generateKey({ name: "Ed25519" }, true, ["sign", "verify"]);

const signatureBuf = await crypto.subtle.sign("Ed25519", keyPair.privateKey, data);
const signature = new Uint8Array(signatureBuf);

// Write raw signature
await Deno.writeFile(sigOut, signature);

// Bundle with JWK public key for verification
const jwk = await crypto.subtle.exportKey("jwk", keyPair.publicKey);
const signatureB64 = btoa(String.fromCharCode(...signature));

await Deno.writeTextFile(
  sigOut + ".json",
  JSON.stringify({ alg: "Ed25519", jwk, signature: signatureB64 }, null, 2),
);

console.log("Signed:", sigOut, sigOut + ".json");
