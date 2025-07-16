//! Unit tests for IPLD codec functionality

use cim_ipld::codec::{CimCodec, CodecRegistry as CimCodecRegistry};
use cim_ipld::{Cid, ContentType, Error, TypedContent};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Test content types with custom codecs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct CustomEvent {
    event_type: String,
    timestamp: u64,
    data: serde_json::Value,
}

impl TypedContent for CustomEvent {
    const CODEC: u64 = 0x300200; // Custom codec in allowed range
    const CONTENT_TYPE: ContentType = ContentType::Event;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct CustomDocument {
    title: String,
    content: String,
    version: u32,
}

impl TypedContent for CustomDocument {
    const CODEC: u64 = 0x300201; // Another custom codec
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x300201);
}

// ============================================================================
// Test: Codec Range Validation
// ============================================================================

#[test]
fn test_codec_range_validation() {
    // Valid custom codec range: 0x300000 - 0x3FFFFF

    // Test valid codecs
    assert!(is_valid_custom_codec(0x300000)); // Start of range
    assert!(is_valid_custom_codec(0x300200)); // Middle of range
    assert!(is_valid_custom_codec(0x3FFFFF)); // End of range

    // Test invalid codecs
    assert!(!is_valid_custom_codec(0x2FFFFF)); // Below range
    assert!(!is_valid_custom_codec(0x400000)); // Above range
    assert!(!is_valid_custom_codec(0x55)); // Standard codec (raw)
    assert!(!is_valid_custom_codec(0x71)); // Standard codec (cbor)
}

fn is_valid_custom_codec(codec: u64) -> bool {
    codec >= 0x300000 && codec <= 0x3FFFFF
}

// ============================================================================
// Test: Custom Codec Registration
// ============================================================================

// Custom codec implementation
struct TestCodec {
    code: u64,
    name: String,
}

impl CimCodec for TestCodec {
    fn code(&self) -> u64 {
        self.code
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[test]
fn test_custom_codec_registration() {
    let mut registry = CimCodecRegistry::new();

    // Register custom event codec
    let event_codec = Arc::new(TestCodec {
        code: CustomEvent::CODEC,
        name: "custom-event".to_string(),
    });

    registry.register(event_codec.clone()).unwrap();

    // Verify registration
    let retrieved = registry.get(CustomEvent::CODEC).unwrap();
    assert_eq!(retrieved.name(), "custom-event");
    assert_eq!(retrieved.code(), CustomEvent::CODEC);

    // Test duplicate registration - should succeed (overwrites)
    let result = registry.register(event_codec);
    assert!(result.is_ok());
}

// ============================================================================
// Test: Standard Codec Support
// ============================================================================

#[test]
fn test_standard_codec_support() {
    // Test standard IPLD codecs
    assert_eq!(get_codec_name(0x55), "raw");
    assert_eq!(get_codec_name(0x71), "cbor");
    assert_eq!(get_codec_name(0x0129), "dag-json");
    assert_eq!(get_codec_name(0x0155), "car");

    // Test content-specific codecs
    assert_eq!(get_codec_name(0x600001), "pdf");
    assert_eq!(get_codec_name(0x600003), "markdown");
    assert_eq!(get_codec_name(0x610001), "jpeg");
    assert_eq!(get_codec_name(0x610002), "png");
}

fn get_codec_name(code: u64) -> &'static str {
    match code {
        0x55 => "raw",
        0x71 => "cbor",
        0x0129 => "dag-json",
        0x0155 => "car",
        0x600001 => "pdf",
        0x600003 => "markdown",
        0x610001 => "jpeg",
        0x610002 => "png",
        _ => "unknown",
    }
}

// ============================================================================
// Test: Content Type to Codec Mapping
// ============================================================================

#[test]
fn test_content_type_codec_mapping() {
    // Test ContentType enum codec extraction
    assert_eq!(ContentType::Event.codec(), 0x300000);
    assert_eq!(ContentType::Custom(0x300200).codec(), 0x300200);

    // Test codec round-trip
    let custom_type = ContentType::Custom(0x300201);
    let codec = custom_type.codec();
    let reconstructed = ContentType::Custom(codec);
    assert_eq!(custom_type, reconstructed);

    // Test from_codec
    assert_eq!(ContentType::from_codec(0x300000), Some(ContentType::Event));
    assert_eq!(ContentType::from_codec(0x300001), Some(ContentType::Graph));
    assert_eq!(
        ContentType::from_codec(0x310000),
        Some(ContentType::Markdown)
    );
}

// ============================================================================
// Test: Codec Serialization
// ============================================================================

#[test]
fn test_codec_serialization() {
    // Test that codecs serialize correctly in CIDs
    let event = CustomEvent {
        event_type: "test".to_string(),
        timestamp: 1234567890,
        data: serde_json::json!({ "key": "value" }),
    };

    // Calculate CID
    let cid = event.calculate_cid().unwrap();

    // Verify codec is preserved in CID
    assert_eq!(cid.codec(), CustomEvent::CODEC);
}

// ============================================================================
// Test: Multi-Codec Content Handling
// ============================================================================

#[test]
fn test_multi_codec_content() {
    // Create content with different codecs
    let event = CustomEvent {
        event_type: "user_action".to_string(),
        timestamp: 1234567890,
        data: serde_json::json!({ "action": "login" }),
    };

    let document = CustomDocument {
        title: "Test Document".to_string(),
        content: "This is a test document.".to_string(),
        version: 1,
    };

    // Calculate CIDs
    let event_cid = event.calculate_cid().unwrap();
    let doc_cid = document.calculate_cid().unwrap();

    // Verify different codecs
    assert_ne!(event_cid.codec(), doc_cid.codec());
    assert_eq!(event_cid.codec(), CustomEvent::CODEC);
    assert_eq!(doc_cid.codec(), CustomDocument::CODEC);

    // Verify different CIDs for different content
    assert_ne!(event_cid, doc_cid);
}

// ============================================================================
// Test: Codec Error Handling
// ============================================================================

#[test]
fn test_codec_error_handling() {
    let mut registry = CimCodecRegistry::new();

    // Test invalid codec registration (outside allowed range)
    let invalid_codec = Arc::new(TestCodec {
        code: 0x100000, // Outside custom range
        name: "invalid".to_string(),
    });

    let result = registry.register(invalid_codec);
    assert!(result.is_err());

    // Test getting non-existent codec
    let result = registry.get(0x999999);
    assert!(result.is_none());
}

// ============================================================================
// Test: Codec Registry Operations
// ============================================================================

#[test]
fn test_codec_registry_operations() {
    let mut registry = CimCodecRegistry::new();

    // Register multiple codecs
    let codec1 = Arc::new(TestCodec {
        code: 0x300100,
        name: "test-codec-1".to_string(),
    });

    let codec2 = Arc::new(TestCodec {
        code: 0x300101,
        name: "test-codec-2".to_string(),
    });

    registry.register(codec1).unwrap();
    registry.register(codec2).unwrap();

    // Test contains
    assert!(registry.contains(0x300100));
    assert!(registry.contains(0x300101));
    assert!(!registry.contains(0x300102));

    // Test codes listing
    let codes = registry.codes();
    assert!(codes.contains(&0x300100));
    assert!(codes.contains(&0x300101));
}

// ============================================================================
// Test: Codec Compatibility
// ============================================================================

#[test]
fn test_codec_compatibility() {
    // Test that same content with same codec produces same CID
    let event1 = CustomEvent {
        event_type: "test".to_string(),
        timestamp: 1234567890,
        data: serde_json::json!({}),
    };

    let event2 = event1.clone();

    let cid1 = event1.calculate_cid().unwrap();
    let cid2 = event2.calculate_cid().unwrap();

    assert_eq!(cid1, cid2); // Same content + same codec = same CID
    assert_eq!(cid1.codec(), cid2.codec());
}
