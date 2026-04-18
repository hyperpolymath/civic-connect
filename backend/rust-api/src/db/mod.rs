// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! Database operations using sqlx
//!
//! PostgreSQL with PostGIS for spatial queries

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::error::Result;

/// Create database connection pool
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    Ok(pool)
}

/// Database models
pub mod models {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use sqlx::FromRow;
    use uuid::Uuid;

    /// User database model
    #[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
    pub struct User {
        pub id: Uuid,
        pub email_hash: String,
        pub username: String,
        pub password_hash: String,
        pub current_level: i16,
        pub experience_points: i32,
        pub location_hash: Option<String>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
        pub last_active: DateTime<Utc>,
        pub is_verified: bool,
    }

    /// Event database model
    #[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
    pub struct Event {
        pub id: Uuid,
        pub organizer_id: Uuid,
        pub title: String,
        pub description: String,
        pub location_hash: String,
        // location_geom would be handled by PostGIS
        pub start_time: DateTime<Utc>,
        pub end_time: DateTime<Utc>,
        pub capacity: Option<i32>,
        pub tags: Vec<String>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    /// Verification audit log entry
    #[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
    pub struct Verification {
        pub id: Uuid,
        pub event_id: Uuid,
        pub user_id: Uuid,
        pub organizer_id: Uuid,
        pub signature: Vec<u8>,
        pub verified_at: DateTime<Utc>,
        pub experience_awarded: i32,
        pub location_hash: String,
    }

    /// Message (encrypted content)
    #[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
    pub struct Message {
        pub id: Uuid,
        pub sender_id: Uuid,
        pub recipient_id: Uuid,
        pub encrypted_content: Vec<u8>,
        pub sent_at: DateTime<Utc>,
        pub delivered_at: Option<DateTime<Utc>>,
        pub read_at: Option<DateTime<Utc>>,
    }

    /// Mentorship relationship
    #[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
    pub struct Mentorship {
        pub id: Uuid,
        pub mentor_id: Uuid,
        pub mentee_id: Uuid,
        pub status: String,
        pub started_at: DateTime<Utc>,
        pub ended_at: Option<DateTime<Utc>>,
    }

    /// Level progression audit entry
    #[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
    pub struct LevelProgression {
        pub id: Uuid,
        pub user_id: Uuid,
        pub from_level: i16,
        pub to_level: i16,
        pub reason: String,
        pub progressed_at: DateTime<Utc>,
        pub metadata: serde_json::Value,
    }
}
