// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <jonathan.jewell@open.ac.uk>
//
// oDNS Proxy — Oblivious DNS Proxy (Rust rewrite per ADR-0003)
//
// Accepts DNS-over-TLS (DoT) queries on port 853, encrypts them with
// Kyber-1024 KEM + XChaCha20-Poly1305, and forwards to the oDNS resolver.
// The resolver never sees the client's IP address.
//
// RFC 9230: Oblivious DNS over HTTPS (adapted for DoT)
// FIPS 203: ML-KEM / Kyber-1024 key encapsulation
// CRYPTO-POLICY.adoc: Full cryptographic requirements

use std::fs;
use std::sync::Arc;

use base64::Engine;
use clap::Parser;
use hickory_proto::op::Message;
use hickory_proto::serialize::binary::BinDecodable;
use odns_common::crypto::{self, PublicKey};
use odns_common::protocol;
use pqcrypto_traits::kem::PublicKey as PublicKeyTrait;
use rustls::ServerConfig;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;
use tracing::{error, info, warn};

#[derive(Parser)]
#[command(name = "odns-proxy", about = "Oblivious DNS proxy with Kyber-1024 KEM")]
struct Args {
    /// Listen address for DoT (DNS-over-TLS) connections
    #[arg(long, default_value = "[::]:853")]
    listen: String,

    /// oDNS resolver address (TCP)
    #[arg(long, default_value = "localhost:8853")]
    resolver: String,

    /// Kyber-1024 public key (base64-encoded, 1568 bytes decoded)
    #[arg(long, conflicts_with = "pubkey_file")]
    pubkey: Option<String>,

    /// Path to file containing Kyber-1024 public key (raw bytes)
    #[arg(long, conflicts_with = "pubkey")]
    pubkey_file: Option<String>,

    /// TLS certificate file (PEM)
    #[arg(long, default_value = "cert.pem")]
    cert: String,

    /// TLS private key file (PEM)
    #[arg(long, default_value = "key.pem")]
    key: String,

    /// IPv6-only mode (reject IPv4-mapped addresses)
    #[arg(long)]
    ipv6_only: bool,
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

    // Load Kyber-1024 public key
    let public_key = load_public_key(&args)?;
    info!(
        pk_len = public_key.as_bytes().len(),
        "loaded Kyber-1024 public key"
    );

    // Load TLS config (TLS 1.3 only)
    let tls_config = load_tls_config(&args.cert, &args.key)?;
    let acceptor = TlsAcceptor::from(Arc::new(tls_config));

    // Bind listener
    let listener = TcpListener::bind(&args.listen).await?;
    info!(
        listen = %args.listen,
        resolver = %args.resolver,
        ipv6_only = args.ipv6_only,
        "oDNS proxy started (DoT, Kyber-1024 KEM)"
    );

    let public_key = Arc::new(public_key);
    let resolver_addr: Arc<str> = args.resolver.into();

    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "accept failed");
                continue;
            }
        };

        if args.ipv6_only && peer.ip().is_ipv4() {
            warn!(peer = %peer, "rejected IPv4 connection (ipv6-only mode)");
            continue;
        }

        let acceptor = acceptor.clone();
        let pk = Arc::clone(&public_key);
        let resolver = Arc::clone(&resolver_addr);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, acceptor, &pk, &resolver).await {
                error!(error = %e, "connection handler failed");
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    acceptor: TlsAcceptor,
    public_key: &PublicKey,
    resolver_addr: &str,
) -> anyhow::Result<()> {
    // TLS handshake
    let mut tls_stream = acceptor.accept(stream).await?;

    // Read DNS query (2-byte length prefix + DNS wire)
    let dns_wire = protocol::read_framed(&mut tls_stream).await?;

    // Parse to log the query name (privacy: we see the query but not in production logs)
    if let Ok(msg) = Message::from_bytes(&dns_wire) {
        for q in msg.queries() {
            info!(name = %q.name(), qtype = ?q.query_type(), "query");
        }
    }

    // Encrypt query with Kyber-1024 KEM + XChaCha20-Poly1305
    let encrypted = crypto::encrypt_query(&dns_wire, public_key)?;

    // Forward to resolver
    let response = forward_to_resolver(resolver_addr, &encrypted).await?;

    // Send response back to client
    protocol::write_framed(&mut tls_stream, &response).await?;

    Ok(())
}

/// Forward an encrypted query to the oDNS resolver and return the DNS response.
async fn forward_to_resolver(resolver_addr: &str, encrypted: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut stream = TcpStream::connect(resolver_addr).await?;

    // Send encrypted query
    protocol::write_framed(&mut stream, encrypted).await?;

    // Read DNS response (plaintext — resolver decrypts and resolves)
    let response = protocol::read_framed(&mut stream).await?;

    Ok(response)
}

fn load_public_key(args: &Args) -> anyhow::Result<PublicKey> {
    let bytes = if let Some(ref b64) = args.pubkey {
        base64::engine::general_purpose::STANDARD.decode(b64)?
    } else if let Some(ref path) = args.pubkey_file {
        fs::read(path)?
    } else {
        anyhow::bail!("provide --pubkey (base64) or --pubkey-file (raw bytes)");
    };

    Ok(crypto::public_key_from_bytes(&bytes)?)
}

fn load_tls_config(cert_path: &str, key_path: &str) -> anyhow::Result<ServerConfig> {
    let cert_pem = fs::read(cert_path)?;
    let key_pem = fs::read(key_path)?;

    let cert_chain: Vec<rustls::pki_types::CertificateDer<'static>> =
        rustls_pemfile::certs(&mut &cert_pem[..]).collect::<Result<Vec<_>, _>>()?;

    let key_der = rustls_pemfile::private_key(&mut &key_pem[..])?
        .ok_or_else(|| anyhow::anyhow!("no private key found in {}", key_path))?;

    let config = ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
        .with_no_client_auth()
        .with_single_cert(cert_chain, key_der)?;

    Ok(config)
}
