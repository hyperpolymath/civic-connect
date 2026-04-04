// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Benchmarks for the Webmention rate limiter.
//
// Baselines established 2026-04-04.
// All benchmarks use Criterion with wall-time sampling.
//
// To run:
//   cd indieweb2-bastion/services/webmention-rate-limiter
//   cargo bench --bench rate_limiter_bench
//
// Benchmarked operations:
//   - Rate limiter check latency (single IP, hot path)
//   - Rate limiter check with source URL (combined IP + source)
//   - Validator throughput (single call)
//   - Validator throughput (batch, simulating burst)
//   - Source URL normalisation (called on every source check)

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use std::net::{IpAddr, Ipv4Addr};
use webmention_rate_limiter::{
    config::{RateLimitConfig, ValidationConfig},
    limiter::RateLimiter,
    validator::WebmentionValidator,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_limiter() -> RateLimiter {
    RateLimiter::new(RateLimitConfig {
        max_rate_rpm: 1_000_000, // Effectively unlimited so we measure hot-path overhead.
        max_rate_per_source: 1_000_000,
        burst_threshold_multiplier: 1_000_000.0,
        ..Default::default()
    })
}

fn make_validator() -> WebmentionValidator {
    WebmentionValidator::new(ValidationConfig::default())
}

// ---------------------------------------------------------------------------
// Rate limiter benchmarks
// ---------------------------------------------------------------------------

fn bench_check_ip_latency(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("bench: create tokio runtime");
    let limiter = make_limiter();
    let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

    c.bench_function("rate_limiter_check_ip", |b| {
        b.iter(|| rt.block_on(async { limiter.check_ip(ip).await }));
    });
}

fn bench_check_combined_latency(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("bench: create tokio runtime");
    let limiter = make_limiter();
    let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let source = "https://external-blog.example.com/post/1";

    c.bench_function("rate_limiter_check_ip_and_source", |b| {
        b.iter(|| rt.block_on(async { limiter.check(ip, Some(source)).await }));
    });
}

fn bench_check_many_unique_ips(c: &mut Criterion) {
    // Simulates a distributed incoming wave: many unique IPs, each checked once.
    let rt = tokio::runtime::Runtime::new().expect("bench: create tokio runtime");
    let mut group = c.benchmark_group("rate_limiter_unique_ips");

    for ip_count in [100usize, 1_000, 10_000] {
        let ips: Vec<IpAddr> = (0..ip_count)
            .map(|i| {
                let b = ((i >> 16) & 0xFF) as u8;
                let c_byte = ((i >> 8) & 0xFF) as u8;
                let d = (i & 0xFF) as u8;
                IpAddr::V4(Ipv4Addr::new(10, b, c_byte, d))
            })
            .collect();

        group.throughput(Throughput::Elements(ip_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(ip_count),
            &ip_count,
            |b, _| {
                b.iter_batched(
                    || {
                        (
                            make_limiter(),
                            ips.clone(),
                        )
                    },
                    |(lim, ip_list)| {
                        rt.block_on(async {
                            for ip in &ip_list {
                                let _ = lim.check_ip(*ip).await;
                            }
                        });
                    },
                    BatchSize::LargeInput,
                );
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Validator benchmarks
// ---------------------------------------------------------------------------

fn bench_validator_valid_request(c: &mut Criterion) {
    let validator = make_validator();
    c.bench_function("validator_valid_request", |b| {
        b.iter(|| {
            validator.validate(
                Some("application/x-www-form-urlencoded"),
                Some("https://external-blog.example.com/post/1"),
                Some("https://my-site.example.org/article/hello"),
            )
        });
    });
}

fn bench_validator_invalid_content_type(c: &mut Criterion) {
    let validator = make_validator();
    c.bench_function("validator_invalid_content_type", |b| {
        b.iter(|| {
            validator.validate(
                Some("application/json"),
                Some("https://sender.example.com/post/1"),
                Some("https://receiver.example.org/article/2"),
            )
        });
    });
}

fn bench_validator_self_ping(c: &mut Criterion) {
    let validator = make_validator();
    c.bench_function("validator_self_ping_blocked", |b| {
        b.iter(|| {
            validator.validate(
                Some("application/x-www-form-urlencoded"),
                Some("https://blog.example.com/post/1"),
                Some("https://www.example.com/article/2"),
            )
        });
    });
}

fn bench_validator_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("validator_throughput");

    for batch_size in [10usize, 100, 1_000] {
        let validator = make_validator();
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &n| {
                b.iter(|| {
                    for i in 0..n {
                        let source = format!("https://source-{i}.example.com/post/1");
                        let target = format!("https://target-{i}.example.org/article/2");
                        let _ = validator.validate(
                            Some("application/x-www-form-urlencoded"),
                            Some(&source),
                            Some(&target),
                        );
                    }
                });
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion entry-points
// ---------------------------------------------------------------------------

criterion_group!(
    limiter_benches,
    bench_check_ip_latency,
    bench_check_combined_latency,
    bench_check_many_unique_ips,
);

criterion_group!(
    validator_benches,
    bench_validator_valid_request,
    bench_validator_invalid_content_type,
    bench_validator_self_ping,
    bench_validator_throughput,
);

criterion_main!(limiter_benches, validator_benches);
