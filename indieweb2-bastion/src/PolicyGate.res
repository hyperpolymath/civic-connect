// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <jonathan.jewell@open.ac.uk>
//
// Policy-gate validation engine (CURPS — Consent-driven URIs with Policy State).
//
// Validates a decoded policy against invariants from CRYPTO-POLICY.adoc and
// the Nickel schema (policy/curps/schema.ncl). Each validator returns an array
// of human-readable error strings; an empty array means the check passed.

/// Known guard names that routes may reference.
let knownGuards = ["mtls", "waf", "policy-gate", "webmention-rate-limit"]

/// Known privilege names for RBAC roles.
let knownPrivileges = ["publish_manifest", "rotate_keys", "mutate_dns"]

/// Algorithms that MUST NOT appear anywhere in an active crypto policy.
let terminatedAlgorithms = [
  "Ed25519", "SHA-1", "MD5", "RSA", "X25519",
  "HKDF-SHA256", "SHA-256", "ChaCha20-Poly1305",
]

let validateBasics = (p: PolicyTypes.policy): array<string> => {
  let errs = Belt.MutableQueue.make()
  if p.version == "" { Belt.MutableQueue.add(errs, "version must be non-empty") }
  p.routes->Belt.Array.forEach(r =>
    if r.plane == "control" && !(r.guards->Belt.Array.some(g => g == "mtls")) {
      Belt.MutableQueue.add(errs, "control-plane route requires mtls: " ++ r.path)
    }
  )
  if p.constraints.max_rate_rpm <= 0 {
    Belt.MutableQueue.add(errs, "max_rate_rpm must be > 0")
  }
  Belt.MutableQueue.toArray(errs)
}

let validateParadoxExclusion = (p: PolicyTypes.policy): array<string> => {
  let errs = Belt.MutableQueue.make()
  p.roles->Belt.Array.forEach(role =>
    if role.name == "trusted_contributor" && role.privileges->Belt.Array.some(pr => pr == "rotate_keys") {
      Belt.MutableQueue.add(errs, "trusted_contributor must not have rotate_keys")
    }
  )
  Belt.MutableQueue.toArray(errs)
}

/// Capabilities must not be "stub" — they should reference real files or URIs.
let validateCapabilities = (p: PolicyTypes.policy): array<string> => {
  let errs = Belt.MutableQueue.make()
  let caps = p.capabilities
  if caps.maintainer == "stub" {
    Belt.MutableQueue.add(errs, "capabilities.maintainer is stub — must reference a real file")
  }
  if caps.trusted_contributor == "stub" {
    Belt.MutableQueue.add(errs, "capabilities.trusted_contributor is stub — must reference a real file")
  }
  if caps.default_consent == "stub" {
    Belt.MutableQueue.add(errs, "capabilities.default-consent is stub — must reference a real file")
  }
  Belt.MutableQueue.toArray(errs)
}

/// All guard names referenced by routes must be in the known set.
let validateRouteGuards = (p: PolicyTypes.policy): array<string> => {
  let errs = Belt.MutableQueue.make()
  p.routes->Belt.Array.forEach(r =>
    r.guards->Belt.Array.forEach(g =>
      if !(knownGuards->Belt.Array.some(kg => kg == g)) {
        Belt.MutableQueue.add(errs, "unknown guard '" ++ g ++ "' on route " ++ r.path)
      }
    )
  )
  Belt.MutableQueue.toArray(errs)
}

/// Mutation approval count must be positive and not exceed the maintainer count.
let validateMutationApprovals = (p: PolicyTypes.policy): array<string> => {
  let errs = Belt.MutableQueue.make()
  let maintainerCount =
    p.roles
    ->Belt.Array.keepMap(r => r.name == "maintainer" ? Some(Belt.Array.length(r.members)) : None)
    ->Belt.Array.get(0)
    ->Belt.Option.getWithDefault(0)
  p.mutations->Belt.Array.forEach(m => {
    if m.approvals <= 0 {
      Belt.MutableQueue.add(errs, "mutation '" ++ m.name ++ "' must require >= 1 approval")
    }
    if m.approvals > maintainerCount && maintainerCount > 0 {
      Belt.MutableQueue.add(
        errs,
        "mutation '" ++ m.name ++ "' requires " ++ Belt.Int.toString(m.approvals) ++
        " approvals but only " ++ Belt.Int.toString(maintainerCount) ++ " maintainers exist",
      )
    }
    if m.timelock_hours <= 0 {
      Belt.MutableQueue.add(errs, "mutation '" ++ m.name ++ "' must have timelock_hours > 0")
    }
  })
  Belt.MutableQueue.toArray(errs)
}

/// All role privileges must be in the known set.
let validateRolePrivileges = (p: PolicyTypes.policy): array<string> => {
  let errs = Belt.MutableQueue.make()
  p.roles->Belt.Array.forEach(role =>
    role.privileges->Belt.Array.forEach(priv =>
      if !(knownPrivileges->Belt.Array.some(kp => kp == priv)) {
        Belt.MutableQueue.add(errs, "unknown privilege '" ++ priv ++ "' in role " ++ role.name)
      }
    )
  )
  Belt.MutableQueue.toArray(errs)
}

/// Consent bindings with required=true must have a valid manifest_ref.
let validateConsentBindings = (p: PolicyTypes.policy): array<string> => {
  let errs = Belt.MutableQueue.make()
  p.consent_bindings->Belt.Array.forEach(cb => {
    if cb.required && cb.manifest_ref == "" {
      Belt.MutableQueue.add(errs, "consent binding '" ++ cb.name ++ "' is required but has empty manifest_ref")
    }
    let tel = cb.defaults.telemetry
    if tel != "on" && tel != "off" {
      Belt.MutableQueue.add(errs, "consent binding '" ++ cb.name ++ "' has invalid telemetry default: " ++ tel)
    }
    let idx = cb.defaults.indexing
    if idx != "on" && idx != "off" {
      Belt.MutableQueue.add(errs, "consent binding '" ++ cb.name ++ "' has invalid indexing default: " ++ idx)
    }
  })
  Belt.MutableQueue.toArray(errs)
}

/// If a crypto policy is present, verify no active algorithm uses a terminated name.
let validateCryptoCompliance = (p: PolicyTypes.policy): array<string> => {
  let errs = Belt.MutableQueue.make()
  switch p.crypto {
  | None =>
    Belt.MutableQueue.add(errs, "crypto policy section is missing — CRYPTO-POLICY.adoc requires it")
  | Some(cp) =>
    // Check each active algorithm is not in the terminated list
    let activeAlgos = [
      ("password_hashing", cp.password_hashing),
      ("general_hashing", cp.general_hashing),
      ("pq_signatures", cp.pq_signatures),
      ("pq_key_exchange", cp.pq_key_exchange),
      ("classical_sigs", cp.classical_sigs),
      ("symmetric", cp.symmetric),
      ("key_derivation", cp.key_derivation),
      ("rng", cp.rng),
      ("database_hashing", cp.database_hashing),
      ("fallback_sig", cp.fallback_sig),
    ]
    activeAlgos->Belt.Array.forEach(((field, algo)) => {
      if algo.status == "terminated" {
        Belt.MutableQueue.add(errs, "crypto." ++ field ++ " (" ++ algo.name ++ ") has status 'terminated'")
      }
      if terminatedAlgorithms->Belt.Array.some(t => t == algo.name) {
        Belt.MutableQueue.add(
          errs,
          "crypto." ++ field ++ " uses terminated algorithm: " ++ algo.name,
        )
      }
    })
    // Verify all terminated entries are actually marked terminated
    cp.terminated->Belt.Array.forEach(algo =>
      if algo.status != "terminated" {
        Belt.MutableQueue.add(
          errs,
          "terminated list entry '" ++ algo.name ++ "' has status '" ++ algo.status ++ "' (expected 'terminated')",
        )
      }
    )
  }
  Belt.MutableQueue.toArray(errs)
}

/// All routes should have the policy-gate guard.
let validatePolicyGatePresence = (p: PolicyTypes.policy): array<string> => {
  let errs = Belt.MutableQueue.make()
  p.routes->Belt.Array.forEach(r =>
    if !(r.guards->Belt.Array.some(g => g == "policy-gate")) {
      Belt.MutableQueue.add(errs, "route " ++ r.path ++ " is missing 'policy-gate' guard")
    }
  )
  Belt.MutableQueue.toArray(errs)
}

/// Aggregate all validators.
let validateAll = (p: PolicyTypes.policy): array<string> =>
  Belt.Array.concatMany([
    validateBasics(p),
    validateParadoxExclusion(p),
    validateCapabilities(p),
    validateRouteGuards(p),
    validateMutationApprovals(p),
    validateRolePrivileges(p),
    validateConsentBindings(p),
    validateCryptoCompliance(p),
    validatePolicyGatePresence(p),
  ])
