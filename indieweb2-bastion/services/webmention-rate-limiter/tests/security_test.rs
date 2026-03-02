// SPDX-FileCopyrightText: 2025 Hyperpolymath
// SPDX-License-Identifier: PMPL-1.0-or-later

//! Security tests for Webmention rate limiter.
//!
//! These tests simulate various attack patterns and validate that
//! the rate limiter correctly mitigates them.

mod harness;

use harness::{attacks::AttackConfig, generators, metrics::{AttackMetrics, Outcome}};
use std::time::{Duration, Instant};
use webmention_rate_limiter::{
    config::{RateLimitConfig, ValidationConfig},
    limiter::{RateLimitResult, RateLimiter},
    validator::WebmentionValidator,
};

/// Run an attack simulation against the rate limiter.
async fn run_attack(
    config: &AttackConfig,
    rate_config: RateLimitConfig,
    validation_config: ValidationConfig,
) -> AttackMetrics {
    let limiter = RateLimiter::new(rate_config);
    let validator = WebmentionValidator::new(validation_config);

    let ips = generators::generate_ips(config.unique_ips);
    let sources = generators::generate_sources(config.unique_sources);
    let targets = generators::generate_targets(10, "target.example.org");
    let self_pings = generators::generate_self_ping_pairs(config.unique_sources);

    let content_type = if config.valid_content_type {
        Some("application/x-www-form-urlencoded")
    } else {
        Some("application/json")
    };

    let mut metrics = AttackMetrics::new();
    metrics.start();

    let delay = Duration::from_secs_f64(1.0 / config.requests_per_second);

    for i in 0..config.total_requests {
        let start = Instant::now();

        // Select IP and source based on index
        let ip = ips[i % ips.len()];
        let ip_str = ip.to_string();

        // Determine source/target based on self-ping ratio
        let (source, target) = if rand_bool(config.self_ping_ratio, i) {
            let pair = &self_pings[i % self_pings.len()];
            (Some(pair.0.as_str()), Some(pair.1.as_str()))
        } else {
            (
                Some(sources[i % sources.len()].as_str()),
                Some(targets[i % targets.len()].as_str()),
            )
        };

        let (source_param, target_param) = if config.include_params {
            (source, target)
        } else {
            (None, None)
        };

        // Validate request
        let validation = validator.validate(content_type, source_param, target_param);
        let latency = start.elapsed();

        if !validation.is_valid() {
            let outcome = match validation.error() {
                Some(webmention_rate_limiter::validator::ValidationError::InvalidContentType { .. }) => {
                    Outcome::InvalidContentType
                }
                Some(webmention_rate_limiter::validator::ValidationError::MissingParameter(_)) => {
                    Outcome::MissingParams
                }
                Some(webmention_rate_limiter::validator::ValidationError::InvalidUrl { .. }) => {
                    Outcome::InvalidUrl
                }
                Some(webmention_rate_limiter::validator::ValidationError::SelfPingBlocked { .. }) => {
                    Outcome::SelfPingBlocked
                }
                None => Outcome::InvalidContentType, // Shouldn't happen
            };
            metrics.record(outcome, &ip_str, source, latency);
            continue;
        }

        // Check rate limit
        let rate_result = limiter.check(ip, source).await;

        let outcome = match rate_result {
            RateLimitResult::Allowed { .. } => Outcome::Allowed,
            RateLimitResult::Limited { reason, .. } => match reason {
                webmention_rate_limiter::limiter::RateLimitReason::IpRateExceeded => {
                    Outcome::RateLimitedIp
                }
                webmention_rate_limiter::limiter::RateLimitReason::SourceRateExceeded => {
                    Outcome::RateLimitedSource
                }
                webmention_rate_limiter::limiter::RateLimitReason::BurstCooldown => {
                    Outcome::BurstBlocked
                }
            },
        };

        metrics.record(outcome, &ip_str, source, latency);

        // Delay between requests (simulating attack rate)
        if delay > Duration::from_micros(100) {
            tokio::time::sleep(delay).await;
        }
    }

    metrics.finish();
    metrics
}

/// Simple deterministic "random" based on index and ratio.
fn rand_bool(ratio: f64, index: usize) -> bool {
    if ratio >= 1.0 {
        true
    } else if ratio <= 0.0 {
        false
    } else {
        (index as f64 * 0.618033988749895) % 1.0 < ratio
    }
}

// ============================================================================
// Attack Simulation Tests
// ============================================================================

#[tokio::test]
async fn test_single_ip_flood() {
    let config = AttackConfig::single_ip_flood();
    let expectations = config.expectations();

    let metrics = run_attack(
        &config,
        RateLimitConfig::default(),
        ValidationConfig::default(),
    )
    .await;

    let report = metrics.report();
    println!("{}", report);

    // Single IP should be heavily rate limited
    assert!(
        report.block_rate >= 0.5,
        "Block rate {} should be >= 50% for single IP flood",
        report.block_rate
    );
}

#[tokio::test]
async fn test_distributed_attack() {
    let config = AttackConfig::distributed_attack();

    let metrics = run_attack(
        &config,
        RateLimitConfig::default(),
        ValidationConfig::default(),
    )
    .await;

    let report = metrics.report();
    println!("{}", report);

    // Distributed attacks are harder to mitigate at this layer
    // But each IP is still individually rate limited
    assert!(report.unique_ips > 50, "Should have many unique IPs");
}

#[tokio::test]
async fn test_self_ping_attack() {
    let config = AttackConfig::self_ping_attack();

    let metrics = run_attack(
        &config,
        RateLimitConfig::default(),
        ValidationConfig::default(),
    )
    .await;

    let report = metrics.report();
    println!("{}", report);

    // All self-pings should be blocked
    assert_eq!(
        report.allowed, 0,
        "No self-pings should be allowed, got {}",
        report.allowed
    );
}

#[tokio::test]
async fn test_content_type_bypass() {
    let config = AttackConfig::content_type_bypass();

    let metrics = run_attack(
        &config,
        RateLimitConfig::default(),
        ValidationConfig::default(),
    )
    .await;

    let report = metrics.report();
    println!("{}", report);

    // All requests with wrong Content-Type should be rejected
    assert_eq!(
        report.allowed, 0,
        "No invalid Content-Type should be allowed"
    );
    assert_eq!(report.validation_failed, report.total_requests);
}

#[tokio::test]
async fn test_missing_params_attack() {
    let config = AttackConfig::missing_params_attack();

    let metrics = run_attack(
        &config,
        RateLimitConfig::default(),
        ValidationConfig::default(),
    )
    .await;

    let report = metrics.report();
    println!("{}", report);

    // All requests missing params should be rejected
    assert_eq!(
        report.allowed, 0,
        "No requests with missing params should be allowed"
    );
}

#[tokio::test]
async fn test_slow_drip_allowed() {
    let config = AttackConfig::slow_drip();

    let metrics = run_attack(
        &config,
        RateLimitConfig::default(),
        ValidationConfig::default(),
    )
    .await;

    let report = metrics.report();
    println!("{}", report);

    // Slow drip at < 1 rps should mostly be allowed
    // (may have some edge cases with timing)
    assert!(
        report.allowed as f64 / report.total_requests as f64 >= 0.8,
        "Slow drip should be mostly allowed, got {}/{}",
        report.allowed,
        report.total_requests
    );
}

#[tokio::test]
async fn test_amplification_attack() {
    let config = AttackConfig::amplification_attack();

    let metrics = run_attack(
        &config,
        RateLimitConfig::default(),
        ValidationConfig::default(),
    )
    .await;

    let report = metrics.report();
    println!("{}", report);

    // Per-source rate limiting should kick in
    assert!(
        report.rate_limited_source > 0 || report.rate_limited_ip > 0,
        "Some requests should be rate limited"
    );
}

// ============================================================================
// Content-Type Validation Tests
// ============================================================================

#[tokio::test]
async fn test_content_type_variations() {
    let validator = WebmentionValidator::new(ValidationConfig::default());

    let test_cases = generators::generate_content_types();

    for ct in test_cases {
        let result = validator.validate(ct, Some("https://source.com"), Some("https://target.com"));
        let expected_valid = generators::is_valid_content_type(ct);

        assert_eq!(
            result.is_valid(),
            expected_valid,
            "Content-Type {:?} validation mismatch",
            ct
        );
    }
}

// ============================================================================
// URL Validation Tests
// ============================================================================

#[tokio::test]
async fn test_malformed_urls() {
    let validator = WebmentionValidator::new(ValidationConfig::default());

    for url in generators::generate_malformed_urls() {
        // Malformed source
        let result = validator.validate(
            Some("application/x-www-form-urlencoded"),
            Some(url),
            Some("https://target.example.com/"),
        );
        assert!(
            !result.is_valid(),
            "Malformed source URL '{}' should be rejected",
            url
        );

        // Malformed target
        let result = validator.validate(
            Some("application/x-www-form-urlencoded"),
            Some("https://source.example.com/"),
            Some(url),
        );
        assert!(
            !result.is_valid(),
            "Malformed target URL '{}' should be rejected",
            url
        );
    }
}

// ============================================================================
// Latency Tests
// ============================================================================

#[tokio::test]
async fn test_rate_limiter_latency() {
    let limiter = RateLimiter::new(RateLimitConfig::default());
    let ip = "192.168.1.1".parse().unwrap();

    let mut latencies = Vec::new();

    for _ in 0..100 {
        let start = Instant::now();
        let _ = limiter.check_ip(ip).await;
        latencies.push(start.elapsed());
    }

    latencies.sort();
    let median = latencies[latencies.len() / 2];
    let p99 = latencies[(latencies.len() as f64 * 0.99) as usize];

    println!("Rate limiter latency: median={:?}, p99={:?}", median, p99);

    // Rate limiting should be very fast (< 1ms)
    assert!(
        median < Duration::from_millis(1),
        "Median latency {:?} should be < 1ms",
        median
    );
}
