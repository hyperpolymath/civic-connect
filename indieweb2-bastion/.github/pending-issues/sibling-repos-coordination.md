---
title: "[Meta] Cross-reference IndieWeb security architecture with sibling repos"
labels: documentation, meta, triage
---

## Context

This issue tracks coordination between indieweb2-bastion (infrastructure layer) and application-layer security libraries in the Hyperpolymath ecosystem.

## Related Repositories

| Repository | Layer | Purpose | Status |
|------------|-------|---------|--------|
| **indieweb2-bastion** | Network/Ingress | Gateway, DNS, provenance, consent | This repo |
| **sanctify-php** | Application | Input sanitization, output escaping, Turtle/RDF | Sibling |
| **php-aegis** | Application | PHP security library for IndieWeb protocols | Proposed |
| **sinople** | Application | Webmention rate limiting, validation | Existing |

## Tasks

### Documentation Cross-References

- [ ] Add link to `docs/INDIEWEB_INTEGRATION.adoc` from sanctify-php README
- [ ] Add link from sanctify-php to bastion's provenance tracking docs
- [ ] Create shared glossary of security terms across repos

### Architecture Alignment

- [ ] Ensure sanctify-php adopts provenance tracking pattern from bastion
- [ ] Ensure sanctify-php uses Nickel-style policy contracts where appropriate
- [ ] Align consent manifest format between bastion and application layers

### Issue Tracking

Related issues in this repo:
- [ ] #TBD - Webmention rate limiting at ingress
- [ ] #TBD - Micropub Content-Type validation
- [ ] #TBD - IndieAuth token pre-validation

### Integration Testing

- [ ] Create integration test suite that validates bastion + sanctify-php together
- [ ] Document expected behavior at each layer boundary
- [ ] Create sample deployment showing both layers

## Notes

The assessment from sanctify-php correctly identified that indieweb2-bastion is infrastructure-level, not protocol-level. This coordination ensures both layers work together without gaps or overlaps.

## Lessons Exchange

### FROM bastion → TO application repos:
1. **Provenance tracking** - Track sanitization chain
2. **Policy contracts** - Nickel-style security policies
3. **Consent-aware architecture** - User control principles

### FROM application repos → TO bastion:
1. **Protocol awareness** - IndieWeb-specific ingress rules
2. **Content-Type enforcement** - Block malformed requests early
3. **Rate limiting per endpoint** - Different limits for different protocols
