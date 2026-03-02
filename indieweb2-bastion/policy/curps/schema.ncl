# SPDX-License-Identifier: PMPL-1.0-or-later
# CURPS Policy contracts — strict

let String = String
let Bool   = Bool
let Num    = Number

contract CapabilitySet = {
  maintainer          | String
  trusted_contributor | String
  default-consent     | String
}

contract Mutation = {
  name           | String
  description    | String
  approvals      | Num
  timelock_hours | Num
}

contract Role = {
  name        | String
  members     | Array(String)
  privileges  | Array(String)
}

contract Route = {
  path     | String
  plane    | String
  methods  | Array(String)
  guards   | Array(String)
}

contract ConsentDefaults = {
  telemetry | String
  indexing  | String
}

contract ConsentBinding = {
  name         | String
  manifest_ref | String
  required     | Bool
  defaults     | ConsentDefaults
}

contract Constraints = {
  require_mtls      | Bool
  log_all_mutations | Bool
  max_rate_rpm      | Num
}

# Webmention-specific route constraints for ingress hardening
contract WebmentionConstraints = {
  max_rate_rpm            | Num
  max_rate_per_source     | Num
  require_content_type    | Array(String)
  require_source_target   | Bool
  block_self_ping         | Bool
  cooldown_on_burst_ms    | Num
}

contract WebmentionPolicy = {
  routes      | Array(String)
  constraints | WebmentionConstraints
  enabled     | Bool
}

contract CryptoAlgorithm = {
  name     | String,
  standard | String,
  status   | [| 'required, 'deprecated, 'terminated |],
}

contract CryptoPolicy = {
  password_hashing  | CryptoAlgorithm,
  general_hashing   | CryptoAlgorithm,
  pq_signatures     | CryptoAlgorithm,
  pq_key_exchange   | CryptoAlgorithm,
  classical_sigs    | CryptoAlgorithm,
  symmetric         | CryptoAlgorithm,
  key_derivation    | CryptoAlgorithm,
  rng               | CryptoAlgorithm,
  database_hashing  | CryptoAlgorithm,
  fallback_sig      | CryptoAlgorithm,
  terminated        | Array CryptoAlgorithm,
}

contract Policy = {
  version          | String
  capabilities     | CapabilitySet
  mutations        | Array(Mutation)
  roles            | Array(Role)
  routes           | Array(Route)
  consent_bindings | Array(ConsentBinding)
  constraints      | Constraints
  crypto           | optional CryptoPolicy
}

export {
  CapabilitySet,
  Mutation,
  Role,
  Route,
  ConsentDefaults,
  ConsentBinding,
  Constraints,
  WebmentionConstraints,
  WebmentionPolicy,
  CryptoAlgorithm,
  CryptoPolicy,
  Policy
}
