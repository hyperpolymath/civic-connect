// SPDX-FileCopyrightText: 2025 Hyperpolymath
// SPDX-License-Identifier: PMPL-1.0-or-later

//! Configuration for the Webmention rate limiter.
//!
//! Default values align with the CURPS policy defined in
//! `policy/curps/webmention.ncl`.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the Webmention rate limiter service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server bind address (default: 0.0.0.0:8080)
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,

    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,

    /// Validation configuration
    #[serde(default)]
    pub validation: ValidationConfig,

    /// Metrics configuration
    #[serde(default)]
    pub metrics: MetricsConfig,
}

/// Rate limiting configuration matching CURPS webmention constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per minute per IP (default: 60)
    #[serde(default = "default_max_rate_rpm")]
    pub max_rate_rpm: u32,

    /// Maximum requests per minute per source URL (default: 10)
    #[serde(default = "default_max_rate_per_source")]
    pub max_rate_per_source: u32,

    /// Cooldown period after burst detection in milliseconds (default: 30000)
    #[serde(default = "default_cooldown_ms")]
    pub cooldown_on_burst_ms: u64,

    /// Burst threshold multiplier (default: 3x normal rate in 10s window)
    #[serde(default = "default_burst_threshold")]
    pub burst_threshold_multiplier: f32,

    /// Time window for rate calculation in seconds (default: 60)
    #[serde(default = "default_window_secs")]
    pub window_secs: u64,
}

/// Validation configuration for Webmention requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Required content types (default: application/x-www-form-urlencoded)
    #[serde(default = "default_content_types")]
    pub require_content_type: Vec<String>,

    /// Require source and target parameters (default: true)
    #[serde(default = "default_true")]
    pub require_source_target: bool,

    /// Block self-ping (same domain source/target) (default: true)
    #[serde(default = "default_true")]
    pub block_self_ping: bool,
}

/// Metrics configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable Prometheus metrics endpoint (default: true)
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Metrics endpoint path (default: /metrics)
    #[serde(default = "default_metrics_path")]
    pub path: String,
}

// Default value functions
fn default_bind_addr() -> String {
    "0.0.0.0:8080".to_string()
}

fn default_max_rate_rpm() -> u32 {
    60 // Matches CURPS policy
}

fn default_max_rate_per_source() -> u32 {
    10 // Matches CURPS policy
}

fn default_cooldown_ms() -> u64 {
    30000 // 30 seconds, matches CURPS policy
}

fn default_burst_threshold() -> f32 {
    3.0
}

fn default_window_secs() -> u64 {
    60
}

fn default_content_types() -> Vec<String> {
    vec!["application/x-www-form-urlencoded".to_string()]
}

fn default_true() -> bool {
    true
}

fn default_metrics_path() -> String {
    "/metrics".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_addr: default_bind_addr(),
            rate_limit: RateLimitConfig::default(),
            validation: ValidationConfig::default(),
            metrics: MetricsConfig::default(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_rate_rpm: default_max_rate_rpm(),
            max_rate_per_source: default_max_rate_per_source(),
            cooldown_on_burst_ms: default_cooldown_ms(),
            burst_threshold_multiplier: default_burst_threshold(),
            window_secs: default_window_secs(),
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            require_content_type: default_content_types(),
            require_source_target: default_true(),
            block_self_ping: default_true(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            path: default_metrics_path(),
        }
    }
}

impl RateLimitConfig {
    /// Get the cooldown duration
    pub fn cooldown_duration(&self) -> Duration {
        Duration::from_millis(self.cooldown_on_burst_ms)
    }

    /// Get the rate window duration
    pub fn window_duration(&self) -> Duration {
        Duration::from_secs(self.window_secs)
    }
}
