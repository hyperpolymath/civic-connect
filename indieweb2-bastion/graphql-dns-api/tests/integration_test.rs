// SPDX-License-Identifier: PMPL-1.0-or-later
//! Integration tests for GraphQL DNS API

use graphql_dns_api::{
    db::Database,
    models::{DNSRecord, DNSRecordType},
};

#[tokio::test]
async fn test_database_create_record() {
    let db = Database::connect("memory").await.unwrap();

    let record = DNSRecord::new(
        "example.com".to_string(),
        DNSRecordType::A,
        3600,
        "192.0.2.1".to_string(),
    );

    let created = db.create_record(record.clone()).await.unwrap();

    assert_eq!(created.name, "example.com");
    assert_eq!(created.record_type, DNSRecordType::A);
    assert_eq!(created.ttl, 3600);
    assert_eq!(created.value, "192.0.2.1");
}

#[tokio::test]
async fn test_database_query_records() {
    let db = Database::connect("memory").await.unwrap();

    // Create test records
    let record1 = DNSRecord::new(
        "example.com".to_string(),
        DNSRecordType::A,
        3600,
        "192.0.2.1".to_string(),
    );
    let record2 = DNSRecord::new(
        "example.com".to_string(),
        DNSRecordType::AAAA,
        3600,
        "2001:db8::1".to_string(),
    );

    db.create_record(record1).await.unwrap();
    db.create_record(record2).await.unwrap();

    // Query all records for example.com
    let records = db
        .query_records(Some("example.com".to_string()), None, 100, 0)
        .await
        .unwrap();

    assert_eq!(records.len(), 2);
}

#[tokio::test]
async fn test_database_update_record() {
    let db = Database::connect("memory").await.unwrap();

    let mut record = DNSRecord::new(
        "example.com".to_string(),
        DNSRecordType::A,
        3600,
        "192.0.2.1".to_string(),
    );

    let created = db.create_record(record.clone()).await.unwrap();
    let id = created.id.to_string();

    // Update the record
    record.value = "192.0.2.2".to_string();
    let updated = db.update_record(&id, record).await.unwrap();

    assert_eq!(updated.value, "192.0.2.2");
}

#[tokio::test]
async fn test_database_delete_record() {
    let db = Database::connect("memory").await.unwrap();

    let record = DNSRecord::new(
        "example.com".to_string(),
        DNSRecordType::A,
        3600,
        "192.0.2.1".to_string(),
    );

    let created = db.create_record(record).await.unwrap();
    let id = created.id.to_string();

    // Delete the record
    let deleted = db.delete_record(&id).await.unwrap();
    assert!(deleted);

    // Verify it's gone
    let result = db.get_record(&id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_content_hash_consistency() {
    let record1 = DNSRecord::new(
        "example.com".to_string(),
        DNSRecordType::A,
        3600,
        "192.0.2.1".to_string(),
    );

    let record2 = DNSRecord::new(
        "example.com".to_string(),
        DNSRecordType::A,
        3600,
        "192.0.2.1".to_string(),
    );

    // Same record content should produce same hash
    assert_eq!(record1.content_hash(), record2.content_hash());

    let record3 = DNSRecord::new(
        "example.com".to_string(),
        DNSRecordType::A,
        3600,
        "192.0.2.2".to_string(), // Different IP
    );

    // Different content should produce different hash
    assert_ne!(record1.content_hash(), record3.content_hash());
}
