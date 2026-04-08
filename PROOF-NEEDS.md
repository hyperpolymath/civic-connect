# Proof Requirements

## Current state
- `src/Abi/Types.idr` — Civic domain types (RBAC, Consent, Auditing)
- `src/Abi/Layout.idr` — Memory layout proofs (C ABI compliance)
- `src/Abi/Foreign.idr` — Type-safe FFI bridge
- `src/Abi/Proofs.idr` — Substantive proofs for security and logic invariants
- ABI layer is formally verified for core invariants

## What was proven
- **Citizen data privacy**: Proved that PII access is only possible with a valid `Consent` proof (`piiAccessRequiresConsent`).
- **Vote/petition integrity**: Proved that tallying is correct and adding a vote specifically increment the correct counter (`tallyIncreasesByOne`, `tallyIndependent`).
- **Access control correctness**: Proved that sensitive privileges (ManageUsers, AccessPII) are restricted to the `Admin` role (`onlyAdminCanManage`, `onlyAdminCanPII`).
- **Audit trail completeness**: Proved that every state mutation designated as `Audited` must be accompanied by an `AuditEntry` (`extractAuditEntry`).

## Recommended prover
- **Idris2** — Completed. The skeletal ABI has been expanded into a fully verified layer.

## Priority
- **LOW** — Core invariants are now formally verified. Ongoing work should maintain these proofs as the system evolves.

## Proof Verification (2026-04-04)
- All proofs successfully verified using Idris2.
- Directory structured for Idris2 module compliance (`src/Abi/`).
- Security and logic invariants formally modeled and proven.
