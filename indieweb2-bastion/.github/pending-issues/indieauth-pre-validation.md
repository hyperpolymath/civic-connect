---
title: "[IndieWeb] Add IndieAuth token format pre-validation at ingress"
labels: enhancement, indieweb, protocol, triage
---

## Protocol

- [x] IndieAuth

## Security Layer

- [x] **Network/Ingress Layer** (indieweb2-bastion) - Token format validation, header checks
- [x] **Application Layer** - Full token verification against token endpoint

## Problem Description

IndieAuth tokens require verification against the token endpoint, but malformed tokens can be rejected at the ingress level:
- Missing `Authorization: Bearer` header
- Malformed token format (should be opaque string, reasonable length)
- Expired JWTs (if JWT format is used and exp is readable)

Rejecting obviously invalid tokens at ingress reduces load on application servers and token endpoints.

## Proposed Solution

### Ingress-Level (Pre-validation Only)

```nickel
// policy/curps/indieauth.ncl
let indieauth_policy = {
  // Endpoints that require IndieAuth
  protected_routes = ["/micropub", "/microsub", "/media"],

  constraints = {
    require_auth_header = true,
    auth_header_format = "Bearer",
    min_token_length = 20,     // Reject obviously short tokens
    max_token_length = 4096,   // Reject absurdly long tokens

    // Optional: If tokens are JWTs, check exp claim without validating signature
    jwt_exp_check = {
      enabled = false,         // Default off - most IndieAuth tokens are opaque
      clock_skew_seconds = 60,
    },
  },

  // What NOT to do at ingress
  explicitly_deferred = [
    "token_verification",      // App must call token endpoint
    "scope_validation",        // App must check scopes
    "me_url_verification",     // App must verify identity
  ]
}
```

### Validation Flow

```
Request → Ingress Gateway
           ├─ Check Authorization header exists
           │   └─ Reject 401 if missing
           ├─ Check "Bearer " prefix
           │   └─ Reject 401 if wrong format
           ├─ Check token length bounds
           │   └─ Reject 401 if out of bounds
           └─ Forward to application
                └─ Application calls token endpoint for real verification
```

## Important Caveats

**The ingress MUST NOT:**
- Verify tokens against the token endpoint (that's application responsibility)
- Validate scopes or permissions
- Cache token verification results
- Trust token claims without verification

**The ingress SHOULD:**
- Log authentication attempts (with token hash, not full token)
- Rate limit authentication failures per IP
- Track failed auth patterns for abuse detection

## Alternatives Considered

1. **No ingress validation**: Valid, but wastes resources on obviously malformed requests
2. **Full token verification at ingress**: Too complex, adds latency, breaks separation of concerns
3. **Token caching at ingress**: Security risk, stale tokens could be accepted

## Related Resources

- [x] I have read [INDIEWEB_INTEGRATION.adoc](../../docs/INDIEWEB_INTEGRATION.adoc)
- [x] I have reviewed the [CURPS policy language](../../policy/curps/policy.ncl)
- Relevant IndieWeb spec: https://indieauth.spec.indieweb.org/

## Additional Context

This follows the principle of "fail fast" - reject obviously bad requests early. Full IndieAuth verification remains the application layer's responsibility.
