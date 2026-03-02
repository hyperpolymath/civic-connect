---
title: "[IndieWeb] Add Webmention rate limiting at ingress level"
labels: enhancement, indieweb, protocol, triage
---

## Protocol

- [x] Webmention

## Security Layer

- [x] **Network/Ingress Layer** (indieweb2-bastion) - Rate limiting, Content-Type validation, mTLS enforcement

## Problem Description

Webmention endpoints are vulnerable to abuse through:
- Spam floods from malicious sources
- DoS attacks via high-volume mention submissions
- Amplification attacks (mention → fetch → process)

Currently, the default bastion rate limit is 120 rpm, but Webmention endpoints should have stricter limits due to the processing overhead each mention incurs.

## Proposed Solution

### Ingress-Level

Add Webmention-specific rate limiting in the CURPS policy:

```nickel
// policy/curps/webmention.ncl
let webmention_policy = {
  routes = ["/webmention", "/webmention.php", "/.well-known/webmention"],
  constraints = {
    max_rate_rpm = 60,          // Stricter than default 120
    max_rate_per_source = 10,   // Per source URL
    require_content_type = ["application/x-www-form-urlencoded"],
    require_source_target = true,
    block_self_ping = true,
    cooldown_on_burst_ms = 30000,
  }
}
```

### Additional Ingress Validations

1. **Source/Target validation**: Reject requests missing `source` or `target` params
2. **Self-ping blocking**: Reject if source and target share same domain
3. **URL format validation**: Reject malformed URLs before passing to application

## Alternatives Considered

1. **Application-only rate limiting**: Insufficient as requests still consume ingress bandwidth
2. **IP-based blocking**: Too coarse, legitimate services may share IPs
3. **Captcha**: Breaks protocol compatibility (machine-to-machine)

## Related Resources

- [x] I have read [INDIEWEB_INTEGRATION.adoc](../../docs/INDIEWEB_INTEGRATION.adoc)
- [x] I have reviewed the [CURPS policy language](../../policy/curps/policy.ncl)
- Relevant IndieWeb spec: https://indieweb.org/Webmention
- Related projects: sinople (application-level Webmention validation)

## Additional Context

This is part of the IndieWeb protocol integration initiative. The application layer (sanctify-php/sinople) handles deep validation; this issue focuses on what can be blocked at the network edge.
