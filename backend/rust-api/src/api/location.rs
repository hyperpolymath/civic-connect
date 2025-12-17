// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! Location endpoints
//!
//! Privacy-preserving location system using H3 hexagonal grid.
//! Server never receives or stores exact coordinates.
//!
//! Flow:
//! 1. Client gets GPS coordinates locally
//! 2. Client computes H3 cell ID (client-side)
//! 3. Client sends H3 cell to server
//! 4. Server queries for events in cell + neighbors

use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::events::EventSummary;

/// Nearby events query parameters
#[derive(Debug, Deserialize)]
pub struct NearbyQuery {
    /// H3 cell ID at resolution 7 (~5km hexagon)
    pub cell: String,
    /// Number of rings of neighbors to include (0-2)
    #[serde(default)]
    pub rings: u8,
    /// Only show upcoming events
    #[serde(default = "default_true")]
    pub upcoming_only: bool,
}

fn default_true() -> bool {
    true
}

/// Response with nearby events and cell info
#[derive(Debug, Serialize)]
pub struct NearbyResponse {
    pub events: Vec<EventSummary>,
    pub cells_searched: Vec<String>,
    pub total_count: u32,
}

/// Find events near a location (privacy-preserving)
/// GET /api/v1/location/nearby
///
/// Note: Client computes H3 cell from GPS locally.
/// Server never receives exact coordinates.
pub async fn nearby_events(Query(query): Query<NearbyQuery>) -> Result<Json<NearbyResponse>> {
    // Validate cell ID format (15 hex characters for resolution 7)
    if query.cell.len() != 15 || !query.cell.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(Json(NearbyResponse {
            events: vec![],
            cells_searched: vec![],
            total_count: 0,
        }));
    }

    // Limit rings to prevent large queries
    let rings = query.rings.min(2);

    // TODO: Use h3o to compute neighboring cells
    // TODO: Query database for events in those cells
    // TODO: Filter by upcoming if requested
    // TODO: Sort by start time

    let cells_searched = vec![query.cell.clone()];
    // Would add neighbor cells here based on rings

    Ok(Json(NearbyResponse {
        events: vec![],
        cells_searched,
        total_count: 0,
    }))
}
