// SPDX-License-Identifier: PMPL-1.0-or-later
//! GraphQL DNS API server for indieweb2-bastion
//!
//! Features:
//! - Full DNS RR coverage (A, AAAA, CNAME, MX, TXT, SRV, CAA, TLSA, NS, SOA, PTR)
//! - DNSSEC zone management
//! - Blockchain provenance anchoring (Ethereum/Polygon)
//! - SurrealDB graph storage
//! - Reverse DNS lookups

use async_graphql::{http::GraphiQLSource, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::{info, Level};

mod blockchain;
mod consent;
mod db;
mod dnssec;
mod error;
mod models;
mod policy;
mod resolvers;
mod schema;

use crate::{
    consent::ConsentClient,
    db::Database,
    policy::PolicyEnforcer,
    resolvers::{MutationRoot, QueryRoot},
};

use std::sync::Arc;
use tokio::sync::RwLock;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub policy: Arc<RwLock<PolicyEnforcer>>,
    pub consent: Arc<ConsentClient>,
}

/// GraphQL handler
async fn graphql_handler(
    State(state): State<AppState>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(state.db)
        .data(state.policy)
        .data(state.consent)
        .finish();

    schema.execute(req.into_inner()).await.into()
}

/// GraphiQL playground handler
async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

/// Health check handler
async fn health() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    info!("Starting GraphQL DNS API server...");

    // Initialize database connection
    let db = Database::connect("memory").await?;
    info!("Connected to SurrealDB");

    // Load Nickel policy configuration
    let policy_path = std::path::Path::new("../policy/curps/policy.ncl");
    let policy_enforcer = match PolicyEnforcer::from_nickel_file(policy_path) {
        Ok(enforcer) => {
            info!("Loaded CURPS policy v{}", enforcer.policy().version);
            Arc::new(RwLock::new(enforcer))
        }
        Err(e) => {
            tracing::warn!("Failed to load policy file ({}), using default policy", e);
            // Create a default policy enforcer for development
            let default_policy = policy::Policy {
                version: "0.1.0".to_string(),
                capabilities: std::collections::HashMap::new(),
                mutations: vec![],
                roles: vec![],
                routes: vec![],
                consent_bindings: vec![],
                constraints: policy::Constraints {
                    require_mtls: false,
                    log_all_mutations: true,
                    max_rate_rpm: 120,
                },
            };
            Arc::new(RwLock::new(PolicyEnforcer {
                policy: default_policy,
                proposals: std::collections::HashMap::new(),
            }))
        }
    };

    // Initialize consent client
    let consent_api_url = std::env::var("CONSENT_API_URL")
        .unwrap_or_else(|_| "http://localhost:8082".to_string());
    let consent_client = Arc::new(ConsentClient::new(consent_api_url.clone()));

    // Test consent API connection
    match consent_client.health_check().await {
        Ok(true) => info!("✓ Connected to consent API at {}", consent_api_url),
        Ok(false) => tracing::warn!("Consent API health check failed"),
        Err(e) => tracing::warn!("Could not connect to consent API ({}), consent checks will fail", e),
    }

    // Create application state
    let state = AppState {
        db,
        policy: policy_enforcer,
        consent: consent_client,
    };

    // Build router with restrictive CORS per security policy
    let allowed_origins = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "https://localhost".to_string());
    let origins: Vec<http::HeaderValue> = allowed_origins
        .split(',')
        .filter_map(|o| o.trim().parse().ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([http::Method::GET, http::Method::POST, http::Method::OPTIONS])
        .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION]);

    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphiql", get(graphiql))
        .route("/health", get(health))
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("GraphQL server listening on http://{}", addr);
    info!("GraphiQL playground: http://{}/graphiql", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
