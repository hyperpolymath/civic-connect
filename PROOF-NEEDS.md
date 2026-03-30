# Proof Requirements

## Current state
- `src/abi/Types.idr` (65 lines) — Civic domain types
- `src/abi/Layout.idr` (177 lines) — Memory layout
- `src/abi/Foreign.idr` (39 lines) — FFI stubs
- ABI layer is minimal/skeletal — no dangerous patterns but also no substantive proofs
- 74K lines of source code overall

## What needs proving
- **Citizen data privacy**: Prove that personally identifiable information (PII) never leaves the local processing boundary without explicit consent
- **Vote/petition integrity**: If the platform handles any form of civic participation, prove tallies are correct and tamper-evident
- **Access control correctness**: Prove role-based access control (citizen, representative, admin) enforces least privilege
- **Audit trail completeness**: Prove all state-changing operations produce audit log entries (no silent mutations)

## Recommended prover
- **Idris2** — Expand the existing skeletal ABI into substantive dependent-type proofs for access control and data flow

## Priority
- **MEDIUM** — Civic platforms inherently handle sensitive citizen data and trust relationships. The ABI exists but is too thin to provide real guarantees. Priority increases if the platform handles voting or petition mechanisms.

## Template ABI Cleanup (2026-03-29)

Template ABI removed -- was creating false impression of formal verification.
The removed files (Types.idr, Layout.idr, Foreign.idr) contained only RSR template
scaffolding with unresolved {{PROJECT}}/{{AUTHOR}} placeholders and no domain-specific proofs.
