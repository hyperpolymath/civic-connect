// SPDX-License-Identifier: PMPL-1.0-or-later
// Consent checking for GraphQL DNS API
//
// Integrates with IndieWeb2 Bastion consent API to enforce
// user consent preferences before DNS operations.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConsentRecord {
    pub identity: String,
    pub telemetry: String,
    pub indexing: String,
    pub webmentions: String,
    pub dns_operations: String,
    pub timestamp: String,
    pub source: String,
}

/// Consent API client
pub struct ConsentClient {
    base_url: String,
    client: reqwest::Client,
}

impl ConsentClient {
    /// Create new consent client
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Check if identity has consented to DNS operations
    pub async fn check_dns_operations_consent(&self, identity: &str) -> Result<bool> {
        let url = format!("{}/consent/{}/check", self.base_url, urlencoding::encode(identity));

        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "operation": "dnsOperations"
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["allowed"].as_bool().unwrap_or(false))
        } else if response.status() == 404 {
            // No consent record - use default (off for DNS operations)
            Ok(false)
        } else {
            Err(anyhow!("Consent API error: {}", response.status()))
        }
    }

    /// Get full consent record for identity
    pub async fn get_consent(&self, identity: &str) -> Result<Option<ConsentRecord>> {
        let url = format!("{}/consent/{}", self.base_url, urlencoding::encode(identity));

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let record: ConsentRecord = response.json().await?;
            Ok(Some(record))
        } else if response.status() == 404 {
            Ok(None)
        } else {
            Err(anyhow!("Consent API error: {}", response.status()))
        }
    }

    /// Health check for consent API
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        let response = self.client.get(&url).send().await?;
        Ok(response.status().is_success())
    }
}

/// Check consent before DNS mutation
pub async fn require_dns_consent(client: &ConsentClient, identity: &str) -> Result<()> {
    let allowed = client.check_dns_operations_consent(identity).await?;

    if !allowed {
        return Err(anyhow!(
            "Identity {} has not consented to DNS operations. \
             Please enable DNS operations in your consent preferences.",
            identity
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consent_client_creation() {
        let client = ConsentClient::new("http://localhost:8082".to_string());
        assert_eq!(client.base_url, "http://localhost:8082");
    }

    #[tokio::test]
    #[ignore] // Requires consent API running
    async fn test_health_check() {
        let client = ConsentClient::new("http://localhost:8082".to_string());
        let result = client.health_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires consent API running and test data
    async fn test_check_dns_operations() {
        let client = ConsentClient::new("http://localhost:8082".to_string());
        let result = client.check_dns_operations_consent("https://example.com/").await;
        assert!(result.is_ok());
    }
}
