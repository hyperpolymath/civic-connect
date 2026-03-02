// SPDX-FileCopyrightText: 2025 Hyperpolymath
// SPDX-License-Identifier: PMPL-1.0-or-later

//! Webmention request validator.
//!
//! Implements ingress-level validation for Webmention requests:
//! - Content-Type validation
//! - Source/target parameter presence
//! - Self-ping blocking (same domain source/target)
//! - URL format validation

use crate::config::ValidationConfig;
use thiserror::Error;
use tracing::debug;
use url::Url;

/// Validation error types.
#[derive(Debug, Error, Clone)]
pub enum ValidationError {
    #[error("Invalid Content-Type: expected one of {expected:?}, got {actual:?}")]
    InvalidContentType {
        expected: Vec<String>,
        actual: Option<String>,
    },

    #[error("Missing required parameter: {0}")]
    MissingParameter(&'static str),

    #[error("Invalid URL format for {param}: {url}")]
    InvalidUrl { param: &'static str, url: String },

    #[error("Self-ping blocked: source and target share domain {domain}")]
    SelfPingBlocked { domain: String },
}

/// Result of validation.
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Request is valid
    Valid,
    /// Request is invalid
    Invalid(ValidationError),
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }

    pub fn error(&self) -> Option<&ValidationError> {
        match self {
            ValidationResult::Valid => None,
            ValidationResult::Invalid(e) => Some(e),
        }
    }
}

/// Webmention request validator.
pub struct WebmentionValidator {
    config: ValidationConfig,
}

impl WebmentionValidator {
    /// Create a new validator with the given configuration.
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Validate the Content-Type header.
    pub fn validate_content_type(&self, content_type: Option<&str>) -> ValidationResult {
        let ct = content_type.map(|s| {
            // Extract just the media type, ignoring charset etc.
            s.split(';').next().unwrap_or(s).trim().to_lowercase()
        });

        let expected_lower: Vec<String> = self
            .config
            .require_content_type
            .iter()
            .map(|s| s.to_lowercase())
            .collect();

        match &ct {
            Some(actual) if expected_lower.contains(actual) => {
                debug!(content_type = %actual, "Content-Type valid");
                ValidationResult::Valid
            }
            _ => {
                debug!(content_type = ?ct, expected = ?expected_lower, "Content-Type invalid");
                ValidationResult::Invalid(ValidationError::InvalidContentType {
                    expected: self.config.require_content_type.clone(),
                    actual: ct,
                })
            }
        }
    }

    /// Validate source and target parameters.
    pub fn validate_source_target(
        &self,
        source: Option<&str>,
        target: Option<&str>,
    ) -> ValidationResult {
        if !self.config.require_source_target {
            return ValidationResult::Valid;
        }

        // Check presence
        let source = match source {
            Some(s) if !s.trim().is_empty() => s,
            _ => {
                debug!("Missing source parameter");
                return ValidationResult::Invalid(ValidationError::MissingParameter("source"));
            }
        };

        let target = match target {
            Some(t) if !t.trim().is_empty() => t,
            _ => {
                debug!("Missing target parameter");
                return ValidationResult::Invalid(ValidationError::MissingParameter("target"));
            }
        };

        // Validate URL formats
        let source_url = match Url::parse(source) {
            Ok(u) => u,
            Err(_) => {
                debug!(source = %source, "Invalid source URL format");
                return ValidationResult::Invalid(ValidationError::InvalidUrl {
                    param: "source",
                    url: source.to_string(),
                });
            }
        };

        let target_url = match Url::parse(target) {
            Ok(u) => u,
            Err(_) => {
                debug!(target = %target, "Invalid target URL format");
                return ValidationResult::Invalid(ValidationError::InvalidUrl {
                    param: "target",
                    url: target.to_string(),
                });
            }
        };

        // Validate URL schemes (only http/https allowed) and require valid host
        if !matches!(source_url.scheme(), "http" | "https") || source_url.host_str().is_none() {
            debug!(source = %source, "Invalid source URL (bad scheme or no host)");
            return ValidationResult::Invalid(ValidationError::InvalidUrl {
                param: "source",
                url: source.to_string(),
            });
        }

        if !matches!(target_url.scheme(), "http" | "https") || target_url.host_str().is_none() {
            debug!(target = %target, "Invalid target URL (bad scheme or no host)");
            return ValidationResult::Invalid(ValidationError::InvalidUrl {
                param: "target",
                url: target.to_string(),
            });
        }

        // Check for self-ping
        if self.config.block_self_ping {
            if let (Some(source_host), Some(target_host)) =
                (source_url.host_str(), target_url.host_str())
            {
                let source_domain = extract_registrable_domain(source_host);
                let target_domain = extract_registrable_domain(target_host);

                if source_domain == target_domain {
                    debug!(
                        source = %source_host,
                        target = %target_host,
                        domain = %source_domain,
                        "Self-ping detected"
                    );
                    return ValidationResult::Invalid(ValidationError::SelfPingBlocked {
                        domain: source_domain,
                    });
                }
            }
        }

        debug!(source = %source, target = %target, "Source/target valid");
        ValidationResult::Valid
    }

    /// Validate a complete Webmention request.
    pub fn validate(
        &self,
        content_type: Option<&str>,
        source: Option<&str>,
        target: Option<&str>,
    ) -> ValidationResult {
        // Validate Content-Type
        let ct_result = self.validate_content_type(content_type);
        if !ct_result.is_valid() {
            return ct_result;
        }

        // Validate source/target
        self.validate_source_target(source, target)
    }
}

/// Extract the registrable domain from a hostname.
/// This is a simplified implementation; a production version would use
/// the Public Suffix List.
fn extract_registrable_domain(host: &str) -> String {
    let host = host.to_lowercase();

    // Handle IP addresses
    if host.parse::<std::net::IpAddr>().is_ok() {
        return host;
    }

    // Simple TLD extraction (handles most common cases)
    let parts: Vec<&str> = host.split('.').collect();
    match parts.len() {
        0 | 1 => host,
        2 => host,
        _ => {
            // Check for common two-part TLDs
            let last_two = format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);
            if is_two_part_tld(&last_two) && parts.len() > 2 {
                // e.g., "example.co.uk" -> "example.co.uk"
                format!(
                    "{}.{}",
                    parts[parts.len() - 3],
                    last_two
                )
            } else {
                // e.g., "sub.example.com" -> "example.com"
                last_two
            }
        }
    }
}

/// Check if a suffix is a known two-part TLD.
fn is_two_part_tld(suffix: &str) -> bool {
    const TWO_PART_TLDS: &[&str] = &[
        "co.uk", "org.uk", "me.uk", "co.nz", "co.jp", "co.kr",
        "com.au", "net.au", "org.au", "com.br", "co.za",
    ];
    TWO_PART_TLDS.contains(&suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_validator() -> WebmentionValidator {
        WebmentionValidator::new(ValidationConfig::default())
    }

    #[test]
    fn test_valid_content_type() {
        let validator = default_validator();

        assert!(validator
            .validate_content_type(Some("application/x-www-form-urlencoded"))
            .is_valid());

        assert!(validator
            .validate_content_type(Some("application/x-www-form-urlencoded; charset=utf-8"))
            .is_valid());
    }

    #[test]
    fn test_invalid_content_type() {
        let validator = default_validator();

        assert!(!validator.validate_content_type(Some("application/json")).is_valid());
        assert!(!validator.validate_content_type(None).is_valid());
    }

    #[test]
    fn test_valid_source_target() {
        let validator = default_validator();

        assert!(validator
            .validate_source_target(
                Some("https://source.example.com/post/1"),
                Some("https://target.example.org/post/2")
            )
            .is_valid());
    }

    #[test]
    fn test_missing_source() {
        let validator = default_validator();

        let result = validator.validate_source_target(
            None,
            Some("https://target.example.org/post/2"),
        );
        assert!(!result.is_valid());
        assert!(matches!(
            result.error(),
            Some(ValidationError::MissingParameter("source"))
        ));
    }

    #[test]
    fn test_self_ping_blocked() {
        let validator = default_validator();

        let result = validator.validate_source_target(
            Some("https://blog.example.com/post/1"),
            Some("https://www.example.com/post/2"),
        );
        assert!(!result.is_valid());
        assert!(matches!(
            result.error(),
            Some(ValidationError::SelfPingBlocked { .. })
        ));
    }

    #[test]
    fn test_extract_registrable_domain() {
        assert_eq!(extract_registrable_domain("example.com"), "example.com");
        assert_eq!(extract_registrable_domain("www.example.com"), "example.com");
        assert_eq!(extract_registrable_domain("blog.example.com"), "example.com");
        assert_eq!(extract_registrable_domain("example.co.uk"), "example.co.uk");
        assert_eq!(extract_registrable_domain("www.example.co.uk"), "example.co.uk");
    }

    #[test]
    fn test_malformed_urls_rejected() {
        let validator = default_validator();

        // FTP scheme should be rejected
        let result = validator.validate_source_target(
            Some("ftp://example.com/file"),
            Some("https://target.com/"),
        );
        assert!(!result.is_valid(), "FTP URL should be rejected");

        // Empty host should be rejected (https:// fails to parse)
        let result = validator.validate_source_target(
            Some("https://"),
            Some("https://target.com/"),
        );
        assert!(!result.is_valid(), "URL with empty host should be rejected");

        // javascript: scheme should be rejected
        let result = validator.validate_source_target(
            Some("javascript:alert(1)"),
            Some("https://target.com/"),
        );
        assert!(!result.is_valid(), "javascript: URL should be rejected");

        // file: scheme should be rejected
        let result = validator.validate_source_target(
            Some("file:///etc/passwd"),
            Some("https://target.com/"),
        );
        assert!(!result.is_valid(), "file: URL should be rejected");
    }
}
