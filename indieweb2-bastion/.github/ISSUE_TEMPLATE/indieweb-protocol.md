---
name: IndieWeb Protocol Integration
about: Propose IndieWeb protocol support for bastion ingress rules
title: '[IndieWeb] '
labels: 'enhancement, indieweb, protocol, triage'
assignees: ''

---

## Protocol

Which IndieWeb protocol does this relate to?

- [ ] Micropub
- [ ] Webmention
- [ ] IndieAuth
- [ ] WebSub
- [ ] Microsub
- [ ] Other: _____________

## Security Layer

Where should this be handled?

- [ ] **Network/Ingress Layer** (indieweb2-bastion) - Rate limiting, Content-Type validation, mTLS enforcement
- [ ] **Application Layer** (php-aegis, sanctify-php) - Input sanitization, output escaping, token verification
- [ ] **Both** - Describe the split below

## Problem Description

What security gap or integration need does this address?

## Proposed Solution

### Ingress-Level (if applicable)

Describe what the bastion should validate/enforce at the network level.

Example Nickel policy (optional):
```nickel
let my_policy = {
  routes = ["/my-endpoint"],
  constraints = {
    // ...
  }
}
```

### Application-Level (if applicable)

Describe what the application library should handle.

## Alternatives Considered

What other approaches were considered?

## Related Resources

- [ ] I have read [INDIEWEB_INTEGRATION.adoc](../../docs/INDIEWEB_INTEGRATION.adoc)
- [ ] I have reviewed the [CURPS policy language](../../policy/curps/policy.ncl)
- Relevant IndieWeb spec: https://indieweb.org/____________
- Related issues: #___

## Additional Context

Any other context, diagrams, or references.
