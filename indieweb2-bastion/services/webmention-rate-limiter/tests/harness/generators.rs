// SPDX-FileCopyrightText: 2025 Hyperpolymath
// SPDX-License-Identifier: PMPL-1.0-or-later

//! Test data generators for attack simulation.

use std::net::{IpAddr, Ipv4Addr};

/// Generate a pool of IP addresses for testing.
pub fn generate_ips(count: usize) -> Vec<IpAddr> {
    (0..count)
        .map(|i| {
            // Use 10.x.x.x private range
            let a = ((i >> 16) & 0xFF) as u8;
            let b = ((i >> 8) & 0xFF) as u8;
            let c = (i & 0xFF) as u8;
            IpAddr::V4(Ipv4Addr::new(10, a, b, c))
        })
        .collect()
}

/// Generate a pool of source URLs for testing.
pub fn generate_sources(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("https://source-{}.example.com/post/{}", i / 10, i % 10))
        .collect()
}

/// Generate a pool of target URLs for testing.
pub fn generate_targets(count: usize, domain: &str) -> Vec<String> {
    (0..count)
        .map(|i| format!("https://{}/article/{}", domain, i))
        .collect()
}

/// Generate self-ping pairs (source and target share domain).
pub fn generate_self_ping_pairs(count: usize) -> Vec<(String, String)> {
    (0..count)
        .map(|i| {
            let domain = format!("site-{}.example.com", i);
            (
                format!("https://blog.{}/post/{}", domain, i),
                format!("https://www.{}/article/{}", domain, i),
            )
        })
        .collect()
}

/// Generate various Content-Type values for bypass testing.
pub fn generate_content_types() -> Vec<Option<&'static str>> {
    vec![
        // Valid
        Some("application/x-www-form-urlencoded"),
        Some("application/x-www-form-urlencoded; charset=utf-8"),
        // Invalid - should be rejected
        Some("application/json"),
        Some("text/plain"),
        Some("multipart/form-data"),
        Some("text/html"),
        Some("application/xml"),
        Some("APPLICATION/X-WWW-FORM-URLENCODED"), // Case variation
        Some("application/x-www-form-urlencoded; boundary=---"),
        None, // Missing
        Some(""), // Empty
        Some("   "), // Whitespace
    ]
}

/// Classify a Content-Type as valid or invalid.
pub fn is_valid_content_type(ct: Option<&str>) -> bool {
    match ct {
        Some(s) => {
            let normalized = s.split(';').next().unwrap_or("").trim().to_lowercase();
            normalized == "application/x-www-form-urlencoded"
        }
        None => false,
    }
}

/// Generate malformed URL variations for testing.
/// These URLs should be rejected by the validator for various reasons:
/// - Empty/whitespace: missing URL
/// - not-a-url: parse failure
/// - ftp/file/javascript/data: invalid scheme (only http/https allowed)
/// - https://: empty host parse failure
/// - ://missing-scheme.com/: parse failure
pub fn generate_malformed_urls() -> Vec<&'static str> {
    vec![
        "",
        "   ",
        "not-a-url",
        "ftp://wrong-scheme.com/",
        "://missing-scheme.com/",
        "https://",
        "javascript:alert(1)",
        "data:text/html,<script>",
        "file:///etc/passwd",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_ips() {
        let ips = generate_ips(256);
        assert_eq!(ips.len(), 256);
        // All should be unique
        let unique: std::collections::HashSet<_> = ips.iter().collect();
        assert_eq!(unique.len(), 256);
    }

    #[test]
    fn test_generate_sources() {
        let sources = generate_sources(100);
        assert_eq!(sources.len(), 100);
        assert!(sources[0].starts_with("https://"));
    }

    #[test]
    fn test_content_type_validation() {
        assert!(is_valid_content_type(Some("application/x-www-form-urlencoded")));
        assert!(is_valid_content_type(Some(
            "application/x-www-form-urlencoded; charset=utf-8"
        )));
        assert!(!is_valid_content_type(Some("application/json")));
        assert!(!is_valid_content_type(None));
    }
}
