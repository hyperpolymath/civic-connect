---
title: "[IndieWeb] Add Micropub Content-Type validation at ingress"
labels: enhancement, indieweb, protocol, triage
---

## Protocol

- [x] Micropub

## Security Layer

- [x] **Network/Ingress Layer** (indieweb2-bastion) - Content-Type validation, payload size limits
- [x] **Application Layer** - Deep input sanitization (handled by php-aegis)

## Problem Description

Micropub endpoints accept three content types:
- `application/json` (JSON payload)
- `application/x-www-form-urlencoded` (form-encoded)
- `multipart/form-data` (file uploads)

Requests with incorrect or missing Content-Type headers should be rejected at the ingress level before reaching the application, preventing:
- Malformed request processing overhead
- Content-Type confusion attacks
- Parser exploitation attempts

## Proposed Solution

### Ingress-Level

Add Micropub-specific validation in the CURPS policy:

```nickel
// policy/curps/micropub.ncl
let micropub_policy = {
  routes = ["/micropub", "/micropub.php"],
  constraints = {
    require_content_type = [
      "application/json",
      "application/x-www-form-urlencoded",
      "multipart/form-data"
    ],
    reject_on_missing_auth = true,       // Must have Authorization header
    max_payload_size_bytes = 10485760,   // 10 MiB (for media uploads)
    max_json_depth = 10,                 // Prevent deeply nested JSON attacks
    require_https = true,                // No plaintext Micropub
  },
  rate_limits = {
    create = 30,    // 30 creates/minute
    update = 60,    // 60 updates/minute
    delete = 10,    // 10 deletes/minute
    media = 10,     // 10 media uploads/minute
  }
}
```

### Validation Flow

```
Request → Ingress Gateway
           ├─ Check Content-Type header
           │   └─ Reject if not in allowed list
           ├─ Check Authorization header presence
           │   └─ Reject if missing (don't validate token - app layer does that)
           ├─ Check payload size
           │   └─ Reject if > 10 MiB
           └─ Forward to application
```

## Alternatives Considered

1. **Application-only validation**: Wastes resources parsing invalid requests
2. **Strict JSON schema at ingress**: Too complex, better left to application
3. **Block all multipart**: Would break media endpoint functionality

## Related Resources

- [x] I have read [INDIEWEB_INTEGRATION.adoc](../../docs/INDIEWEB_INTEGRATION.adoc)
- [x] I have reviewed the [CURPS policy language](../../policy/curps/policy.ncl)
- Relevant IndieWeb spec: https://indieweb.org/Micropub

## Additional Context

The bastion validates structural requirements; the application layer (php-aegis) handles:
- IndieAuth token verification
- Content sanitization
- Vocabulary validation
