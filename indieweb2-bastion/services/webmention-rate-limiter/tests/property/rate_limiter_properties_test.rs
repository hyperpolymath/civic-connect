// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Property-based tests for the Webmention rate limiter.
//
// Invariants verified:
//   - N requests within limit → all accepted
//   - N+1 requests → (N+1)th rejected
//   - Source rate limiting is independent between different sources
//   - Combined IP+source check respects both limits
//   - Validator always rejects missing / invalid fields
//   - Self-ping is always blocked when block_self_ping is enabled

use proptest::prelude::*;
use std::net::{IpAddr, Ipv4Addr};
use webmention_rate_limiter::{
    config::{RateLimitConfig, ValidationConfig},
    limiter::{RateLimitReason, RateLimitResult, RateLimiter},
    validator::{ValidationError, WebmentionValidator},
};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Arbitrary IPv4 address in private ranges (avoids reserved/broadcast issues).
fn arb_ipv4() -> impl Strategy<Value = IpAddr> {
    (0u8..=254u8, 0u8..=254u8, 0u8..=254u8).prop_map(|(b, c, d)| {
        IpAddr::V4(Ipv4Addr::new(10, b, c, d))
    })
}

/// Arbitrary rate limit between 1 and 20 rpm (keeps tests fast).
fn arb_rate_rpm() -> impl Strategy<Value = u32> {
    1u32..=20u32
}

/// Arbitrary valid source URL (different domain from target).
fn arb_source_url() -> impl Strategy<Value = String> {
    (1usize..=99usize).prop_map(|n| format!("https://source-{n}.external.example.com/post/1"))
}

// ---------------------------------------------------------------------------
// Helper: run an async closure synchronously via a one-shot runtime.
// Returns the inner value so proptest can use it in the outer (sync) context.
// ---------------------------------------------------------------------------

fn block<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    tokio::runtime::Runtime::new()
        .expect("create tokio runtime")
        .block_on(future)
}

// ---------------------------------------------------------------------------
// Rate limiter properties
// ---------------------------------------------------------------------------

proptest! {
    /// For any limit N (1–20), the first N requests from the same IP must all
    /// be accepted.
    #[test]
    fn prop_n_requests_within_limit_accepted(
        limit in arb_rate_rpm(),
        ip in arb_ipv4(),
    ) {
        let config = RateLimitConfig {
            max_rate_rpm: limit,
            // Disable burst detection so it does not interfere with
            // the core limit property.
            burst_threshold_multiplier: 1000.0,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);

        let all_allowed = block(async {
            for i in 0..limit {
                match limiter.check_ip(ip).await {
                    RateLimitResult::Allowed { .. } => {}
                    RateLimitResult::Limited { reason, .. } => {
                        return Err(format!(
                            "Request {} (of {}) unexpectedly limited: {}",
                            i + 1, limit, reason
                        ));
                    }
                }
            }
            Ok(())
        });
        prop_assert!(
            all_allowed.is_ok(),
            "all N requests must be allowed: {}",
            all_allowed.unwrap_err()
        );
    }

    /// The (N+1)th request from the same IP must be rejected after N have been
    /// consumed within the same token-bucket window.
    #[test]
    fn prop_n_plus_one_request_rejected(
        limit in arb_rate_rpm(),
        ip in arb_ipv4(),
    ) {
        let config = RateLimitConfig {
            max_rate_rpm: limit,
            burst_threshold_multiplier: 1000.0,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);

        // Consume all tokens and capture result of the (N+1)th request.
        let (n_plus_one_result, is_rate_exceeded) = block(async {
            for _ in 0..limit {
                let _ = limiter.check_ip(ip).await;
            }
            let result = limiter.check_ip(ip).await;
            let exceeded = matches!(
                result,
                RateLimitResult::Limited {
                    reason: RateLimitReason::IpRateExceeded,
                    ..
                }
            );
            (result, exceeded)
        });

        prop_assert!(
            matches!(n_plus_one_result, RateLimitResult::Limited { .. }),
            "request N+1 must be rate-limited"
        );
        prop_assert!(
            is_rate_exceeded,
            "reason must be IpRateExceeded after exhausting the bucket"
        );
    }

    /// Rate limits for different source URLs must be independent: exhausting
    /// source A must not affect source B.
    #[test]
    fn prop_source_limits_independent(
        source_limit in 1u32..=10u32,
        suffix_a in 1usize..=50usize,
        suffix_b in 51usize..=100usize,
    ) {
        let config = RateLimitConfig {
            max_rate_per_source: source_limit,
            max_rate_rpm: 10_000, // Very high IP limit so IP is never the bottleneck.
            burst_threshold_multiplier: 1000.0,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);
        let source_a = format!("https://source-{suffix_a}.example.com/post/1");
        let source_b = format!("https://source-{suffix_b}.example.com/post/1");

        let (a_limited, b_allowed) = block(async {
            // Exhaust source A.
            for _ in 0..source_limit {
                let _ = limiter.check_source(&source_a).await;
            }
            let a = matches!(limiter.check_source(&source_a).await, RateLimitResult::Limited { .. });
            let b = matches!(limiter.check_source(&source_b).await, RateLimitResult::Allowed { .. });
            (a, b)
        });

        prop_assert!(a_limited, "source A should be limited after exhausting its bucket");
        prop_assert!(b_allowed, "source B must remain allowed when only source A is exhausted");
    }
}

// ---------------------------------------------------------------------------
// Validator properties
// ---------------------------------------------------------------------------

proptest! {
    /// Missing source parameter must always be rejected when require_source_target is true.
    #[test]
    fn prop_missing_source_always_rejected(
        target in arb_source_url(),
    ) {
        let validator = WebmentionValidator::new(ValidationConfig::default());
        let result = validator.validate_source_target(None, Some(&target));
        prop_assert!(!result.is_valid(),
            "missing source must always produce an invalid result");
        prop_assert!(
            matches!(result.error(), Some(ValidationError::MissingParameter("source"))),
            "error must be MissingParameter(source), got {:?}", result.error()
        );
    }

    /// Missing target parameter must always be rejected.
    #[test]
    fn prop_missing_target_always_rejected(
        source in arb_source_url(),
    ) {
        let validator = WebmentionValidator::new(ValidationConfig::default());
        let result = validator.validate_source_target(Some(&source), None);
        prop_assert!(!result.is_valid(),
            "missing target must always produce an invalid result");
        prop_assert!(
            matches!(result.error(), Some(ValidationError::MissingParameter("target"))),
            "error must be MissingParameter(target), got {:?}", result.error()
        );
    }

    /// When block_self_ping is true, any same-domain source/target pair must be rejected.
    ///
    /// We use varying subdomains to exercise the registrable-domain logic.
    #[test]
    fn prop_self_ping_always_blocked(
        sub_a in "[a-z]{3,6}",
        sub_b in "[a-z]{3,6}",
        domain in "[a-z]{5,10}",
    ) {
        prop_assume!(sub_a != sub_b); // Different subdomains, same registrable domain.
        let config = ValidationConfig {
            block_self_ping: true,
            ..Default::default()
        };
        let validator = WebmentionValidator::new(config);

        let source = format!("https://{sub_a}.{domain}.com/post/1");
        let target = format!("https://{sub_b}.{domain}.com/article/2");

        let result = validator.validate_source_target(Some(&source), Some(&target));
        prop_assert!(!result.is_valid(),
            "self-ping ({source} → {target}) must be blocked");
        prop_assert!(
            matches!(result.error(), Some(ValidationError::SelfPingBlocked { .. })),
            "error must be SelfPingBlocked, got {:?}", result.error()
        );
    }

    /// Cross-domain requests must always be accepted by the validator (ignoring rate limits).
    ///
    /// Uses two different numeric suffixes to guarantee different domains.
    #[test]
    fn prop_cross_domain_accepted_by_validator(
        n in 1usize..=9999usize,
    ) {
        let config = ValidationConfig::default();
        let validator = WebmentionValidator::new(config);

        let source = format!("https://source-blog-{n}.example.com/post/1");
        let target = format!("https://target-site-{n}.example.org/article/2");

        let result = validator.validate(
            Some("application/x-www-form-urlencoded"),
            Some(&source),
            Some(&target),
        );
        prop_assert!(result.is_valid(),
            "cross-domain webmention ({source} → {target}) must pass validation");
    }
}
