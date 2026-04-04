// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// E2E tests for the full Webmention processing flow.
//
// Tests the complete path through the webmention service:
//   Incoming request → content-type validation → source/target validation
//   → rate-limit check → accept/reject decision → outcome verification
//
// These tests do not require a running HTTP server; they exercise the
// logic layer (validator + rate limiter) which is what the handlers delegate to.

use std::net::{IpAddr, Ipv4Addr};
use webmention_rate_limiter::{
    config::{RateLimitConfig, ValidationConfig},
    limiter::{RateLimitReason, RateLimitResult, RateLimiter},
    validator::{ValidationError, ValidationResult, WebmentionValidator},
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a validator and limiter with sensible defaults for E2E tests.
fn make_stack(max_rate_rpm: u32, max_rate_per_source: u32) -> (WebmentionValidator, RateLimiter) {
    let validator = WebmentionValidator::new(ValidationConfig::default());
    let limiter = RateLimiter::new(RateLimitConfig {
        max_rate_rpm,
        max_rate_per_source,
        burst_threshold_multiplier: 1000.0, // Disable burst detection in most E2E tests.
        ..Default::default()
    });
    (validator, limiter)
}

/// Represents a single inbound webmention request.
struct WebmentionRequest {
    ip: IpAddr,
    content_type: Option<&'static str>,
    source: Option<&'static str>,
    target: Option<&'static str>,
}

/// Outcome after processing a webmention request through the full stack.
#[derive(Debug, PartialEq, Eq)]
enum ProcessResult {
    Accepted,
    RejectedValidation(String),
    RejectedRateLimit(String),
}

/// Process a webmention request through the validator then the rate limiter.
async fn process(
    req: &WebmentionRequest,
    validator: &WebmentionValidator,
    limiter: &RateLimiter,
) -> ProcessResult {
    // Step 1: Validate the request.
    let validation = validator.validate(req.content_type, req.source, req.target);
    if !validation.is_valid() {
        let reason = validation
            .error()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "unknown validation error".to_string());
        return ProcessResult::RejectedValidation(reason);
    }

    // Step 2: Check rate limits (IP + source).
    let rate_result = limiter.check(req.ip, req.source).await;
    match rate_result {
        RateLimitResult::Allowed { .. } => ProcessResult::Accepted,
        RateLimitResult::Limited { reason, .. } => {
            ProcessResult::RejectedRateLimit(reason.to_string())
        }
    }
}

// ---------------------------------------------------------------------------
// Happy-path E2E
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_valid_webmention_accepted() {
    let (validator, limiter) = make_stack(60, 10);
    let req = WebmentionRequest {
        ip: IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)), // TEST-NET-3
        content_type: Some("application/x-www-form-urlencoded"),
        source: Some("https://external-blog.example.com/post/mentioning-you"),
        target: Some("https://my-site.example.org/article/hello-world"),
    };

    let result = process(&req, &validator, &limiter).await;
    assert_eq!(
        result,
        ProcessResult::Accepted,
        "A well-formed cross-domain webmention from a fresh IP must be accepted"
    );
}

#[tokio::test]
async fn e2e_charset_in_content_type_accepted() {
    let (validator, limiter) = make_stack(60, 10);
    let req = WebmentionRequest {
        ip: IpAddr::V4(Ipv4Addr::new(203, 0, 113, 2)),
        content_type: Some("application/x-www-form-urlencoded; charset=utf-8"),
        source: Some("https://sender.example.com/post/1"),
        target: Some("https://receiver.example.org/article/2"),
    };

    let result = process(&req, &validator, &limiter).await;
    assert_eq!(
        result,
        ProcessResult::Accepted,
        "Content-Type with charset parameter must still be accepted"
    );
}

// ---------------------------------------------------------------------------
// Validation rejection paths
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_invalid_content_type_rejected_before_rate_limit() {
    let (validator, limiter) = make_stack(60, 10);
    let req = WebmentionRequest {
        ip: IpAddr::V4(Ipv4Addr::new(203, 0, 113, 3)),
        content_type: Some("application/json"), // Must be rejected.
        source: Some("https://sender.example.com/post/1"),
        target: Some("https://receiver.example.org/article/2"),
    };

    let result = process(&req, &validator, &limiter).await;
    assert!(
        matches!(result, ProcessResult::RejectedValidation(_)),
        "Invalid content-type must be caught at validation, not rate-limit: {result:?}"
    );
}

#[tokio::test]
async fn e2e_missing_source_rejected() {
    let (validator, limiter) = make_stack(60, 10);
    let req = WebmentionRequest {
        ip: IpAddr::V4(Ipv4Addr::new(203, 0, 113, 4)),
        content_type: Some("application/x-www-form-urlencoded"),
        source: None, // Missing.
        target: Some("https://receiver.example.org/article/2"),
    };

    let result = process(&req, &validator, &limiter).await;
    assert!(
        matches!(result, ProcessResult::RejectedValidation(_)),
        "Missing source must be rejected at validation: {result:?}"
    );
}

#[tokio::test]
async fn e2e_missing_target_rejected() {
    let (validator, limiter) = make_stack(60, 10);
    let req = WebmentionRequest {
        ip: IpAddr::V4(Ipv4Addr::new(203, 0, 113, 5)),
        content_type: Some("application/x-www-form-urlencoded"),
        source: Some("https://sender.example.com/post/1"),
        target: None, // Missing.
    };

    let result = process(&req, &validator, &limiter).await;
    assert!(
        matches!(result, ProcessResult::RejectedValidation(_)),
        "Missing target must be rejected at validation: {result:?}"
    );
}

#[tokio::test]
async fn e2e_self_ping_rejected() {
    let (validator, limiter) = make_stack(60, 10);
    let req = WebmentionRequest {
        ip: IpAddr::V4(Ipv4Addr::new(203, 0, 113, 6)),
        content_type: Some("application/x-www-form-urlencoded"),
        source: Some("https://blog.example.com/post/1"), // Same registrable domain.
        target: Some("https://www.example.com/article/2"),
    };

    let result = process(&req, &validator, &limiter).await;
    assert!(
        matches!(result, ProcessResult::RejectedValidation(_)),
        "Self-ping must be rejected at validation: {result:?}"
    );
}

#[tokio::test]
async fn e2e_ftp_url_rejected() {
    let (validator, limiter) = make_stack(60, 10);
    let req = WebmentionRequest {
        ip: IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)),
        content_type: Some("application/x-www-form-urlencoded"),
        source: Some("ftp://sender.example.com/post/1"), // Invalid scheme.
        target: Some("https://receiver.example.org/article/2"),
    };

    let result = process(&req, &validator, &limiter).await;
    assert!(
        matches!(result, ProcessResult::RejectedValidation(_)),
        "FTP source URL must be rejected at validation: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Rate-limit rejection paths
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_ip_rate_limit_enforced() {
    let (validator, limiter) = make_stack(3, 100);
    let ip = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1)); // TEST-NET-2

    // Three valid requests must be accepted.
    for i in 0..3u32 {
        let req = WebmentionRequest {
            ip,
            content_type: Some("application/x-www-form-urlencoded"),
            source: Some("https://sender.example.com/post/1"),
            target: Some("https://receiver.example.org/article/2"),
        };
        let result = process(&req, &validator, &limiter).await;
        assert_eq!(
            result,
            ProcessResult::Accepted,
            "Request {i} must be accepted before IP limit is reached"
        );
    }

    // The fourth request from the same IP must be rejected by the rate limiter.
    let req = WebmentionRequest {
        ip,
        content_type: Some("application/x-www-form-urlencoded"),
        source: Some("https://sender.example.com/post/1"),
        target: Some("https://receiver.example.org/article/2"),
    };
    let result = process(&req, &validator, &limiter).await;
    assert!(
        matches!(result, ProcessResult::RejectedRateLimit(_)),
        "The 4th request must be rate-limited: {result:?}"
    );
}

#[tokio::test]
async fn e2e_source_rate_limit_enforced() {
    // High IP limit so only the source limit kicks in.
    let (validator, limiter) = make_stack(10_000, 2);
    let ip = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 2));

    // Two valid requests from the same source must be accepted.
    for i in 0..2u32 {
        let req = WebmentionRequest {
            ip,
            content_type: Some("application/x-www-form-urlencoded"),
            source: Some("https://spammy-source.example.com/post/1"),
            target: Some("https://receiver.example.org/article/2"),
        };
        let result = process(&req, &validator, &limiter).await;
        assert_eq!(
            result,
            ProcessResult::Accepted,
            "Request {i} must be accepted before source limit"
        );
    }

    // Third request from same source must be rejected.
    let req = WebmentionRequest {
        ip,
        content_type: Some("application/x-www-form-urlencoded"),
        source: Some("https://spammy-source.example.com/post/1"),
        target: Some("https://receiver.example.org/article/2"),
    };
    let result = process(&req, &validator, &limiter).await;
    assert!(
        matches!(result, ProcessResult::RejectedRateLimit(_)),
        "3rd request from same source must be rate-limited: {result:?}"
    );
}

#[tokio::test]
async fn e2e_different_ips_independent() {
    // Limit of 1 per IP; second IP must still be allowed even after first IP is limited.
    let (validator, limiter) = make_stack(1, 100);
    let ip_a = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 10));
    let ip_b = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 11));

    // Exhaust IP A's budget.
    let req_a = WebmentionRequest {
        ip: ip_a,
        content_type: Some("application/x-www-form-urlencoded"),
        source: Some("https://sender-a.example.com/post/1"),
        target: Some("https://receiver.example.org/article/1"),
    };
    let _ = process(&req_a, &validator, &limiter).await;

    // IP A is now limited.
    let result_a = process(&req_a, &validator, &limiter).await;
    assert!(
        matches!(result_a, ProcessResult::RejectedRateLimit(_)),
        "IP A must be rate-limited after 1 request: {result_a:?}"
    );

    // IP B must still be allowed (independent counter).
    let req_b = WebmentionRequest {
        ip: ip_b,
        content_type: Some("application/x-www-form-urlencoded"),
        source: Some("https://sender-b.example.com/post/1"),
        target: Some("https://receiver.example.org/article/2"),
    };
    let result_b = process(&req_b, &validator, &limiter).await;
    assert_eq!(
        result_b,
        ProcessResult::Accepted,
        "IP B must be unaffected by IP A's rate limit: {result_b:?}"
    );
}

// ---------------------------------------------------------------------------
// Validation precedes rate limiting (no counter increment on bad request)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_invalid_requests_do_not_consume_rate_limit_tokens() {
    // Very tight limit: only 1 allowed per IP.
    let (validator, limiter) = make_stack(1, 100);
    let ip = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 20));

    // Submit 5 invalid requests (wrong content-type). They must all be rejected
    // at the validation stage, never reaching the rate limiter.
    for _ in 0..5 {
        let req = WebmentionRequest {
            ip,
            content_type: Some("application/json"),
            source: Some("https://sender.example.com/post/1"),
            target: Some("https://receiver.example.org/article/2"),
        };
        let result = process(&req, &validator, &limiter).await;
        assert!(
            matches!(result, ProcessResult::RejectedValidation(_)),
            "Invalid requests must be rejected at validation"
        );
    }

    // The 1 valid request must still be accepted (no tokens were spent).
    let valid_req = WebmentionRequest {
        ip,
        content_type: Some("application/x-www-form-urlencoded"),
        source: Some("https://sender.example.com/post/1"),
        target: Some("https://receiver.example.org/article/2"),
    };
    let result = process(&valid_req, &validator, &limiter).await;
    assert_eq!(
        result,
        ProcessResult::Accepted,
        "The 1 allowed request must succeed: invalid requests must not consume tokens"
    );
}
