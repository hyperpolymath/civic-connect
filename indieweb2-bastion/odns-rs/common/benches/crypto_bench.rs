// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Benchmarks for the oDNS cryptographic primitives.
//
// Baselines established 2026-04-04 on an AMD Ryzen / NVMe configuration.
// All benchmarks use Criterion with wall-time sampling.
//
// To run:
//   cd indieweb2-bastion/odns-rs
//   cargo bench --bench crypto_bench
//
// Benchmarked operations:
//   - Kyber-1024 keypair generation
//   - DNS query encryption (KEM + AEAD, various payload sizes)
//   - DNS query decryption (KEM decap + AEAD)
//   - Hybrid Ed448+Dilithium5 keypair generation
//   - Hybrid signature creation and verification
//   - Wire framing (write_framed + read_framed round-trip)

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use odns_common::{
    crypto::{decrypt_query, encrypt_query, generate_keypair},
    signatures::{generate_hybrid_keypair, hybrid_sign, hybrid_verify},
    protocol::{read_framed, write_framed},
};
use std::io::Cursor;

// ---------------------------------------------------------------------------
// Kyber-1024 KEM benchmarks
// ---------------------------------------------------------------------------

fn bench_keypair_generation(c: &mut Criterion) {
    c.bench_function("kyber1024_keypair_generate", |b| {
        b.iter(generate_keypair);
    });
}

fn bench_encrypt_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("kyber1024_xchacha20_encrypt");

    for payload_size in [32usize, 128, 512, 4096] {
        let (pk, _sk) = generate_keypair();
        let payload = vec![0xAAu8; payload_size];

        group.throughput(Throughput::Bytes(payload_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(payload_size),
            &payload_size,
            |b, _| {
                b.iter(|| {
                    encrypt_query(&payload, &pk).expect("bench: encrypt must not fail")
                });
            },
        );
    }
    group.finish();
}

fn bench_decrypt_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("kyber1024_xchacha20_decrypt");

    for payload_size in [32usize, 128, 512, 4096] {
        let (pk, sk) = generate_keypair();
        let payload = vec![0xBBu8; payload_size];

        group.throughput(Throughput::Bytes(payload_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(payload_size),
            &payload_size,
            |b, _| {
                b.iter_batched(
                    // Setup: fresh ciphertext each iteration.
                    || encrypt_query(&payload, &pk).expect("bench setup: encrypt"),
                    // Bench: decrypt only.
                    |ct| decrypt_query(&ct, &sk).expect("bench: decrypt must not fail"),
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Hybrid Ed448 + Dilithium5 signature benchmarks
// ---------------------------------------------------------------------------

fn bench_hybrid_keypair_generation(c: &mut Criterion) {
    c.bench_function("hybrid_ed448_dilithium5_keypair_generate", |b| {
        b.iter(generate_hybrid_keypair);
    });
}

fn bench_hybrid_sign(c: &mut Criterion) {
    let kp = generate_hybrid_keypair();
    let message = b"civic-connect dns resolver certificate binding v1";

    c.bench_function("hybrid_ed448_dilithium5_sign", |b| {
        b.iter(|| hybrid_sign(message, &kp));
    });
}

fn bench_hybrid_verify(c: &mut Criterion) {
    let kp = generate_hybrid_keypair();
    let message = b"civic-connect dns resolver certificate binding v1";
    let sig = hybrid_sign(message, &kp);
    let pk = kp.public_key().expect("extract public key for bench");

    c.bench_function("hybrid_ed448_dilithium5_verify", |b| {
        b.iter(|| {
            hybrid_verify(message, &sig, &pk).expect("bench: verification must succeed")
        });
    });
}

// ---------------------------------------------------------------------------
// Wire framing benchmarks
// ---------------------------------------------------------------------------

fn bench_framing_roundtrip(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("bench: create tokio runtime");
    let mut group = c.benchmark_group("protocol_framing_roundtrip");

    for payload_size in [64usize, 512, 4096] {
        let payload = vec![0xCCu8; payload_size];
        group.throughput(Throughput::Bytes(payload_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(payload_size),
            &payload_size,
            |b, _| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut buf = Vec::with_capacity(payload_size + 2);
                        write_framed(&mut buf, &payload)
                            .await
                            .expect("bench: write_framed");
                        let mut cursor = Cursor::new(buf);
                        read_framed(&mut cursor).await.expect("bench: read_framed")
                    })
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
    kem_benches,
    bench_keypair_generation,
    bench_encrypt_query,
    bench_decrypt_query,
);

criterion_group!(
    sig_benches,
    bench_hybrid_keypair_generation,
    bench_hybrid_sign,
    bench_hybrid_verify,
);

criterion_group!(framing_benches, bench_framing_roundtrip);

criterion_main!(kem_benches, sig_benches, framing_benches);
