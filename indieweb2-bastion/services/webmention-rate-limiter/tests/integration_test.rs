// SPDX-FileCopyrightText: 2025 Hyperpolymath
// SPDX-License-Identifier: PMPL-1.0-or-later

//! Integration tests for the Webmention rate limiter.

use std::net::IpAddr;
use webmention_rate_limiter::{
    config::{RateLimitConfig, ValidationConfig},
    limiter::{RateLimitResult, RateLimiter},
    validator::{ValidationResult, WebmentionValidator},
};

#[tokio::test]
async fn test_full_validation_flow() {
    let limiter = RateLimiter::new(RateLimitConfig {
        max_rate_rpm: 10,
        max_rate_per_source: 5,
        ..Default::default()
    });
    let validator = WebmentionValidator::new(ValidationConfig::default());

    let ip: IpAddr = "192.168.1.100".parse().unwrap();
    let source = "https://external-blog.example.com/post/1";
    let target = "https://my-site.example.org/article/hello";
    let content_type = "application/x-www-form-urlencoded";

    // Validate request
    let validation = validator.validate(Some(content_type), Some(source), Some(target));
    assert!(validation.is_valid());

    // Check rate limit
    let rate_result = limiter.check(ip, Some(source)).await;
    assert!(matches!(rate_result, RateLimitResult::Allowed { .. }));
}

#[tokio::test]
async fn test_rate_limit_exhaustion() {
    let limiter = RateLimiter::new(RateLimitConfig {
        max_rate_rpm: 3,
        max_rate_per_source: 10,
        ..Default::default()
    });

    let ip: IpAddr = "10.0.0.1".parse().unwrap();

    // Exhaust rate limit
    for i in 0..3 {
        let result = limiter.check_ip(ip).await;
        assert!(
            matches!(result, RateLimitResult::Allowed { .. }),
            "Request {} should be allowed",
            i + 1
        );
    }

    // Next request should be limited
    let result = limiter.check_ip(ip).await;
    assert!(matches!(result, RateLimitResult::Limited { .. }));
}

#[tokio::test]
async fn test_validation_rejects_invalid_content_type() {
    let validator = WebmentionValidator::new(ValidationConfig::default());

    let result = validator.validate(Some("application/json"), Some("https://source.com"), Some("https://target.com"));
    assert!(!result.is_valid());
}

#[tokio::test]
async fn test_validation_rejects_self_ping() {
    let validator = WebmentionValidator::new(ValidationConfig::default());

    // Same domain should be rejected
    let result = validator.validate(
        Some("application/x-www-form-urlencoded"),
        Some("https://blog.example.com/post/1"),
        Some("https://www.example.com/post/2"),
    );
    assert!(!result.is_valid());
}

#[tokio::test]
async fn test_source_rate_limiting_independent() {
    let limiter = RateLimiter::new(RateLimitConfig {
        max_rate_rpm: 100,
        max_rate_per_source: 2,
        ..Default::default()
    });

    let source1 = "https://spammer.example.com/post";
    let source2 = "https://legitimate.example.org/article";

    // Source 1 - exhaust limit
    for _ in 0..2 {
        let result = limiter.check_source(source1).await;
        assert!(matches!(result, RateLimitResult::Allowed { .. }));
    }

    // Source 1 - should be limited
    let result = limiter.check_source(source1).await;
    assert!(matches!(result, RateLimitResult::Limited { .. }));

    // Source 2 - should still be allowed (independent)
    let result = limiter.check_source(source2).await;
    assert!(matches!(result, RateLimitResult::Allowed { .. }));
}
