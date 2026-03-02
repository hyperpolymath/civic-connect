// SPDX-FileCopyrightText: 2025 Hyperpolymath
// SPDX-License-Identifier: PMPL-1.0-or-later

//! Webmention Rate Limiter
//!
//! This crate provides ingress-level rate limiting for Webmention endpoints,
//! implementing the constraints defined in the CURPS policy:
//!
//! - Per-IP rate limiting (60 rpm default)
//! - Per-source URL rate limiting (10 rpm)
//! - Content-Type validation
//! - Source/target parameter validation
//! - Self-ping blocking (same domain source/target)
//! - Burst detection with cooldown

pub mod config;
pub mod handlers;
pub mod limiter;
pub mod validator;

pub use config::Config;
pub use limiter::{RateLimitResult, RateLimiter};
pub use validator::{ValidationResult, WebmentionValidator};
