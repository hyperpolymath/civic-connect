// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! Event endpoints
//!
//! Location privacy: Events stored with H3 cell, not exact coordinates
//! Discovery uses proximity queries on cell neighborhoods

use axum::{extract::Path, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::error::{ApiError, Result};

/// Event listing response
#[derive(Debug, Serialize)]
pub struct EventSummary {
    pub id: Uuid,
    pub title: String,
    pub organizer_username: String,
    pub organizer_level: u8,
    pub start_time: DateTime<Utc>,
    pub location_cell: String, // H3 cell ID
    pub attendee_count: u32,
    pub capacity: Option<u32>,
    pub tags: Vec<String>,
}

/// Full event details
#[derive(Debug, Serialize)]
pub struct EventDetails {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub organizer_id: Uuid,
    pub organizer_username: String,
    pub organizer_level: u8,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub location_cell: String,
    pub attendee_count: u32,
    pub capacity: Option<u32>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Create event request
#[derive(Debug, Deserialize, Validate)]
pub struct CreateEventRequest {
    #[validate(length(min = 3, max = 200))]
    pub title: String,
    #[validate(length(max = 5000))]
    pub description: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    #[validate(length(equal = 15))]
    pub location_cell: String, // H3 cell ID
    pub capacity: Option<u32>,
    pub tags: Vec<String>,
}

/// List events (paginated)
/// GET /api/v1/events
pub async fn list_events() -> Result<Json<Vec<EventSummary>>> {
    // TODO: Parse query parameters (page, limit, filters)
    // TODO: Query database
    // TODO: Return paginated results

    Ok(Json(vec![]))
}

/// Get event by ID
/// GET /api/v1/events/:id
pub async fn get_event(Path(id): Path<Uuid>) -> Result<Json<EventDetails>> {
    // TODO: Query database
    // TODO: Check if event exists

    let _ = id;
    Err(ApiError::EventNotFound)
}

/// Create new event
/// POST /api/v1/events
pub async fn create_event(Json(req): Json<CreateEventRequest>) -> Result<Json<EventDetails>> {
    // Validate input
    req.validate()
        .map_err(|e| ApiError::InvalidInput(e.to_string()))?;

    // TODO: Check user is authenticated and level >= 2
    // TODO: Validate times (start < end, not in past)
    // TODO: Create event in database
    // TODO: Return created event

    Err(ApiError::Forbidden)
}
