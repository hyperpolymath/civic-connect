// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! User endpoints
//!
//! Privacy: User location stored as H3 cell, never coordinates

use axum::{extract::Path, Json};
use serde::Serialize;
use uuid::Uuid;

use crate::error::{ApiError, Result};

/// Public user profile (minimal PII)
#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub level: u8,
    pub events_attended: u32,
    pub events_organized: u32,
    pub member_since: String,
    // Note: No email, no location - privacy first
}

/// Get current authenticated user
/// GET /api/v1/users/me
pub async fn get_current_user() -> Result<Json<UserProfile>> {
    // TODO: Extract user from JWT token
    // TODO: Look up user in database

    Err(ApiError::Unauthorized)
}

/// Get user by ID (public profile only)
/// GET /api/v1/users/:id
pub async fn get_user(Path(id): Path<Uuid>) -> Result<Json<UserProfile>> {
    // TODO: Look up user in database
    // TODO: Return only public profile fields

    let _ = id; // Suppress unused warning
    Err(ApiError::UserNotFound)
}
