// SPDX-License-Identifier: PMPL-1.0-or-later
//! Data models for DNS records and blockchain provenance

use async_graphql::{Enum, InputObject, SimpleObject, ID};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DNS record type enumeration - full RR coverage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Enum)]
pub enum DNSRecordType {
    /// IPv4 address
    A,
    /// IPv6 address
    AAAA,
    /// Canonical name (alias)
    CNAME,
    /// Mail exchange
    MX,
    /// Text record
    TXT,
    /// Service locator
    SRV,
    /// Certification Authority Authorization
    CAA,
    /// DANE certificate association
    TLSA,
    /// Name server
    NS,
    /// Start of authority
    SOA,
    /// Pointer (reverse DNS)
    PTR,
}

impl DNSRecordType {
    /// Convert to DNS wire format type code
    pub fn to_type_code(&self) -> u16 {
        match self {
            Self::A => 1,
            Self::NS => 2,
            Self::CNAME => 5,
            Self::SOA => 6,
            Self::PTR => 12,
            Self::MX => 15,
            Self::TXT => 16,
            Self::AAAA => 28,
            Self::SRV => 33,
            Self::CAA => 257,
            Self::TLSA => 52,
        }
    }
}

/// DNS record with blockchain provenance
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct DNSRecord {
    /// Unique record identifier
    pub id: ID,
    /// Fully qualified domain name
    pub name: String,
    /// DNS record type
    #[serde(rename = "type")]
    pub record_type: DNSRecordType,
    /// Time to live (seconds)
    pub ttl: i32,
    /// Record value (format depends on type)
    pub value: String,
    /// DNSSEC enabled for this record
    pub dnssec: bool,
    /// DNSSEC signature (if enabled)
    pub rrsig: Option<String>,
    /// Blockchain transaction hash (provenance anchor)
    pub blockchain_tx_hash: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl DNSRecord {
    /// Create a new DNS record
    pub fn new(name: String, record_type: DNSRecordType, ttl: i32, value: String) -> Self {
        let now = Utc::now();
        Self {
            id: ID(Uuid::new_v4().to_string()),
            name,
            record_type,
            ttl,
            value,
            dnssec: false,
            rrsig: None,
            blockchain_tx_hash: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Calculate content hash for blockchain anchoring (BLAKE3 per CRYPTO-POLICY.adoc CPR-009)
    pub fn content_hash(&self) -> String {
        let content = format!(
            "{}:{}:{}:{}",
            self.name, self.record_type as u16, self.ttl, self.value
        );
        let hash = blake3::hash(content.as_bytes());
        hex::encode(hash.as_bytes())
    }
}

/// Input for creating/updating DNS records
#[derive(Debug, Clone, InputObject)]
pub struct DNSRecordInput {
    pub name: String,
    #[graphql(name = "type")]
    pub record_type: DNSRecordType,
    pub ttl: i32,
    pub value: String,
    pub dnssec: Option<bool>,
}

/// DNSSEC zone configuration
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct DNSSECZone {
    /// Zone name
    pub zone: String,
    /// DNSSEC enabled
    pub enabled: bool,
    /// Key signing key (KSK) public key
    pub ksk: Option<String>,
    /// Zone signing key (ZSK) public key
    pub zsk: Option<String>,
    /// DS record for parent zone
    pub ds_record: Option<String>,
    /// Last key rotation
    pub last_rotation: Option<DateTime<Utc>>,
}

/// Reverse DNS lookup result
#[derive(Debug, Clone, SimpleObject)]
pub struct ReverseDNSResult {
    /// IP address queried
    pub ip: String,
    /// Resolved hostnames
    pub hostnames: Vec<String>,
    /// PTR records
    pub ptr_records: Vec<DNSRecord>,
}

/// Blockchain provenance information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct BlockchainProvenance {
    /// Record ID
    pub record_id: ID,
    /// Content hash anchored to blockchain
    pub content_hash: String,
    /// Blockchain network (ethereum, polygon)
    pub network: String,
    /// Transaction hash
    pub tx_hash: String,
    /// Block number
    pub block_number: i64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// DNS query statistics
#[derive(Debug, Clone, SimpleObject)]
pub struct DNSStatistics {
    /// Total records
    pub total_records: i32,
    /// Records by type
    pub records_by_type: Vec<RecordTypeCount>,
    /// DNSSEC enabled zones
    pub dnssec_zones: i32,
    /// Blockchain anchored records
    pub blockchain_anchored: i32,
}

/// Record count by type
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct RecordTypeCount {
    #[graphql(name = "type")]
    pub record_type: DNSRecordType,
    pub count: i32,
}
