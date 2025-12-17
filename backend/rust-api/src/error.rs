// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! Error types for the CivicConnect API

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Result type alias using ApiError
pub type Result<T> = std::result::Result<T, ApiError>;

/// API error types
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("Event not found")]
    EventNotFound,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Already verified")]
    AlreadyVerified,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Outside time window")]
    OutsideTimeWindow,

    #[error("Outside location")]
    OutsideLocation,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

/// Error response body
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: &'static str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Self::InvalidCredentials => (StatusCode::UNAUTHORIZED, "INVALID_CREDENTIALS"),
            Self::UserNotFound => (StatusCode::NOT_FOUND, "USER_NOT_FOUND"),
            Self::EventNotFound => (StatusCode::NOT_FOUND, "EVENT_NOT_FOUND"),
            Self::InvalidInput(_) => (StatusCode::BAD_REQUEST, "INVALID_INPUT"),
            Self::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED"),
            Self::AlreadyVerified => (StatusCode::CONFLICT, "ALREADY_VERIFIED"),
            Self::InvalidSignature => (StatusCode::BAD_REQUEST, "INVALID_SIGNATURE"),
            Self::OutsideTimeWindow => (StatusCode::BAD_REQUEST, "OUTSIDE_TIME_WINDOW"),
            Self::OutsideLocation => (StatusCode::BAD_REQUEST, "OUTSIDE_LOCATION"),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            Self::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
            Self::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
        };

        let body = Json(ErrorResponse {
            error: self.to_string(),
            code,
        });

        (status, body).into_response()
    }
}
