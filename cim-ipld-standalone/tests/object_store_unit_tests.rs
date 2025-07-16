//! Unit tests for object store functionality

use cim_ipld::object_store::{ContentDomain, ObjectInfo, PartitionStrategy, PullOptions};
use cim_ipld::{Cid, ContentType, TypedContent};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

// Test content type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: String,
    value: i32,
    description: String,
}

impl TypedContent for TestData {
    const CODEC: u64 = 0x300100;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x300100);
}

// ============================================================================
// Test: Content Domain Detection
// ============================================================================

#[test]
fn test_content_domain_detection() {
    let strategy = PartitionStrategy::default();

    // Test music detection by extension
    let domain = strategy.determine_domain(Some("song.mp3"), None, None, None);
    assert_eq!(domain, ContentDomain::Music);

    // Test video detection by extension
    let domain = strategy.determine_domain(Some("movie.mp4"), None, None, None);
    assert_eq!(domain, ContentDomain::Video);

    // Test document detection
    let domain = strategy.determine_domain(Some("report.pdf"), None, None, None);
    assert_eq!(domain, ContentDomain::Documents);
}

// ============================================================================
// Test: Partition Strategy Domain Mapping
// ============================================================================

#[test]
fn test_partition_strategy_domain_mapping() {
    let strategy = PartitionStrategy::default();

    // Test bucket mapping for different domains
    assert_eq!(
        strategy.get_bucket_for_domain(ContentDomain::Music),
        "cim-media-music"
    );

    assert_eq!(
        strategy.get_bucket_for_domain(ContentDomain::Documents),
        "cim-docs-general"
    );

    assert_eq!(
        strategy.get_bucket_for_domain(ContentDomain::Medical),
        "cim-health-medical"
    );
}

// ============================================================================
// Test: Pull Options
// ============================================================================

#[test]
fn test_pull_options() {
    // Test default options
    let default_opts = PullOptions::default();
    assert!(default_opts.limit.is_none());
    assert!(default_opts.min_size.is_none());
    assert!(default_opts.max_size.is_none());
    assert!(!default_opts.compressed_only);

    // Test custom options
    let custom_opts = PullOptions {
        limit: Some(100),
        min_size: Some(1024),
        max_size: Some(1024 * 1024),
        compressed_only: true,
    };

    assert_eq!(custom_opts.limit.unwrap(), 100);
    assert_eq!(custom_opts.min_size.unwrap(), 1024);
    assert_eq!(custom_opts.max_size.unwrap(), 1024 * 1024);
    assert!(custom_opts.compressed_only);
}

// ============================================================================
// Test: Object Info
// ============================================================================

#[test]
fn test_object_info() {
    let test_cid =
        Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap();

    let info = ObjectInfo {
        cid: test_cid,
        size: 1024,
        created_at: SystemTime::now(),
        compressed: false,
    };

    assert_eq!(info.size, 1024);
    assert!(!info.compressed);
    assert!(!info.cid.to_string().is_empty());
}

// ============================================================================
// Test: Pattern Matching for Content Classification
// ============================================================================

#[test]
fn test_pattern_matching() {
    let strategy = PartitionStrategy::default();

    // Test contract detection by content
    let domain = strategy.determine_domain(
        None,
        None,
        Some("This contract agreement hereby establishes the terms and conditions"),
        None,
    );
    assert_eq!(domain, ContentDomain::Contracts);

    // Test invoice detection by content
    let domain = strategy.determine_domain(
        None,
        None,
        Some("Invoice Number: 12345\nBill To: Customer\nTotal Due: $100.00"),
        None,
    );
    assert_eq!(domain, ContentDomain::Invoices);

    // Test medical detection by content
    let domain = strategy.determine_domain(
        None,
        None,
        Some("Patient diagnosis: hypertension. Prescription: medication"),
        None,
    );
    assert_eq!(domain, ContentDomain::Medical);
}

// ============================================================================
// Test: MIME Type Detection
// ============================================================================

#[test]
fn test_mime_type_detection() {
    let strategy = PartitionStrategy::default();

    // Test audio MIME type
    let domain = strategy.determine_domain(None, Some("audio/mpeg"), None, None);
    assert_eq!(domain, ContentDomain::Music);

    // Test video MIME type
    let domain = strategy.determine_domain(None, Some("video/mp4"), None, None);
    assert_eq!(domain, ContentDomain::Video);

    // Test document MIME type
    let domain = strategy.determine_domain(None, Some("application/pdf"), None, None);
    assert_eq!(domain, ContentDomain::Documents);
}

// ============================================================================
// Test: Custom Domain Mapping
// ============================================================================

#[test]
fn test_custom_domain_mapping() {
    let mut strategy = PartitionStrategy::default();

    // Add custom domain mapping
    strategy.add_domain_mapping(
        ContentDomain::Research,
        "custom-research-bucket".to_string(),
    );

    // Verify custom mapping
    assert_eq!(
        strategy.get_bucket_for_domain(ContentDomain::Research),
        "custom-research-bucket"
    );

    // Add custom extension mapping
    strategy.add_extension_mapping("research".to_string(), ContentDomain::Research);

    // Test custom extension
    let domain = strategy.determine_domain(Some("study.research"), None, None, None);
    assert_eq!(domain, ContentDomain::Research);
}

// ============================================================================
// Test: Content Deduplication
// ============================================================================

#[test]
fn test_content_deduplication() {
    // Same content should produce same CID
    let data1 = TestData {
        id: "dedup-test".to_string(),
        value: 999,
        description: "Deduplication test".to_string(),
    };

    let data2 = data1.clone(); // Exact copy

    let cid1 = data1.calculate_cid().unwrap();
    let cid2 = data2.calculate_cid().unwrap();

    assert_eq!(cid1, cid2); // Same content = same CID

    // Different content should produce different CID
    let data3 = TestData {
        id: "dedup-test".to_string(),
        value: 1000, // Different value
        description: "Deduplication test".to_string(),
    };

    let cid3 = data3.calculate_cid().unwrap();

    assert_ne!(cid1, cid3); // Different content = different CID
}

// ============================================================================
// Test: Social Media Detection
// ============================================================================

#[test]
fn test_social_media_detection() {
    let strategy = PartitionStrategy::default();

    // Test social media content with hashtags and mentions
    let domain = strategy.determine_domain(
        None,
        None,
        Some("Check out this #awesome post! @friend you should see this"),
        None,
    );
    assert_eq!(domain, ContentDomain::SocialMedia);

    // Test meme content
    let domain = strategy.determine_domain(
        None,
        None,
        Some("LOL this meme is so funny and viral!"),
        None,
    );
    assert_eq!(domain, ContentDomain::Memes);
}
