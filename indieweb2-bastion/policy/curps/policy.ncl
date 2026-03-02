# IndieWeb2 CURPS policy — v0.1 strict
# SPDX-FileCopyrightText: 2025 Hyperpolymath
# SPDX-License-Identifier: PMPL-1.0-or-later

let schema = import "./schema.ncl"
let webmention = import "./webmention.ncl"

let policy = schema.Policy {
  version = "0.1.0",

  capabilities = {
    maintainer          = "stub",
    trusted_contributor = "stub",
    default-consent     = "stub",
  },

  mutations = [
    {
      name = "rotate_keys",
      description = "Rotate bastion, resolver, and SDP credentials",
      approvals = 2,
      timelock_hours = 24,
    },
    {
      name = "mutate_dns",
      description = "Apply GraphQL DNS mutations (records/zones)",
      approvals = 1,
      timelock_hours = 1,
    },
  ],

  roles = [
    {
      name = "maintainer",
      members = ["identity:alice", "identity:jonathan"],
      privileges = ["publish_manifest", "rotate_keys", "mutate_dns"],
    },
    {
      name = "trusted_contributor",
      members = [],
      privileges = ["publish_manifest"],
    },
  ],

  routes = [
    {
      path = "/.well-known",
      plane = "data",
      methods = ["GET"],
      guards = ["waf", "policy-gate"],
    },
    {
      path = "/odns",
      plane = "data",
      methods = ["GET","POST"],
      guards = ["mtls", "waf", "policy-gate"],
    },
    {
      path = "/graphql-dns",
      plane = "control",
      methods = ["POST"],
      guards = ["mtls", "policy-gate"],
    },
    # IndieWeb protocol routes
    {
      path = "/webmention",
      plane = "data",
      methods = ["POST"],
      guards = ["webmention-rate-limit", "waf", "policy-gate"],
    },
    {
      path = "/webmention.php",
      plane = "data",
      methods = ["POST"],
      guards = ["webmention-rate-limit", "waf", "policy-gate"],
    },
    {
      path = "/.well-known/webmention",
      plane = "data",
      methods = ["POST"],
      guards = ["webmention-rate-limit", "waf", "policy-gate"],
    },
  ],

  consent_bindings = [
    {
      name = "default-consent",
      manifest_ref = "ipfs://QmConsentManifest001",
      required = true,
      defaults = { telemetry = "off", indexing = "on" },
    },
  ],

  constraints = {
    require_mtls = true,
    log_all_mutations = true,
    max_rate_rpm = 120,
  },

  crypto = {
    password_hashing = {
      name = "Argon2id",
      standard = "RFC 9106 (512 MiB / 8 iter / 4 lanes)",
      status = 'required,
    },
    general_hashing = {
      name = "SHAKE3-512",
      standard = "FIPS 202 (SHA-3)",
      status = 'required,
    },
    pq_signatures = {
      name = "Dilithium5-AES / ML-DSA-87",
      standard = "FIPS 204",
      status = 'required,
    },
    pq_key_exchange = {
      name = "Kyber-1024 / ML-KEM-1024",
      standard = "FIPS 203",
      status = 'required,
    },
    classical_sigs = {
      name = "Ed448 + Dilithium5 hybrid",
      standard = "RFC 8032 + FIPS 204",
      status = 'required,
    },
    symmetric = {
      name = "XChaCha20-Poly1305",
      standard = "RFC 8439 (extended nonce)",
      status = 'required,
    },
    key_derivation = {
      name = "HKDF-SHAKE512",
      standard = "RFC 5869 + FIPS 202",
      status = 'required,
    },
    rng = {
      name = "ChaCha20-DRBG",
      standard = "NIST SP 800-90A",
      status = 'required,
    },
    database_hashing = {
      name = "BLAKE3 + SHAKE3-512",
      standard = "BLAKE3 spec + FIPS 202",
      status = 'required,
    },
    fallback_sig = {
      name = "SPHINCS+ / SLH-DSA",
      standard = "FIPS 205",
      status = 'required,
    },
    terminated = [
      { name = "Ed25519", standard = "Replaced by Ed448+Dilithium5", status = 'terminated },
      { name = "SHA-1", standard = "Collision attacks demonstrated", status = 'terminated },
      { name = "RSA", standard = "Quantum-vulnerable", status = 'terminated },
      { name = "X25519", standard = "Replaced by Kyber-1024", status = 'terminated },
      { name = "HKDF-SHA256", standard = "Replaced by HKDF-SHAKE512", status = 'terminated },
    ],
  },
}

export {
  policy,
  webmention_policy = webmention.webmention_policy,
}
