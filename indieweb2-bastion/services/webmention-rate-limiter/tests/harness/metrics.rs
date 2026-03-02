// SPDX-FileCopyrightText: 2025 Hyperpolymath
// SPDX-License-Identifier: PMPL-1.0-or-later

//! Metrics collection for attack simulation results.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Collects metrics during attack simulation.
#[derive(Debug, Default)]
pub struct AttackMetrics {
    /// Start time of the attack
    start_time: Option<Instant>,
    /// End time of the attack
    end_time: Option<Instant>,
    /// Count of requests by outcome
    outcomes: HashMap<Outcome, usize>,
    /// Count of requests by IP
    requests_per_ip: HashMap<String, usize>,
    /// Count of requests by source URL
    requests_per_source: HashMap<String, usize>,
    /// Latency samples (microseconds)
    latencies: Vec<u64>,
}

/// Possible outcomes for a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    Allowed,
    RateLimitedIp,
    RateLimitedSource,
    BurstBlocked,
    InvalidContentType,
    MissingParams,
    InvalidUrl,
    SelfPingBlocked,
}

impl AttackMetrics {
    /// Create a new metrics collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark the start of an attack.
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Mark the end of an attack.
    pub fn finish(&mut self) {
        self.end_time = Some(Instant::now());
    }

    /// Record a request outcome.
    pub fn record(&mut self, outcome: Outcome, ip: &str, source: Option<&str>, latency: Duration) {
        *self.outcomes.entry(outcome).or_insert(0) += 1;
        *self.requests_per_ip.entry(ip.to_string()).or_insert(0) += 1;
        if let Some(s) = source {
            *self.requests_per_source.entry(s.to_string()).or_insert(0) += 1;
        }
        self.latencies.push(latency.as_micros() as u64);
    }

    /// Get total request count.
    pub fn total_requests(&self) -> usize {
        self.outcomes.values().sum()
    }

    /// Get count for a specific outcome.
    pub fn count(&self, outcome: Outcome) -> usize {
        self.outcomes.get(&outcome).copied().unwrap_or(0)
    }

    /// Get duration of the attack.
    pub fn duration(&self) -> Duration {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => end.duration_since(start),
            (Some(start), None) => start.elapsed(),
            _ => Duration::ZERO,
        }
    }

    /// Get requests per second.
    pub fn requests_per_second(&self) -> f64 {
        let secs = self.duration().as_secs_f64();
        if secs > 0.0 {
            self.total_requests() as f64 / secs
        } else {
            0.0
        }
    }

    /// Get block rate (ratio of blocked to total).
    pub fn block_rate(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            return 0.0;
        }
        let allowed = self.count(Outcome::Allowed);
        (total - allowed) as f64 / total as f64
    }

    /// Get median latency in microseconds.
    pub fn median_latency_us(&self) -> u64 {
        if self.latencies.is_empty() {
            return 0;
        }
        let mut sorted = self.latencies.clone();
        sorted.sort_unstable();
        sorted[sorted.len() / 2]
    }

    /// Get p99 latency in microseconds.
    pub fn p99_latency_us(&self) -> u64 {
        if self.latencies.is_empty() {
            return 0;
        }
        let mut sorted = self.latencies.clone();
        sorted.sort_unstable();
        let idx = (sorted.len() as f64 * 0.99) as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    /// Get number of unique IPs that made requests.
    pub fn unique_ips(&self) -> usize {
        self.requests_per_ip.len()
    }

    /// Get number of unique source URLs.
    pub fn unique_sources(&self) -> usize {
        self.requests_per_source.len()
    }

    /// Generate a summary report.
    pub fn report(&self) -> MetricsReport {
        MetricsReport {
            total_requests: self.total_requests(),
            allowed: self.count(Outcome::Allowed),
            rate_limited_ip: self.count(Outcome::RateLimitedIp),
            rate_limited_source: self.count(Outcome::RateLimitedSource),
            burst_blocked: self.count(Outcome::BurstBlocked),
            validation_failed: self.count(Outcome::InvalidContentType)
                + self.count(Outcome::MissingParams)
                + self.count(Outcome::InvalidUrl)
                + self.count(Outcome::SelfPingBlocked),
            duration_ms: self.duration().as_millis() as u64,
            requests_per_second: self.requests_per_second(),
            block_rate: self.block_rate(),
            median_latency_us: self.median_latency_us(),
            p99_latency_us: self.p99_latency_us(),
            unique_ips: self.unique_ips(),
            unique_sources: self.unique_sources(),
        }
    }
}

/// Summary report of attack metrics.
#[derive(Debug, Clone)]
pub struct MetricsReport {
    pub total_requests: usize,
    pub allowed: usize,
    pub rate_limited_ip: usize,
    pub rate_limited_source: usize,
    pub burst_blocked: usize,
    pub validation_failed: usize,
    pub duration_ms: u64,
    pub requests_per_second: f64,
    pub block_rate: f64,
    pub median_latency_us: u64,
    pub p99_latency_us: u64,
    pub unique_ips: usize,
    pub unique_sources: usize,
}

impl std::fmt::Display for MetricsReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Attack Metrics Report ===")?;
        writeln!(f, "Duration:          {} ms", self.duration_ms)?;
        writeln!(f, "Total Requests:    {}", self.total_requests)?;
        writeln!(f, "Requests/sec:      {:.2}", self.requests_per_second)?;
        writeln!(f)?;
        writeln!(f, "--- Outcomes ---")?;
        writeln!(f, "Allowed:           {} ({:.1}%)",
            self.allowed,
            self.allowed as f64 / self.total_requests as f64 * 100.0)?;
        writeln!(f, "Rate Limited (IP): {}", self.rate_limited_ip)?;
        writeln!(f, "Rate Limited (Src):{}", self.rate_limited_source)?;
        writeln!(f, "Burst Blocked:     {}", self.burst_blocked)?;
        writeln!(f, "Validation Failed: {}", self.validation_failed)?;
        writeln!(f, "Block Rate:        {:.1}%", self.block_rate * 100.0)?;
        writeln!(f)?;
        writeln!(f, "--- Latency ---")?;
        writeln!(f, "Median:            {} us", self.median_latency_us)?;
        writeln!(f, "P99:               {} us", self.p99_latency_us)?;
        writeln!(f)?;
        writeln!(f, "--- Distribution ---")?;
        writeln!(f, "Unique IPs:        {}", self.unique_ips)?;
        writeln!(f, "Unique Sources:    {}", self.unique_sources)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collection() {
        let mut metrics = AttackMetrics::new();
        metrics.start();

        metrics.record(Outcome::Allowed, "10.0.0.1", Some("https://a.com"), Duration::from_micros(100));
        metrics.record(Outcome::Allowed, "10.0.0.1", Some("https://b.com"), Duration::from_micros(150));
        metrics.record(Outcome::RateLimitedIp, "10.0.0.1", Some("https://c.com"), Duration::from_micros(50));

        metrics.finish();

        assert_eq!(metrics.total_requests(), 3);
        assert_eq!(metrics.count(Outcome::Allowed), 2);
        assert_eq!(metrics.count(Outcome::RateLimitedIp), 1);
        assert_eq!(metrics.unique_ips(), 1);
        assert_eq!(metrics.unique_sources(), 3);
    }

    #[test]
    fn test_block_rate() {
        let mut metrics = AttackMetrics::new();
        for _ in 0..3 {
            metrics.record(Outcome::Allowed, "10.0.0.1", None, Duration::ZERO);
        }
        for _ in 0..7 {
            metrics.record(Outcome::RateLimitedIp, "10.0.0.1", None, Duration::ZERO);
        }

        assert!((metrics.block_rate() - 0.7).abs() < 0.01);
    }
}
