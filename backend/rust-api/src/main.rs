// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! CivicConnect REST API Server
//!
//! High-performance API layer for the civic organizing platform.
//! Handles HTTP requests, cryptographic operations, and location services.

use std::net::SocketAddr;

use anyhow::Result;
use axum::{
    http::{header, Method},
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod crypto;
mod db;
mod error;
mod location;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    // Database pool will be added here
    // Redis connection will be added here
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "civicconnect_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    tracing::info!("Starting CivicConnect API server");

    // Build application routes
    let app = create_router();

    // Bind to address
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the application router with all routes
fn create_router() -> Router {
    // CORS configuration - restrict in production
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_origin(tower_http::cors::Any); // TODO: Restrict in production

    Router::new()
        // Health check
        .route("/health", get(api::health::health_check))
        // API v1 routes
        .nest("/api/v1", api_v1_routes())
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

/// API v1 routes
fn api_v1_routes() -> Router {
    Router::new()
        // Authentication
        .route("/auth/register", post(api::auth::register))
        .route("/auth/login", post(api::auth::login))
        // Users
        .route("/users/me", get(api::users::get_current_user))
        .route("/users/:id", get(api::users::get_user))
        // Events
        .route("/events", get(api::events::list_events))
        .route("/events", post(api::events::create_event))
        .route("/events/:id", get(api::events::get_event))
        // Verification
        .route("/verify/qr", post(api::verify::generate_qr))
        .route("/verify/scan", post(api::verify::verify_attendance))
        // Location (privacy-preserving)
        .route("/location/nearby", get(api::location::nearby_events))
}
