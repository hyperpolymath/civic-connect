// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! CivicConnect API Library
//!
//! Core types and functionality for the CivicConnect REST API.

pub mod api;
pub mod crypto;
pub mod db;
pub mod error;
pub mod location;

/// Re-export commonly used types
pub use error::{ApiError, Result};
