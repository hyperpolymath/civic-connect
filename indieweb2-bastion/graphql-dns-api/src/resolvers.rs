// SPDX-License-Identifier: PMPL-1.0-or-later
//! GraphQL resolvers for DNS queries and mutations

use async_graphql::{Context, Object, Result, ID};
use chrono::Utc;

use crate::{
    blockchain::BlockchainClient,
    db::Database,
    dnssec::DNSSECManager,
    models::{
        BlockchainProvenance, DNSRecord, DNSRecordInput, DNSRecordType, DNSSECZone, DNSStatistics,
        ReverseDNSResult,
    },
};

/// GraphQL Query root
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get single DNS record by ID
    async fn dns_record(&self, ctx: &Context<'_>, id: ID) -> Result<Option<DNSRecord>> {
        let db = ctx.data::<Database>()?;
        match db.get_record(&id.to_string()).await {
            Ok(record) => Ok(Some(record)),
            Err(_) => Ok(None),
        }
    }

    /// Query DNS records by name and/or type
    async fn dns_records(
        &self,
        ctx: &Context<'_>,
        name: Option<String>,
        #[graphql(name = "type")] record_type: Option<DNSRecordType>,
        #[graphql(default = 100)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> Result<Vec<DNSRecord>> {
        let db = ctx.data::<Database>()?;
        let records = db.query_records(name, record_type, limit, offset).await?;
        Ok(records)
    }

    /// Reverse DNS lookup (IP to hostname)
    async fn reverse_dns(&self, ctx: &Context<'_>, ip: String) -> Result<ReverseDNSResult> {
        let db = ctx.data::<Database>()?;

        // Convert IP to reverse DNS name (e.g., 192.0.2.1 -> 1.2.0.192.in-addr.arpa)
        let reverse_name = ip_to_reverse_name(&ip)?;

        // Query PTR records
        let ptr_records = db
            .query_records(Some(reverse_name), Some(DNSRecordType::PTR), 100, 0)
            .await?;

        // Extract hostnames from PTR records
        let hostnames: Vec<String> = ptr_records.iter().map(|r| r.value.clone()).collect();

        Ok(ReverseDNSResult {
            ip,
            hostnames,
            ptr_records,
        })
    }

    /// Get DNSSEC configuration for a zone
    async fn dnssec_zone(&self, ctx: &Context<'_>, zone: String) -> Result<Option<DNSSECZone>> {
        let db = ctx.data::<Database>()?;
        match db.get_dnssec_zone(&zone).await {
            Ok(zone) => Ok(Some(zone)),
            Err(_) => Ok(None),
        }
    }

    /// Get blockchain provenance for a record
    async fn blockchain_provenance(
        &self,
        ctx: &Context<'_>,
        record_id: ID,
    ) -> Result<Option<BlockchainProvenance>> {
        let db = ctx.data::<Database>()?;
        match db.get_provenance(&record_id.to_string()).await {
            Ok(provenance) => Ok(Some(provenance)),
            Err(_) => Ok(None),
        }
    }

    /// Get DNS statistics
    async fn statistics(&self, ctx: &Context<'_>) -> Result<DNSStatistics> {
        let db = ctx.data::<Database>()?;
        let (total_records, records_by_type, dnssec_zones, blockchain_anchored) =
            db.get_statistics().await?;

        Ok(DNSStatistics {
            total_records,
            records_by_type,
            dnssec_zones,
            blockchain_anchored,
        })
    }

    /// Health check
    async fn health(&self) -> Result<String> {
        Ok("OK".to_string())
    }

    /// Get current policy configuration
    async fn policy(&self, ctx: &Context<'_>) -> Result<crate::policy::Policy> {
        let enforcer = ctx.data::<std::sync::Arc<tokio::sync::RwLock<crate::policy::PolicyEnforcer>>>()?;
        let enforcer = enforcer.read().await;
        Ok(enforcer.policy().clone())
    }

    /// Get mutation proposal by ID
    async fn proposal(&self, ctx: &Context<'_>, id: ID) -> Result<Option<crate::policy::MutationProposal>> {
        let enforcer = ctx.data::<std::sync::Arc<tokio::sync::RwLock<crate::policy::PolicyEnforcer>>>()?;
        let enforcer = enforcer.read().await;
        Ok(enforcer.get_proposal(&id.to_string()).cloned())
    }

    /// Get all mutation proposals
    async fn proposals(
        &self,
        ctx: &Context<'_>,
        status: Option<crate::policy::ProposalStatus>,
    ) -> Result<Vec<crate::policy::MutationProposal>> {
        let enforcer = ctx.data::<std::sync::Arc<tokio::sync::RwLock<crate::policy::PolicyEnforcer>>>()?;
        let enforcer = enforcer.read().await;
        let all_proposals = enforcer.get_proposals();

        Ok(if let Some(status_filter) = status {
            all_proposals.into_iter().filter(|p| p.status == status_filter).collect()
        } else {
            all_proposals
        })
    }

    /// Check if identity has privilege
    async fn has_privilege(&self, ctx: &Context<'_>, identity: String, privilege: String) -> Result<bool> {
        let enforcer = ctx.data::<std::sync::Arc<tokio::sync::RwLock<crate::policy::PolicyEnforcer>>>()?;
        let enforcer = enforcer.read().await;
        Ok(enforcer.has_privilege(&identity, &privilege))
    }
}

/// GraphQL Mutation root
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Create a new DNS record
    async fn create_dns_record(
        &self,
        ctx: &Context<'_>,
        input: DNSRecordInput,
    ) -> Result<DNSRecord> {
        let db = ctx.data::<Database>()?;
        let consent = ctx.data::<std::sync::Arc<crate::consent::ConsentClient>>()?;

        // Get identity from context (mTLS cert or auth token) — reject unauthenticated
        let identity = ctx.data_opt::<String>()
            .cloned()
            .ok_or_else(|| async_graphql::Error::new(
                "Authentication required: no identity in request context"
            ))?;

        // Check DNS operations consent
        crate::consent::require_dns_consent(consent, &identity).await?;

        // Validate record
        validate_dns_record(&input)?;

        // Create record
        let mut record = DNSRecord::new(
            input.name,
            input.record_type,
            input.ttl,
            input.value,
        );

        if let Some(dnssec) = input.dnssec {
            record.dnssec = dnssec;
        }

        let created = db.create_record(record).await?;
        Ok(created)
    }

    /// Update an existing DNS record
    async fn update_dns_record(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: DNSRecordInput,
    ) -> Result<DNSRecord> {
        let db = ctx.data::<Database>()?;

        // Get existing record
        let mut record = db.get_record(&id.to_string()).await?;

        // Validate new data
        validate_dns_record(&input)?;

        // Update fields
        record.name = input.name;
        record.record_type = input.record_type;
        record.ttl = input.ttl;
        record.value = input.value;
        record.updated_at = Utc::now();

        if let Some(dnssec) = input.dnssec {
            record.dnssec = dnssec;
        }

        let updated = db.update_record(&id.to_string(), record).await?;
        Ok(updated)
    }

    /// Delete a DNS record
    async fn delete_dns_record(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let db = ctx.data::<Database>()?;
        db.delete_record(&id.to_string()).await?;
        Ok(true)
    }

    /// Enable DNSSEC for a zone
    async fn enable_dnssec(&self, ctx: &Context<'_>, zone: String) -> Result<DNSSECZone> {
        let db = ctx.data::<Database>()?;
        let dnssec_manager = DNSSECManager::new();

        // Generate DNSSEC keys
        let (ksk, zsk, ds_record) = dnssec_manager.generate_keys(&zone)?;

        let dnssec_zone = DNSSECZone {
            zone,
            enabled: true,
            ksk: Some(ksk),
            zsk: Some(zsk),
            ds_record: Some(ds_record),
            last_rotation: Some(Utc::now()),
        };

        let created = db.upsert_dnssec_zone(dnssec_zone).await?;
        Ok(created)
    }

    /// Rotate DNSSEC keys for a zone
    async fn rotate_dnssec_keys(&self, ctx: &Context<'_>, zone: String) -> Result<DNSSECZone> {
        let db = ctx.data::<Database>()?;
        let dnssec_manager = DNSSECManager::new();

        // Get existing zone
        let mut dnssec_zone = db.get_dnssec_zone(&zone).await?;

        // Rotate keys
        let (ksk, zsk, ds_record) = dnssec_manager.generate_keys(&zone)?;

        dnssec_zone.ksk = Some(ksk);
        dnssec_zone.zsk = Some(zsk);
        dnssec_zone.ds_record = Some(ds_record);
        dnssec_zone.last_rotation = Some(Utc::now());

        let updated = db.upsert_dnssec_zone(dnssec_zone).await?;
        Ok(updated)
    }

    /// Anchor record hash to blockchain
    async fn anchor_to_blockchain(
        &self,
        ctx: &Context<'_>,
        record_id: ID,
        network: String,
    ) -> Result<BlockchainProvenance> {
        let db = ctx.data::<Database>()?;

        // Get the record
        let record = db.get_record(&record_id.to_string()).await?;

        // Calculate content hash
        let content_hash = record.content_hash();

        // Initialize blockchain client
        let blockchain_client = BlockchainClient::new(&network)?;

        // Anchor hash to blockchain
        let (tx_hash, block_number) = blockchain_client.anchor_hash(&content_hash).await?;

        // Store provenance
        let provenance = BlockchainProvenance {
            record_id: record_id.clone(),
            content_hash,
            network,
            tx_hash,
            block_number,
            timestamp: Utc::now(),
        };

        let stored = db.store_provenance(provenance).await?;
        Ok(stored)
    }

    /// Propose a mutation (requires approval and timelock)
    async fn propose_mutation(
        &self,
        ctx: &Context<'_>,
        mutation_name: String,
        payload: serde_json::Value,
    ) -> Result<crate::policy::MutationProposal> {
        let enforcer = ctx.data::<std::sync::Arc<tokio::sync::RwLock<crate::policy::PolicyEnforcer>>>()?;
        let mut enforcer = enforcer.write().await;

        // Get identity from context (mTLS cert or auth token) — reject unauthenticated
        let identity = ctx.data_opt::<String>()
            .cloned()
            .ok_or_else(|| async_graphql::Error::new(
                "Authentication required: no identity in request context"
            ))?;

        let proposal = enforcer.propose_mutation(&mutation_name, &identity, payload)?;
        Ok(proposal)
    }

    /// Approve a mutation proposal
    async fn approve_mutation(
        &self,
        ctx: &Context<'_>,
        proposal_id: ID,
    ) -> Result<crate::policy::MutationProposal> {
        let enforcer = ctx.data::<std::sync::Arc<tokio::sync::RwLock<crate::policy::PolicyEnforcer>>>()?;
        let mut enforcer = enforcer.write().await;

        // Get identity from context (mTLS cert or auth token) — reject unauthenticated
        let identity = ctx.data_opt::<String>()
            .cloned()
            .ok_or_else(|| async_graphql::Error::new(
                "Authentication required: no identity in request context"
            ))?;

        let proposal = enforcer.approve_proposal(&proposal_id.to_string(), &identity)?;
        Ok(proposal)
    }

    /// Execute an approved mutation
    async fn execute_mutation(
        &self,
        ctx: &Context<'_>,
        proposal_id: ID,
    ) -> Result<crate::policy::MutationProposal> {
        let enforcer = ctx.data::<std::sync::Arc<tokio::sync::RwLock<crate::policy::PolicyEnforcer>>>()?;
        let mut enforcer = enforcer.write().await;

        let proposal = enforcer.execute_proposal(&proposal_id.to_string())?;

        // TODO: Actually execute the mutation based on proposal.payload
        // For now, just mark as executed

        Ok(proposal)
    }
}

/// Validate DNS record input
fn validate_dns_record(input: &DNSRecordInput) -> Result<()> {
    // Validate name (basic check)
    if input.name.is_empty() {
        return Err("DNS name cannot be empty".into());
    }

    // Validate TTL
    if input.ttl < 0 {
        return Err("TTL must be positive".into());
    }

    // Validate value based on type
    match input.record_type {
        DNSRecordType::A => {
            // Basic IPv4 validation
            if !input.value.split('.').all(|part| part.parse::<u8>().is_ok()) {
                return Err("Invalid IPv4 address".into());
            }
        }
        DNSRecordType::AAAA => {
            // Basic IPv6 validation
            if !input.value.contains(':') {
                return Err("Invalid IPv6 address".into());
            }
        }
        _ => {
            // Other types - basic non-empty check
            if input.value.is_empty() {
                return Err("Record value cannot be empty".into());
            }
        }
    }

    Ok(())
}

/// Convert IP address to reverse DNS name
fn ip_to_reverse_name(ip: &str) -> Result<String> {
    if ip.contains(':') {
        // IPv6
        // Simplified: actual implementation would expand and reverse nibbles
        Ok(format!("{}.ip6.arpa", ip.replace(':', ".")))
    } else {
        // IPv4
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() != 4 {
            return Err("Invalid IPv4 address".into());
        }
        Ok(format!("{}.{}.{}.{}.in-addr.arpa", parts[3], parts[2], parts[1], parts[0]))
    }
}
