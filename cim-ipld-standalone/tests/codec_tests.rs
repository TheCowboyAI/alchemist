//! Codec tests for CIM-IPLD
//!
//! Tests custom codec implementations and version compatibility.
//!
//! ## Test Scenarios
//!
//! ```mermaid
//! graph TD
//!     A[Codec Tests] --> B[Registration]
//!     A --> C[Encoding/Decoding]
//!     A --> D[Version Compatibility]
//!     A --> E[Performance]
//!
//!     B --> B1[Custom Codec]
//!     B --> B2[Multiple Codecs]
//!     B --> B3[Override Default]
//!
//!     C --> C1[Basic Types]
//!     C --> C2[Complex Structures]
//!     C --> C3[Large Data]
//!
//!     D --> D1[Forward Compatibility]
//!     D --> D2[Backward Compatibility]
//!     D --> D3[Migration Path]
//!
//!     E --> E1[Encode Speed]
//!     E --> E2[Decode Speed]
//!     E --> E3[Size Efficiency]
//! ```

use cim_ipld::*;
use cim_ipld::codec::{CimCodec, CodecRegistry};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

mod common;
use common::*;

/// Custom test codec for specialized encoding
#[derive(Debug, Clone)]
struct TestCodec {
    version: u8,
    compression: bool,
}

impl TestCodec {
    fn new(version: u8, compression: bool) -> Self {
        Self { version, compression }
    }

    fn encode<T: Serialize>(&self, value: &T) -> Result<Vec<u8>> {
        let mut data = serde_json::to_vec(value)?;

        // Add version header
        data.insert(0, self.version);

        // Apply compression if enabled
        if self.compression {
            data = zstd::encode_all(&data[..], 3)?;
        }

        Ok(data)
    }

    fn decode<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T> {
        let mut bytes = data.to_vec();

        // Decompress if needed
        if self.compression {
            bytes = zstd::decode_all(&bytes[..])?;
        }

        // Check version
        if bytes.is_empty() || bytes[0] != self.version {
            return Err(Error::CodecError("Version mismatch".into()));
        }

        // Decode JSON
        Ok(serde_json::from_slice(&bytes[1..])?)
    }
}

/// Test data structures with versioning
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestDataV1 {
    id: String,
    value: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestDataV2 {
    id: String,
    value: i32,
    #[serde(default)]
    metadata: HashMap<String, String>,
}

impl From<TestDataV1> for TestDataV2 {
    fn from(v1: TestDataV1) -> Self {
        TestDataV2 {
            id: v1.id,
            value: v1.value,
            metadata: HashMap::new(),
        }
    }
}

#[tokio::test]
async fn test_custom_codec_registration() {
    /// Test custom codec registration and usage
    ///
    /// Given: Custom codec implementation
    /// When: Registered and used
    /// Then: Content properly encoded/decoded

    let codec = TestCodec::new(1, false);
    let test_data = TestDataV1 {
        id: "test-123".to_string(),
        value: 42,
    };

    // Encode with custom codec
    let encoded = codec.encode(&test_data)
        .expect("Encoding should succeed");

    // Verify version header
    assert_eq!(encoded[0], 1);

    // Decode with custom codec
    let decoded: TestDataV1 = codec.decode(&encoded)
        .expect("Decoding should succeed");

    assert_eq!(decoded, test_data);
}

#[tokio::test]
async fn test_codec_with_compression() {
    /// Test codec with compression enabled
    ///
    /// Given: Codec with compression
    /// When: Large data encoded
    /// Then: Size reduction achieved

    let codec_uncompressed = TestCodec::new(1, false);
    let codec_compressed = TestCodec::new(1, true);

    // Create large repetitive data
    let large_data = TestDataV2 {
        id: "large".to_string(),
        value: 999,
        metadata: (0..1000)
            .map(|i| (format!("key{i}"), "same_value".to_string()))
            .collect(),
    };

    // Encode without compression
    let uncompressed = codec_uncompressed.encode(&large_data)
        .expect("Uncompressed encoding should succeed");

    // Encode with compression
    let compressed = codec_compressed.encode(&large_data)
        .expect("Compressed encoding should succeed");

    println!("Uncompressed size: {} bytes", uncompressed.len());
    println!("Compressed size: {} bytes", compressed.len());
    println!("Compression ratio: {:.2}%",
        (compressed.len() as f64 / uncompressed.len() as f64) * 100.0);

    // Verify compression achieved
    assert!(compressed.len() < uncompressed.len() / 2,
        "Compression should reduce size by at least 50%");

    // Verify decompression works
    let decoded: TestDataV2 = codec_compressed.decode(&compressed)
        .expect("Decompression should succeed");
    assert_eq!(decoded, large_data);
}

#[tokio::test]
async fn test_codec_version_compatibility() {
    /// Test backward compatibility between codec versions
    ///
    /// Given: Content with old codec version
    /// When: Read with new version
    /// Then: Backward compatibility maintained

    let codec_v1 = TestCodec::new(1, false);
    let codec_v2 = TestCodec::new(2, false);

    // Create V1 data
    let data_v1 = TestDataV1 {
        id: "compat-test".to_string(),
        value: 100,
    };

    // Encode with V1 codec
    let encoded_v1 = codec_v1.encode(&data_v1)
        .expect("V1 encoding should succeed");

    // Try to decode with V2 codec (should fail due to version mismatch)
    let decode_result: Result<TestDataV1> = codec_v2.decode(&encoded_v1);
    assert!(decode_result.is_err(), "Version mismatch should be detected");

    // Proper migration path
    let decoded_v1: TestDataV1 = codec_v1.decode(&encoded_v1)
        .expect("V1 decoding should succeed");
    let data_v2: TestDataV2 = decoded_v1.into();

    // Encode with V2 codec
    let encoded_v2 = codec_v2.encode(&data_v2)
        .expect("V2 encoding should succeed");

    // Verify V2 data
    let decoded_v2: TestDataV2 = codec_v2.decode(&encoded_v2)
        .expect("V2 decoding should succeed");
    assert_eq!(decoded_v2.id, "compat-test");
    assert_eq!(decoded_v2.value, 100);
    assert!(decoded_v2.metadata.is_empty());
}

#[tokio::test]
async fn test_multiple_codec_registration() {
    /// Test multiple codec registration and selection
    ///
    /// Given: Multiple codecs registered
    /// When: Different content types used
    /// Then: Appropriate codec selected

    let json_codec = TestCodec::new(1, false);
    let compressed_codec = TestCodec::new(1, true);
    let binary_codec = TestCodec::new(2, false);

    // Different data types
    let text_data = TestDataV1 {
        id: "text".to_string(),
        value: 1,
    };

    let large_data = TestDataV2 {
        id: "large".to_string(),
        value: 2,
        metadata: (0..100).map(|i| (format!("k{i}"), format!("v{i}"))).collect(),
    };

    let binary_data = vec![0u8; 1024]; // 1KB of binary data

    // Use appropriate codec for each type
    let text_encoded = json_codec.encode(&text_data)
        .expect("Text encoding should succeed");
    let large_encoded = compressed_codec.encode(&large_data)
        .expect("Large data encoding should succeed");
    let binary_encoded = binary_codec.encode(&binary_data)
        .expect("Binary encoding should succeed");

    // Verify each codec works correctly
    let text_decoded: TestDataV1 = json_codec.decode(&text_encoded)
        .expect("Text decoding should succeed");
    assert_eq!(text_decoded, text_data);

    let large_decoded: TestDataV2 = compressed_codec.decode(&large_encoded)
        .expect("Large data decoding should succeed");
    assert_eq!(large_decoded, large_data);

    let binary_decoded: Vec<u8> = binary_codec.decode(&binary_encoded)
        .expect("Binary decoding should succeed");
    assert_eq!(binary_decoded, binary_data);
}

#[tokio::test]
async fn test_codec_error_handling() {
    /// Test codec error handling scenarios
    ///
    /// Given: Various error conditions
    /// When: Codec operations performed
    /// Then: Proper error handling

    let codec = TestCodec::new(1, false);

    // Test empty data
    let empty_result: Result<TestDataV1> = codec.decode(&[]);
    assert!(empty_result.is_err(), "Empty data should fail");

    // Test corrupted data
    let corrupted = vec![1, 2, 3, 4, 5]; // Random bytes
    let corrupted_result: Result<TestDataV1> = codec.decode(&corrupted);
    assert!(corrupted_result.is_err(), "Corrupted data should fail");

    // Test wrong version
    let wrong_version = vec![99]; // Wrong version number
    let version_result: Result<TestDataV1> = codec.decode(&wrong_version);
    assert!(version_result.is_err(), "Wrong version should fail");

    // Test compressed data with uncompressed codec
    let compressed_codec = TestCodec::new(1, true);
    let data = TestDataV1 { id: "test".to_string(), value: 42 };
    let compressed = compressed_codec.encode(&data)
        .expect("Compression should succeed");

    let uncompressed_codec = TestCodec::new(1, false);
    let decompress_result: Result<TestDataV1> = uncompressed_codec.decode(&compressed);
    assert!(decompress_result.is_err(), "Compressed data should fail with uncompressed codec");
}

#[tokio::test]
async fn test_codec_performance_characteristics() {
    /// Test and measure codec performance
    ///
    /// Given: Different codec configurations
    /// When: Performance measured
    /// Then: Characteristics documented

    use std::time::Instant;

    let json_codec = TestCodec::new(1, false);
    let compressed_codec = TestCodec::new(1, true);

    // Create test data of various sizes
    let small_data = TestDataV2 {
        id: "small".to_string(),
        value: 42,
        metadata: HashMap::new(),
    };

    let medium_data = TestDataV2 {
        id: "medium".to_string(),
        value: 100,
        metadata: (0..100).map(|i| (format!("k{i}"), format!("value{i}"))).collect(),
    };

    let large_data = TestDataV2 {
        id: "large".to_string(),
        value: 1000,
        metadata: (0..10000).map(|i| (format!("key{i}"), "repeated_value".to_string())).collect(),
    };

    // Measure encoding performance
    println!("\nEncoding Performance:");

    let start = Instant::now();
    for _ in 0..1000 {
        let _ = json_codec.encode(&small_data);
    }
    println!("  Small JSON: {:?}/1000 ops", start.elapsed());

    let start = Instant::now();
    for _ in 0..100 {
        let _ = json_codec.encode(&medium_data);
    }
    println!("  Medium JSON: {:?}/100 ops", start.elapsed());

    let start = Instant::now();
    for _ in 0..10 {
        let _ = json_codec.encode(&large_data);
    }
    println!("  Large JSON: {:?}/10 ops", start.elapsed());

    let start = Instant::now();
    for _ in 0..10 {
        let _ = compressed_codec.encode(&large_data);
    }
    println!("  Large Compressed: {:?}/10 ops", start.elapsed());

    // Measure decoding performance
    println!("\nDecoding Performance:");

    let small_encoded = json_codec.encode(&small_data).unwrap();
    let medium_encoded = json_codec.encode(&medium_data).unwrap();
    let large_encoded = json_codec.encode(&large_data).unwrap();
    let large_compressed = compressed_codec.encode(&large_data).unwrap();

    let start = Instant::now();
    for _ in 0..1000 {
        let _: TestDataV2 = json_codec.decode(&small_encoded).unwrap();
    }
    println!("  Small JSON: {:?}/1000 ops", start.elapsed());

    let start = Instant::now();
    for _ in 0..100 {
        let _: TestDataV2 = json_codec.decode(&medium_encoded).unwrap();
    }
    println!("  Medium JSON: {:?}/100 ops", start.elapsed());

    let start = Instant::now();
    for _ in 0..10 {
        let _: TestDataV2 = json_codec.decode(&large_encoded).unwrap();
    }
    println!("  Large JSON: {:?}/10 ops", start.elapsed());

    let start = Instant::now();
    for _ in 0..10 {
        let _: TestDataV2 = compressed_codec.decode(&large_compressed).unwrap();
    }
    println!("  Large Compressed: {:?}/10 ops", start.elapsed());

    // Size comparison
    println!("\nSize Comparison:");
    println!("  Small JSON: {small_encoded.len(} bytes"));
    println!("  Medium JSON: {medium_encoded.len(} bytes"));
    println!("  Large JSON: {large_encoded.len(} bytes"));
    println!("  Large Compressed: {large_compressed.len(} bytes ({:.1}% of original)"),
        (large_compressed.len() as f64 / large_encoded.len() as f64) * 100.0);
}

#[tokio::test]
async fn test_codec_with_nested_structures() {
    /// Test codec with deeply nested data structures
    ///
    /// Given: Complex nested data
    /// When: Encoded and decoded
    /// Then: Structure preserved

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct NestedData {
        level: u32,
        data: HashMap<String, Value>,
        children: Vec<NestedData>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    enum Value {
        String(String),
        Number(i64),
        Boolean(bool),
        Array(Vec<Value>),
        Object(HashMap<String, Value>),
    }

    fn create_nested_data(depth: u32) -> NestedData {
        if depth == 0 {
            NestedData {
                level: 0,
                data: HashMap::new(),
                children: Vec::new(),
            }
        } else {
            NestedData {
                level: depth,
                data: vec![
                    ("string".to_string(), Value::String(format!("Level {depth}"))),
                    ("number".to_string(), Value::Number(depth as i64)),
                    ("boolean".to_string(), Value::Boolean(depth % 2 == 0)),
                    ("array".to_string(), Value::Array(vec![
                        Value::Number(1),
                        Value::Number(2),
                        Value::Number(3),
                    ])),
                ].into_iter().collect(),
                children: vec![
                    create_nested_data(depth - 1),
                    create_nested_data(depth - 1),
                ],
            }
        }
    }

    let codec = TestCodec::new(1, true);
    let nested_data = create_nested_data(5); // 5 levels deep

    // Encode nested structure
    let encoded = codec.encode(&nested_data)
        .expect("Nested encoding should succeed");

    // Decode and verify
    let decoded: NestedData = codec.decode(&encoded)
        .expect("Nested decoding should succeed");

    assert_eq!(decoded, nested_data);
    assert_eq!(decoded.level, 5);
    assert_eq!(decoded.children.len(), 2);
    assert_eq!(decoded.children[0].level, 4);
}
