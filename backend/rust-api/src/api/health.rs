// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! Health check endpoint

use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

/// Health check endpoint
/// GET /health
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
    })
}
