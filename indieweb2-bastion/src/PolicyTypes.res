// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <jonathan.jewell@open.ac.uk>
//
// Policy type definitions matching Nickel schema (policy/curps/schema.ncl).

type capabilitySet = {
  maintainer: string,
  trusted_contributor: string,
  default_consent: string, /* map from "default-consent" */
}

type mutation = {
  name: string,
  description: string,
  approvals: int,
  timelock_hours: int,
}

type role = {
  name: string,
  members: array<string>,
  privileges: array<string>,
}

type route = {
  path: string,
  plane: string,
  methods: array<string>,
  guards: array<string>,
}

type consentDefaults = {
  telemetry: string,
  indexing: string,
}

type consentBinding = {
  name: string,
  manifest_ref: string,
  required: bool,
  defaults: consentDefaults,
}

type constraints = {
  require_mtls: bool,
  log_all_mutations: bool,
  max_rate_rpm: int,
}

type cryptoAlgorithm = {
  name: string,
  standard: string,
  status: string, /* "required" | "deprecated" | "terminated" */
}

type cryptoPolicy = {
  password_hashing: cryptoAlgorithm,
  general_hashing: cryptoAlgorithm,
  pq_signatures: cryptoAlgorithm,
  pq_key_exchange: cryptoAlgorithm,
  classical_sigs: cryptoAlgorithm,
  symmetric: cryptoAlgorithm,
  key_derivation: cryptoAlgorithm,
  rng: cryptoAlgorithm,
  database_hashing: cryptoAlgorithm,
  fallback_sig: cryptoAlgorithm,
  terminated: array<cryptoAlgorithm>,
}

type policy = {
  version: string,
  capabilities: capabilitySet,
  mutations: array<mutation>,
  roles: array<role>,
  routes: array<route>,
  consent_bindings: array<consentBinding>,
  constraints: constraints,
  crypto: option<cryptoPolicy>,
}
