// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <jonathan.jewell@open.ac.uk>
//
// oDNS Resolver — Oblivious DNS Resolver (Rust rewrite per ADR-0003)
//
// Receives Kyber-1024 encrypted DNS queries from the proxy, decrypts them,
// resolves via upstream DNS, and returns plaintext responses. The resolver
// never sees the client's IP address (privacy guarantee).
//
// FIPS 203: ML-KEM / Kyber-1024 key encapsulation
// CRYPTO-POLICY.adoc: Full cryptographic requirements

use std::fs;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use clap::{Parser, Subcommand};
use hickory_proto::op::Message;
use hickory_proto::serialize::binary::BinDecodable;
use odns_common::crypto::{self, SecretKey};
use odns_common::protocol;
use pqcrypto_traits::kem::{PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Parser)]
#[command(name = "odns-resolver", about = "Oblivious DNS resolver with Kyber-1024 KEM")]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    /// Listen address for encrypted connections from proxy
    #[arg(long, default_value = "[::]:8853")]
    listen: String,

    /// Upstream DNS server (UDP)
    #[arg(long, default_value = "[2606:4700:4700::1111]:53")]
    upstream: String,

    /// Kyber-1024 secret key (base64-encoded, 3168 bytes decoded)
    #[arg(long, conflicts_with = "privkey_file")]
    privkey: Option<String>,

    /// Path to file containing Kyber-1024 secret key (raw bytes)
    #[arg(long, conflicts_with = "privkey")]
    privkey_file: Option<String>,

    /// Key rotation interval (e.g. "24h", "6h")
    #[arg(long, default_value = "24h", value_parser = parse_duration)]
    rotate: Duration,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a new Kyber-1024 keypair and print base64-encoded keys
    Keygen,
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if let Some(hours) = s.strip_suffix('h') {
        let h: u64 = hours.parse().map_err(|e| format!("invalid hours: {e}"))?;
        Ok(Duration::from_secs(h * 3600))
    } else if let Some(mins) = s.strip_suffix('m') {
        let m: u64 = mins.parse().map_err(|e| format!("invalid minutes: {e}"))?;
        Ok(Duration::from_secs(m * 60))
    } else if let Some(secs) = s.strip_suffix('s') {
        let s: u64 = secs.parse().map_err(|e| format!("invalid seconds: {e}"))?;
        Ok(Duration::from_secs(s))
    } else {
        let s: u64 = s.parse().map_err(|e| format!("invalid duration: {e}"))?;
        Ok(Duration::from_secs(s))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();

    // Handle keygen subcommand
    if matches!(args.command, Some(Command::Keygen)) {
        return keygen();
    }

    // Load Kyber-1024 secret key
    let secret_key = load_secret_key(&args)?;
    info!(
        sk_len = secret_key.as_bytes().len(),
        "loaded Kyber-1024 secret key"
    );

    let secret_key = Arc::new(RwLock::new(secret_key));
    let upstream: Arc<str> = args.upstream.clone().into();

    // Start key rotation background task
    if args.rotate > Duration::ZERO {
        let sk = Arc::clone(&secret_key);
        let interval = args.rotate;
        tokio::spawn(async move {
            key_rotation_loop(sk, interval).await;
        });
    }

    // Bind listener
    let listener = TcpListener::bind(&args.listen).await?;
    info!(
        listen = %args.listen,
        upstream = %args.upstream,
        rotate = ?args.rotate,
        "oDNS resolver started (Kyber-1024 KEM, privacy mode)"
    );

    loop {
        let (stream, _peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "accept failed");
                continue;
            }
        };

        // NOTE: We intentionally do NOT log the peer address.
        // The proxy is our only expected peer. Logging it would
        // not reveal client IPs but we keep the privacy discipline.

        let sk = Arc::clone(&secret_key);
        let upstream = Arc::clone(&upstream);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, &sk, &upstream).await {
                error!(error = %e, "connection handler failed");
            }
        });
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    secret_key: &RwLock<SecretKey>,
    upstream: &str,
) -> anyhow::Result<()> {
    // Read encrypted query from proxy
    let encrypted = protocol::read_framed(&mut stream).await?;

    // Decrypt with Kyber-1024 + XChaCha20-Poly1305
    let dns_wire = {
        let sk = secret_key.read().await;
        crypto::decrypt_query(&encrypted, &sk)?
    };

    // Parse DNS query
    let query = Message::from_bytes(&dns_wire)?;
    for q in query.queries() {
        info!(name = %q.name(), qtype = ?q.query_type(), "resolving");
    }

    // Resolve via upstream DNS (UDP)
    let response_wire = resolve_upstream(&dns_wire, upstream).await?;

    // Send plaintext DNS response back to proxy
    protocol::write_framed(&mut stream, &response_wire).await?;

    Ok(())
}

/// Resolve a DNS query via upstream DNS server over UDP.
async fn resolve_upstream(query_wire: &[u8], upstream: &str) -> anyhow::Result<Vec<u8>> {
    let socket = UdpSocket::bind("[::]:0").await?;
    socket.send_to(query_wire, upstream).await?;

    let mut buf = vec![0u8; 4096];
    let (len, _) = tokio::time::timeout(Duration::from_secs(5), socket.recv_from(&mut buf))
        .await
        .map_err(|_| anyhow::anyhow!("upstream DNS timeout"))??;

    buf.truncate(len);

    // Validate the response is parseable DNS
    let _msg = Message::from_bytes(&buf)?;

    Ok(buf)
}

/// Periodically rotate Kyber-1024 keys.
async fn key_rotation_loop(secret_key: Arc<RwLock<SecretKey>>, interval: Duration) {
    loop {
        tokio::time::sleep(interval).await;

        let (pk, sk) = crypto::generate_keypair();
        let b64 = base64::engine::general_purpose::STANDARD;

        info!("rotating Kyber-1024 keys");
        info!(
            new_pubkey = b64.encode(pk.as_bytes()),
            "new public key — update proxy configuration"
        );

        let mut current = secret_key.write().await;
        *current = sk;

        info!("key rotation complete");
    }
}

/// Generate and print a new Kyber-1024 keypair.
fn keygen() -> anyhow::Result<()> {
    let (pk, sk) = crypto::generate_keypair();
    let b64 = base64::engine::general_purpose::STANDARD;

    println!("Kyber-1024 / ML-KEM-1024 Key Pair (FIPS 203)");
    println!("=============================================");
    println!();
    println!(
        "Public Key  ({:>4} bytes, base64):",
        pk.as_bytes().len()
    );
    println!("{}", b64.encode(pk.as_bytes()));
    println!();
    println!(
        "Secret Key  ({:>4} bytes, base64):",
        sk.as_bytes().len()
    );
    println!("{}", b64.encode(sk.as_bytes()));
    println!();
    println!("Store the secret key securely on the resolver (--privkey or --privkey-file).");
    println!("Configure the proxy with the public key (--pubkey or --pubkey-file).");
    println!();
    println!("Algorithm:  Kyber-1024 / ML-KEM-1024 (FIPS 203)");
    println!("KDF:        HKDF-SHA3-512 (RFC 5869 + FIPS 202)");
    println!("Symmetric:  XChaCha20-Poly1305 (extended 192-bit nonce)");

    Ok(())
}

fn load_secret_key(args: &Args) -> anyhow::Result<SecretKey> {
    let bytes = if let Some(ref b64) = args.privkey {
        base64::engine::general_purpose::STANDARD.decode(b64)?
    } else if let Some(ref path) = args.privkey_file {
        fs::read(path)?
    } else {
        anyhow::bail!(
            "provide --privkey (base64) or --privkey-file (raw bytes)\n\
             generate a keypair with: odns-resolver keygen"
        );
    };

    Ok(crypto::secret_key_from_bytes(&bytes)?)
}
