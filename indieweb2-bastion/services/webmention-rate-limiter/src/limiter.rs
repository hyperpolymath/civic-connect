// SPDX-FileCopyrightText: 2025 Hyperpolymath
// SPDX-License-Identifier: PMPL-1.0-or-later

//! Token bucket rate limiter implementation for Webmention endpoints.
//!
//! Implements dual-layer rate limiting:
//! 1. Per-IP rate limiting (60 rpm default)
//! 2. Per-source URL rate limiting (10 rpm default)
//!
//! Includes burst detection with configurable cooldown.

use crate::config::RateLimitConfig;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Result of a rate limit check.
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    /// Request is allowed
    Allowed {
        /// Remaining requests in current window
        remaining: u32,
        /// Time until window resets
        reset_in: Duration,
    },
    /// Request is rate limited
    Limited {
        /// Reason for rate limiting
        reason: RateLimitReason,
        /// Time until rate limit expires
        retry_after: Duration,
    },
}

/// Reason for rate limiting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitReason {
    /// IP exceeded per-IP rate limit
    IpRateExceeded,
    /// Source URL exceeded per-source rate limit
    SourceRateExceeded,
    /// IP is in cooldown after burst detection
    BurstCooldown,
}

impl std::fmt::Display for RateLimitReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IpRateExceeded => write!(f, "IP rate limit exceeded"),
            Self::SourceRateExceeded => write!(f, "Source URL rate limit exceeded"),
            Self::BurstCooldown => write!(f, "Burst detected, in cooldown"),
        }
    }
}

/// Token bucket for rate limiting.
#[derive(Debug)]
struct TokenBucket {
    /// Available tokens
    tokens: f64,
    /// Maximum tokens (bucket capacity)
    max_tokens: f64,
    /// Token refill rate per second
    refill_rate: f64,
    /// Last time tokens were refilled
    last_refill: Instant,
    /// Request timestamps for burst detection
    request_times: Vec<Instant>,
}

impl TokenBucket {
    fn new(max_rate_per_minute: u32) -> Self {
        let max_tokens = max_rate_per_minute as f64;
        let refill_rate = max_tokens / 60.0; // tokens per second

        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
            request_times: Vec::new(),
        }
    }

    /// Refill tokens based on elapsed time.
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }

    /// Try to consume a token. Returns true if successful.
    fn try_consume(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            self.request_times.push(Instant::now());
            true
        } else {
            false
        }
    }

    /// Get remaining tokens.
    fn remaining(&self) -> u32 {
        self.tokens.floor() as u32
    }

    /// Get time until a token is available.
    fn time_until_available(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            let needed = 1.0 - self.tokens;
            Duration::from_secs_f64(needed / self.refill_rate)
        }
    }

    /// Check if burst activity detected in the last 10 seconds.
    fn detect_burst(&mut self, threshold_multiplier: f32) -> bool {
        let now = Instant::now();
        let window = Duration::from_secs(10);

        // Remove old timestamps
        self.request_times
            .retain(|t| now.duration_since(*t) < window);

        // Burst = more than threshold_multiplier * expected rate in 10s window
        // Use max(1, ...) to ensure we have a sensible minimum threshold
        let expected_in_window = (self.max_tokens / 6.0).max(1.0); // 10s = 1/6 of a minute
        let threshold = (expected_in_window * threshold_multiplier as f64).max(3.0) as usize;

        self.request_times.len() > threshold
    }
}

/// IP cooldown state.
#[derive(Debug)]
struct CooldownState {
    /// When cooldown ends
    until: Instant,
}

/// Thread-safe rate limiter.
pub struct RateLimiter {
    /// Configuration
    config: RateLimitConfig,
    /// Per-IP buckets
    ip_buckets: Arc<RwLock<HashMap<IpAddr, TokenBucket>>>,
    /// Per-source URL buckets
    source_buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    /// IPs in cooldown
    cooldowns: Arc<RwLock<HashMap<IpAddr, CooldownState>>>,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            ip_buckets: Arc::new(RwLock::new(HashMap::new())),
            source_buckets: Arc::new(RwLock::new(HashMap::new())),
            cooldowns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check rate limit for an IP address.
    pub async fn check_ip(&self, ip: IpAddr) -> RateLimitResult {
        // Check cooldown first
        {
            let cooldowns = self.cooldowns.read().await;
            if let Some(state) = cooldowns.get(&ip) {
                let now = Instant::now();
                if now < state.until {
                    let retry_after = state.until.duration_since(now);
                    debug!(%ip, ?retry_after, "IP in cooldown");
                    return RateLimitResult::Limited {
                        reason: RateLimitReason::BurstCooldown,
                        retry_after,
                    };
                }
            }
        }

        // Check and update rate limit
        let mut buckets = self.ip_buckets.write().await;
        let bucket = buckets
            .entry(ip)
            .or_insert_with(|| TokenBucket::new(self.config.max_rate_rpm));

        // Check for burst and apply cooldown
        if bucket.detect_burst(self.config.burst_threshold_multiplier) {
            warn!(%ip, "Burst detected, applying cooldown");
            let cooldown_duration = self.config.cooldown_duration();

            // Apply cooldown
            let mut cooldowns = self.cooldowns.write().await;
            cooldowns.insert(
                ip,
                CooldownState {
                    until: Instant::now() + cooldown_duration,
                },
            );

            return RateLimitResult::Limited {
                reason: RateLimitReason::BurstCooldown,
                retry_after: cooldown_duration,
            };
        }

        if bucket.try_consume() {
            RateLimitResult::Allowed {
                remaining: bucket.remaining(),
                reset_in: self.config.window_duration(),
            }
        } else {
            let retry_after = bucket.time_until_available();
            debug!(%ip, ?retry_after, "IP rate limit exceeded");
            RateLimitResult::Limited {
                reason: RateLimitReason::IpRateExceeded,
                retry_after,
            }
        }
    }

    /// Check rate limit for a source URL.
    pub async fn check_source(&self, source_url: &str) -> RateLimitResult {
        // Normalize source URL (remove query params, fragments)
        let normalized = normalize_source_url(source_url);

        let mut buckets = self.source_buckets.write().await;
        let bucket = buckets
            .entry(normalized.clone())
            .or_insert_with(|| TokenBucket::new(self.config.max_rate_per_source));

        if bucket.try_consume() {
            RateLimitResult::Allowed {
                remaining: bucket.remaining(),
                reset_in: self.config.window_duration(),
            }
        } else {
            let retry_after = bucket.time_until_available();
            debug!(source = %normalized, ?retry_after, "Source rate limit exceeded");
            RateLimitResult::Limited {
                reason: RateLimitReason::SourceRateExceeded,
                retry_after,
            }
        }
    }

    /// Check both IP and source rate limits.
    pub async fn check(&self, ip: IpAddr, source_url: Option<&str>) -> RateLimitResult {
        // Check IP first
        let ip_result = self.check_ip(ip).await;
        if let RateLimitResult::Limited { .. } = ip_result {
            return ip_result;
        }

        // Check source if provided
        if let Some(source) = source_url {
            let source_result = self.check_source(source).await;
            if let RateLimitResult::Limited { .. } = source_result {
                return source_result;
            }
        }

        ip_result
    }

    /// Clean up expired entries (should be called periodically).
    pub async fn cleanup(&self) {
        let now = Instant::now();
        let stale_threshold = Duration::from_secs(300); // 5 minutes

        // Clean up IP buckets
        {
            let mut buckets = self.ip_buckets.write().await;
            buckets.retain(|_, bucket| {
                now.duration_since(bucket.last_refill) < stale_threshold
            });
        }

        // Clean up source buckets
        {
            let mut buckets = self.source_buckets.write().await;
            buckets.retain(|_, bucket| {
                now.duration_since(bucket.last_refill) < stale_threshold
            });
        }

        // Clean up expired cooldowns
        {
            let mut cooldowns = self.cooldowns.write().await;
            cooldowns.retain(|_, state| now < state.until);
        }
    }
}

/// Normalize source URL for rate limiting.
fn normalize_source_url(url: &str) -> String {
    match url::Url::parse(url) {
        Ok(mut parsed) => {
            parsed.set_query(None);
            parsed.set_fragment(None);
            // Normalize to lowercase host
            if let Some(host) = parsed.host_str() {
                let _ = parsed.set_host(Some(&host.to_lowercase()));
            }
            parsed.to_string()
        }
        Err(_) => url.to_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_ip_rate_limiting() {
        let config = RateLimitConfig {
            max_rate_rpm: 5,
            // Disable burst detection for this test by setting high threshold
            burst_threshold_multiplier: 100.0,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // First 5 requests should succeed
        for _ in 0..5 {
            match limiter.check_ip(ip).await {
                RateLimitResult::Allowed { .. } => {}
                RateLimitResult::Limited { .. } => panic!("Should not be limited"),
            }
        }

        // 6th request should be limited
        match limiter.check_ip(ip).await {
            RateLimitResult::Limited { reason, .. } => {
                assert_eq!(reason, RateLimitReason::IpRateExceeded);
            }
            RateLimitResult::Allowed { .. } => panic!("Should be limited"),
        }
    }

    #[tokio::test]
    async fn test_source_rate_limiting() {
        let config = RateLimitConfig {
            max_rate_per_source: 2,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);
        let source = "https://example.com/post/1";

        // First 2 requests should succeed
        for _ in 0..2 {
            match limiter.check_source(source).await {
                RateLimitResult::Allowed { .. } => {}
                RateLimitResult::Limited { .. } => panic!("Should not be limited"),
            }
        }

        // 3rd request should be limited
        match limiter.check_source(source).await {
            RateLimitResult::Limited { reason, .. } => {
                assert_eq!(reason, RateLimitReason::SourceRateExceeded);
            }
            RateLimitResult::Allowed { .. } => panic!("Should be limited"),
        }
    }

    #[test]
    fn test_normalize_source_url() {
        assert_eq!(
            normalize_source_url("https://Example.COM/post/1?foo=bar#section"),
            "https://example.com/post/1"
        );
    }
}
