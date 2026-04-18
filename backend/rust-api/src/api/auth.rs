// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! Authentication endpoints
//!
//! Security considerations:
//! - Passwords hashed with Argon2
//! - JWT tokens with 24-hour expiry
//! - Rate limiting on login attempts
//! - No PII in logs

use axum::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::error::{ApiError, Result};

/// Registration request
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3, max = 64))]
    pub username: String,
    #[validate(length(min = 12))]
    pub password: String,
}

/// Login request
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

/// Authentication response with JWT token
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub level: u8,
}

/// Register a new user
/// POST /api/v1/auth/register
pub async fn register(Json(req): Json<RegisterRequest>) -> Result<Json<AuthResponse>> {
    // Validate input
    req.validate()
        .map_err(|e| ApiError::InvalidInput(e.to_string()))?;

    // TODO: Check if email already exists
    // TODO: Hash password with Argon2
    // TODO: Create user in database
    // TODO: Generate JWT token

    // Placeholder response
    Ok(Json(AuthResponse {
        token: "placeholder_token".to_string(),
        user_id: "placeholder_id".to_string(),
        username: req.username,
        level: 0,
    }))
}

/// Login existing user
/// POST /api/v1/auth/login
pub async fn login(Json(req): Json<LoginRequest>) -> Result<Json<AuthResponse>> {
    // Validate input
    req.validate()
        .map_err(|e| ApiError::InvalidInput(e.to_string()))?;

    // TODO: Look up user by email hash
    // TODO: Verify password with Argon2
    // TODO: Generate JWT token
    // TODO: Update last_active timestamp

    // Placeholder - would return InvalidCredentials for bad login
    Err(ApiError::InvalidCredentials)
}
