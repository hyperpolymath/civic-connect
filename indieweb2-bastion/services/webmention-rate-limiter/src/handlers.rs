// SPDX-FileCopyrightText: 2025 Hyperpolymath
// SPDX-License-Identifier: PMPL-1.0-or-later

//! HTTP handlers for the Webmention rate limiter service.
//!
//! This service operates as a reverse proxy filter, validating and
//! rate-limiting requests before forwarding them upstream.

use crate::config::Config;
use crate::limiter::{RateLimitResult, RateLimiter};
use crate::validator::{ValidationResult, WebmentionValidator};
use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{header, HeaderMap, Request, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Shared application state.
pub struct AppState {
    pub limiter: RateLimiter,
    pub validator: WebmentionValidator,
    pub config: Config,
}

/// Error response body.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_secs: Option<u64>,
}

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
}

/// Rate limit check request (for external validation).
#[derive(Debug, Deserialize)]
pub struct CheckRequest {
    pub ip: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub content_type: Option<String>,
}

/// Rate limit check response.
#[derive(Debug, Serialize)]
pub struct CheckResponse {
    pub allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining: Option<u32>,
}

/// Health check endpoint.
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        service: "webmention-rate-limiter",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Check rate limit and validation for a Webmention request.
///
/// This endpoint is called by Envoy or another reverse proxy to validate
/// requests before forwarding them to the backend.
pub async fn check(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CheckRequest>,
) -> impl IntoResponse {
    debug!(
        ip = %req.ip,
        source = ?req.source,
        target = ?req.target,
        content_type = ?req.content_type,
        "Processing rate limit check"
    );

    // Parse IP address
    let ip = match req.ip.parse() {
        Ok(ip) => ip,
        Err(_) => {
            warn!(ip = %req.ip, "Invalid IP address format");
            return (
                StatusCode::BAD_REQUEST,
                Json(CheckResponse {
                    allowed: false,
                    reason: Some("Invalid IP address format".to_string()),
                    retry_after_secs: None,
                    remaining: None,
                }),
            );
        }
    };

    // Validate request
    let validation = state.validator.validate(
        req.content_type.as_deref(),
        req.source.as_deref(),
        req.target.as_deref(),
    );

    if let ValidationResult::Invalid(err) = validation {
        info!(ip = %req.ip, error = %err, "Validation failed");
        return (
            StatusCode::OK, // Return 200 so Envoy can read the body
            Json(CheckResponse {
                allowed: false,
                reason: Some(err.to_string()),
                retry_after_secs: None,
                remaining: None,
            }),
        );
    }

    // Check rate limits
    let rate_result = state.limiter.check(ip, req.source.as_deref()).await;

    match rate_result {
        RateLimitResult::Allowed { remaining, .. } => {
            debug!(ip = %req.ip, remaining, "Request allowed");
            (
                StatusCode::OK,
                Json(CheckResponse {
                    allowed: true,
                    reason: None,
                    retry_after_secs: None,
                    remaining: Some(remaining),
                }),
            )
        }
        RateLimitResult::Limited { reason, retry_after } => {
            info!(
                ip = %req.ip,
                reason = %reason,
                retry_after_secs = retry_after.as_secs(),
                "Request rate limited"
            );
            (
                StatusCode::OK,
                Json(CheckResponse {
                    allowed: false,
                    reason: Some(reason.to_string()),
                    retry_after_secs: Some(retry_after.as_secs()),
                    remaining: None,
                }),
            )
        }
    }
}

/// Middleware-style handler for direct proxying.
///
/// This can be used when the rate limiter sits directly in the request path
/// rather than as an external auth service.
pub async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request<Body>,
) -> Response {
    let ip = addr.ip();
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());

    debug!(
        ip = %ip,
        path = %request.uri().path(),
        content_type = ?content_type,
        "Processing proxied request"
    );

    // Validate Content-Type
    let ct_validation = state.validator.validate_content_type(content_type);
    if let ValidationResult::Invalid(err) = ct_validation {
        return (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Json(ErrorResponse {
                error: err.to_string(),
                code: "INVALID_CONTENT_TYPE",
                retry_after_secs: None,
            }),
        )
            .into_response();
    }

    // Check rate limits
    let rate_result = state.limiter.check_ip(ip).await;

    match rate_result {
        RateLimitResult::Allowed { remaining, .. } => {
            debug!(ip = %ip, remaining, "Request allowed, forwarding");
            // In a real implementation, we would forward to upstream here
            // For MVP, we return a success indicating the request can proceed
            (
                StatusCode::OK,
                [
                    ("X-RateLimit-Remaining", remaining.to_string()),
                    ("X-Webmention-Validated", "true".to_string()),
                ],
                "Request validated successfully",
            )
                .into_response()
        }
        RateLimitResult::Limited { reason, retry_after } => {
            let retry_secs = retry_after.as_secs();
            (
                StatusCode::TOO_MANY_REQUESTS,
                [("Retry-After", retry_secs.to_string())],
                Json(ErrorResponse {
                    error: reason.to_string(),
                    code: "RATE_LIMITED",
                    retry_after_secs: Some(retry_secs),
                }),
            )
                .into_response()
        }
    }
}
