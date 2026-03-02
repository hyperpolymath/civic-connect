// SPDX-License-Identifier: PMPL-1.0-or-later
//! Error types for GraphQL DNS API

use thiserror::Error;

/// Application error types
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] surrealdb::Error),

    #[error("Blockchain error: {0}")]
    Blockchain(String),

    #[error("DNSSEC error: {0}")]
    DNSSEC(String),

    #[error("DNS record not found: {0}")]
    RecordNotFound(String),

    #[error("Invalid DNS record: {0}")]
    InvalidRecord(String),

    #[error("Invalid IP address: {0}")]
    InvalidIP(String),

    #[error("Zone not found: {0}")]
    ZoneNotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, AppError>;
