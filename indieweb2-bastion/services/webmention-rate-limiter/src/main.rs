// SPDX-FileCopyrightText: 2025 Hyperpolymath
// SPDX-License-Identifier: PMPL-1.0-or-later

//! Webmention Rate Limiter Service
//!
//! An ingress-level rate limiter and validator for Webmention endpoints.
//! Implements the constraints defined in the CURPS policy:
//!
//! - 60 rpm per IP (default)
//! - 10 rpm per source URL (default)
//! - Content-Type validation
//! - Source/target parameter validation
//! - Self-ping blocking
//! - Burst detection with cooldown
//!
//! ## Usage
//!
//! The service provides two modes of operation:
//!
//! 1. **External auth service**: Envoy or another proxy calls `/check` to
//!    validate requests before forwarding.
//!
//! 2. **Direct proxy**: Requests are sent directly through the service,
//!    which validates and forwards them.
//!
//! ## Configuration
//!
//! Configuration is loaded from environment variables or a config file:
//!
//! - `BIND_ADDR`: Server bind address (default: 0.0.0.0:8080)
//! - `MAX_RATE_RPM`: Max requests per minute per IP (default: 60)
//! - `MAX_RATE_PER_SOURCE`: Max requests per minute per source URL (default: 10)
//! - `COOLDOWN_MS`: Burst cooldown in milliseconds (default: 30000)

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use webmention_rate_limiter::{
    config::Config,
    handlers::{check, health, AppState},
    limiter::RateLimiter,
    validator::WebmentionValidator,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer().json())
        .with(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .init();

    // Load configuration
    let config = load_config();
    info!(
        bind_addr = %config.bind_addr,
        max_rate_rpm = config.rate_limit.max_rate_rpm,
        max_rate_per_source = config.rate_limit.max_rate_per_source,
        cooldown_ms = config.rate_limit.cooldown_on_burst_ms,
        "Starting Webmention rate limiter"
    );

    // Create application state
    let limiter = RateLimiter::new(config.rate_limit.clone());
    let validator = WebmentionValidator::new(config.validation.clone());

    let state = Arc::new(AppState {
        limiter,
        validator,
        config: config.clone(),
    });

    // Spawn cleanup task
    let cleanup_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            cleanup_state.limiter.cleanup().await;
        }
    });

    // Build router
    let app = Router::new()
        .route("/health", get(health))
        .route("/healthz", get(health))
        .route("/check", post(check))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr: SocketAddr = config.bind_addr.parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!(addr = %addr, "Server listening");

    axum::serve(listener, app).await?;

    Ok(())
}

/// Load configuration from environment variables.
fn load_config() -> Config {
    Config {
        bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
        rate_limit: webmention_rate_limiter::config::RateLimitConfig {
            max_rate_rpm: std::env::var("MAX_RATE_RPM")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            max_rate_per_source: std::env::var("MAX_RATE_PER_SOURCE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            cooldown_on_burst_ms: std::env::var("COOLDOWN_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30000),
            ..Default::default()
        },
        ..Default::default()
    }
}
