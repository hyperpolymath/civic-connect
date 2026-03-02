// SPDX-FileCopyrightText: 2025 Hyperpolymath
// SPDX-License-Identifier: PMPL-1.0-or-later

//! Attack simulation patterns for security testing.

use std::net::IpAddr;
use std::time::Duration;

/// Attack pattern configuration.
#[derive(Debug, Clone)]
pub struct AttackConfig {
    /// Total number of requests to send
    pub total_requests: usize,
    /// Requests per second rate
    pub requests_per_second: f64,
    /// Number of unique IPs to simulate
    pub unique_ips: usize,
    /// Number of unique source URLs
    pub unique_sources: usize,
    /// Whether to use valid Content-Type
    pub valid_content_type: bool,
    /// Whether to include source/target params
    pub include_params: bool,
    /// Percentage of self-ping attempts (0.0-1.0)
    pub self_ping_ratio: f64,
}

impl Default for AttackConfig {
    fn default() -> Self {
        Self {
            total_requests: 100,
            requests_per_second: 10.0,
            unique_ips: 1,
            unique_sources: 1,
            valid_content_type: true,
            include_params: true,
            self_ping_ratio: 0.0,
        }
    }
}

/// Predefined attack patterns.
impl AttackConfig {
    /// Single IP flood - simulates basic DoS from one source.
    pub fn single_ip_flood() -> Self {
        Self {
            total_requests: 200,
            requests_per_second: 100.0,
            unique_ips: 1,
            unique_sources: 10,
            ..Default::default()
        }
    }

    /// Distributed attack - many IPs, low rate each.
    pub fn distributed_attack() -> Self {
        Self {
            total_requests: 500,
            requests_per_second: 50.0,
            unique_ips: 100,
            unique_sources: 50,
            ..Default::default()
        }
    }

    /// Amplification attack - few IPs targeting many sources.
    pub fn amplification_attack() -> Self {
        Self {
            total_requests: 100,
            requests_per_second: 20.0,
            unique_ips: 5,
            unique_sources: 1, // Same victim source
            ..Default::default()
        }
    }

    /// Self-ping attack - attempts internal loops.
    pub fn self_ping_attack() -> Self {
        Self {
            total_requests: 50,
            requests_per_second: 10.0,
            unique_ips: 10,
            unique_sources: 10,
            self_ping_ratio: 1.0, // All self-pings
            ..Default::default()
        }
    }

    /// Content-Type bypass attempts.
    pub fn content_type_bypass() -> Self {
        Self {
            total_requests: 50,
            requests_per_second: 10.0,
            unique_ips: 5,
            unique_sources: 5,
            valid_content_type: false,
            ..Default::default()
        }
    }

    /// Missing parameter attack.
    pub fn missing_params_attack() -> Self {
        Self {
            total_requests: 50,
            requests_per_second: 10.0,
            unique_ips: 5,
            unique_sources: 5,
            include_params: false,
            ..Default::default()
        }
    }

    /// Burst attack - high rate for short duration.
    pub fn burst_attack() -> Self {
        Self {
            total_requests: 50,
            requests_per_second: 500.0, // Very high burst
            unique_ips: 1,
            unique_sources: 10,
            ..Default::default()
        }
    }

    /// Slow drip - stay just under rate limits.
    pub fn slow_drip() -> Self {
        Self {
            total_requests: 100,
            requests_per_second: 0.9, // Just under 1/sec = 54/min < 60 limit
            unique_ips: 1,
            unique_sources: 5,
            ..Default::default()
        }
    }

    /// Calculate expected duration for the attack.
    pub fn expected_duration(&self) -> Duration {
        Duration::from_secs_f64(self.total_requests as f64 / self.requests_per_second)
    }
}

/// Result of an attack simulation.
#[derive(Debug, Clone, Default)]
pub struct AttackResult {
    /// Total requests sent
    pub total_sent: usize,
    /// Requests that were allowed
    pub allowed: usize,
    /// Requests blocked by rate limit
    pub rate_limited: usize,
    /// Requests blocked by validation
    pub validation_failed: usize,
    /// Requests blocked by burst detection
    pub burst_blocked: usize,
    /// Duration of the attack
    pub duration: Duration,
}

impl AttackResult {
    /// Calculate block rate (0.0-1.0).
    pub fn block_rate(&self) -> f64 {
        if self.total_sent == 0 {
            0.0
        } else {
            1.0 - (self.allowed as f64 / self.total_sent as f64)
        }
    }

    /// Calculate effective requests per second allowed.
    pub fn effective_rps(&self) -> f64 {
        self.allowed as f64 / self.duration.as_secs_f64()
    }

    /// Check if attack was successfully mitigated.
    pub fn is_mitigated(&self, max_allowed_ratio: f64) -> bool {
        let allowed_ratio = self.allowed as f64 / self.total_sent as f64;
        allowed_ratio <= max_allowed_ratio
    }
}

/// Expected outcomes for different attack types.
pub struct AttackExpectations {
    /// Maximum ratio of requests that should be allowed
    pub max_allowed_ratio: f64,
    /// Minimum ratio that should be rate limited
    pub min_rate_limited_ratio: f64,
    /// Description of expected behavior
    pub description: &'static str,
}

impl AttackConfig {
    /// Get expected outcomes for this attack pattern.
    pub fn expectations(&self) -> AttackExpectations {
        if !self.valid_content_type {
            AttackExpectations {
                max_allowed_ratio: 0.0,
                min_rate_limited_ratio: 0.0,
                description: "All requests should fail Content-Type validation",
            }
        } else if !self.include_params {
            AttackExpectations {
                max_allowed_ratio: 0.0,
                min_rate_limited_ratio: 0.0,
                description: "All requests should fail parameter validation",
            }
        } else if self.self_ping_ratio >= 1.0 {
            AttackExpectations {
                max_allowed_ratio: 0.0,
                min_rate_limited_ratio: 0.0,
                description: "All requests should be blocked as self-pings",
            }
        } else if self.unique_ips == 1 && self.requests_per_second > 100.0 {
            AttackExpectations {
                max_allowed_ratio: 0.1,
                min_rate_limited_ratio: 0.8,
                description: "Burst should trigger cooldown, most requests blocked",
            }
        } else if self.unique_ips == 1 {
            // Single IP, rate limited to 60/min = 1/sec
            let expected_allowed = 60.0 / self.requests_per_second;
            AttackExpectations {
                max_allowed_ratio: expected_allowed.min(1.0),
                min_rate_limited_ratio: (1.0 - expected_allowed).max(0.0),
                description: "Single IP should be capped at 60 rpm",
            }
        } else {
            // Distributed attack - harder to mitigate at this layer
            AttackExpectations {
                max_allowed_ratio: 0.8,
                min_rate_limited_ratio: 0.1,
                description: "Distributed attack partially mitigated per-IP",
            }
        }
    }
}
