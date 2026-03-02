# Nickel Policy Integration

The GraphQL DNS API integrates with Nickel CURPS (Consent-aware Universal Request Policy System) policies for governance and access control.

## Overview

The API enforces:
- **Role-Based Access Control (RBAC)**: Only authorized identities can propose mutations
- **Approval Requirements**: Critical mutations require multiple approvals
- **Timelock Delays**: Mutations have mandatory delay periods before execution
- **Consent Bindings**: DNS operations respect user consent preferences
- **Rate Limiting**: Enforced request limits per identity

## Policy Loading

The API loads policy from `../policy/curps/policy.ncl` at startup:

```bash
# Requires nickel CLI installed
cargo install nickel-lang-cli

# Start API (loads policy automatically)
cd graphql-dns-api
cargo run
```

If the policy file is missing or invalid, the API falls back to a permissive default policy for development.

## Policy Configuration

The Nickel policy defines:

### Mutations
Mutations that require governance approval:

| Mutation | Approvals | Timelock | Description |
|----------|-----------|----------|-------------|
| `rotate_keys` | 2 | 24h | Rotate bastion/resolver/SDP credentials |
| `mutate_dns` | 1 | 1h | Apply GraphQL DNS mutations (records/zones) |

### Roles
Roles with members and privileges:

| Role | Members | Privileges |
|------|---------|------------|
| `maintainer` | alice, jonathan | publish_manifest, rotate_keys, mutate_dns |
| `trusted_contributor` | (empty) | publish_manifest |

### Constraints
- `require_mtls`: Enforce mutual TLS for all connections
- `log_all_mutations`: Audit log all mutation attempts
- `max_rate_rpm`: 120 requests per minute per identity

## GraphQL API

### Query Current Policy

```graphql
query {
  policy {
    version
    mutations {
      name
      description
      approvals
      timelockHours
    }
    roles {
      name
      members
      privileges
    }
    constraints {
      requireMtls
      logAllMutations
      maxRateRpm
    }
  }
}
```

### Check Privilege

```graphql
query {
  hasPrivilege(
    identity: "identity:alice"
    privilege: "mutate_dns"
  )
}
```

### Propose Mutation

```graphql
mutation {
  proposeMutation(
    mutationName: "mutate_dns"
    payload: {
      operation: "createDNSRecord"
      input: {
        name: "example.com"
        type: A
        ttl: 3600
        value: "192.0.2.1"
      }
    }
  ) {
    id
    mutationName
    proposer
    proposedAt
    timelockUntil
    approvals
    requiredApprovals
    status
  }
}
```

### Approve Proposal

```graphql
mutation {
  approveMutation(
    proposalId: "uuid-of-proposal"
  ) {
    id
    approvals
    requiredApprovals
    status
  }
}
```

### Execute Approved Proposal

```graphql
mutation {
  executeMutation(
    proposalId: "uuid-of-proposal"
  ) {
    id
    status
  }
}
```

### List Proposals

```graphql
query {
  proposals(status: PENDING) {
    id
    mutationName
    proposer
    proposedAt
    timelockUntil
    approvals
    requiredApprovals
    status
  }
}
```

## Governance Workflow

1. **Propose**: Authorized identity proposes a mutation
   - System checks if proposer has privilege
   - Creates proposal with initial approval from proposer
   - Sets timelock expiration timestamp
   - Status: `PENDING` or `TIMELOCK_ACTIVE`

2. **Approve**: Other authorized identities approve
   - System verifies approver has privilege
   - Adds approver to approval list
   - When enough approvals: status → `APPROVED`

3. **Wait**: Timelock delay elapses
   - Cannot execute until `timelockUntil` timestamp passes
   - Ensures time for review and objections

4. **Execute**: Mutation is executed
   - System verifies enough approvals and timelock expired
   - Executes the actual DNS operation
   - Status → `EXECUTED`

## Identity Management

Identities are extracted from request context:
- **mTLS Client Certificates**: Extract from TLS cert CN/SAN
- **JWT Bearer Tokens**: Extract from token claims
- **API Keys**: Map keys to identity strings

Format: `identity:<username>` (e.g., `identity:alice`)

## Development Mode

For development without Nickel:

```rust
// Default policy in main.rs
let default_policy = Policy {
    version: "0.1.0",
    constraints: Constraints {
        require_mtls: false,        // Disabled for dev
        log_all_mutations: true,
        max_rate_rpm: 120,
    },
    mutations: vec![],
    roles: vec![],
    // ... other fields empty
};
```

This allows the API to run without policy enforcement for testing.

## Production Deployment

### Requirements
1. Install Nickel CLI: `cargo install nickel-lang-cli`
2. Create/update policy at `policy/curps/policy.ncl`
3. Configure mTLS certificates for identity extraction
4. Set environment variables for blockchain connectivity

### Policy Updates
- Edit `policy/curps/policy.ncl`
- Validate: `nickel export --format json policy/curps/policy.ncl`
- Restart API to reload policy

### Audit Logs
All mutation proposals, approvals, and executions are logged:
```
2026-01-22T20:00:00Z [INFO] Proposal created: mutate_dns by identity:alice
2026-01-22T20:00:01Z [INFO] Approval added: proposal-uuid by identity:jonathan
2026-01-22T21:00:05Z [INFO] Mutation executed: proposal-uuid
```

## Security Considerations

1. **Identity Verification**: Ensure identities are cryptographically verified (mTLS, JWT signatures)
2. **Timelock Bypass**: Never bypass timelocks, even for "emergency" operations
3. **Approval Requirements**: Never reduce approvals below policy minimum
4. **Policy Changes**: Require same governance as sensitive mutations
5. **Rate Limiting**: Enforce at ingress layer (reverse proxy) as well

## Testing

```bash
# Unit tests
cargo test --package graphql-dns-api

# Integration test with policy
cd graphql-dns-api
cargo run &
curl http://localhost:8080/graphql -d '{"query": "{ policy { version } }"}'
```

## Troubleshooting

### Policy Load Failure
```
WARN Failed to load policy file (Nickel export failed), using default policy
```

**Causes:**
- Nickel CLI not installed
- Policy file has syntax errors
- File permissions deny read access

**Fix:**
```bash
# Install Nickel
cargo install nickel-lang-cli

# Validate policy
nickel export --format json policy/curps/policy.ncl

# Check file exists
ls -la ../policy/curps/policy.ncl
```

### Privilege Denied
```
ERROR Identity identity:bob lacks privilege for mutate_dns
```

**Cause**: Identity not in any role with required privilege

**Fix**: Add identity to appropriate role in policy.ncl:
```nickel
roles = [
  {
    name = "maintainer",
    members = ["identity:alice", "identity:bob"],  # Add bob
    privileges = ["mutate_dns"],
  },
]
```

### Timelock Not Expired
```
ERROR Proposal cannot be executed yet (timelock active)
```

**Cause**: Attempting to execute before timelock expiration

**Fix**: Wait for `timelockUntil` timestamp to pass, or reduce `timelock_hours` in policy for faster testing

## References

- [Nickel Language](https://nickel-lang.org/)
- [CURPS Specification](../policy/curps/README.md)
- [GraphQL DNS API Schema](schema.graphql)
