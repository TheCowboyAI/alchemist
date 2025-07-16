//! Infrastructure Layer 1.3: Content Addressing Tests for cim-ipld
//! 
//! User Story: As a content-addressed storage system, I need to transform and store various content types
//!
//! Test Requirements:
//! - Verify content transformation to IPLD
//! - Verify codec selection for content types
//! - Verify content addressing consistency
//! - Verify content type detection
//!
//! Event Sequence:
//! 1. ContentReceived { content_type, size }
//! 2. CodecSelected { content_type, codec }
//! 3. ContentTransformed { from_type, to_type, cid }
//! 4. ContentAddressed { cid, content_type, size }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Receive Content]
//!     B --> C[ContentReceived]
//!     C --> D[Select Codec]
//!     D --> E[CodecSelected]
//!     E --> F[Transform Content]
//!     F --> G[ContentTransformed]
//!     G --> H[Address Content]
//!     H --> I[ContentAddressed]
//!     I --> J[Test Success]
//! ```

use std::collections::HashMap;

/// Content types supported by IPLD
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IPLDContentType {
    Text,
    Json,
    Cbor,
    Image,
    Audio,
    Binary,
}

/// IPLD codec types
#[derive(Debug, Clone, PartialEq)]
pub enum IPLDCodec {
    DagJson,
    DagCbor,
    Raw,
}

/// Content addressing event types
#[derive(Debug, Clone, PartialEq)]
pub enum ContentAddressingEvent {
    ContentReceived { content_type: IPLDContentType, size: usize },
    CodecSelected { content_type: IPLDContentType, codec: IPLDCodec },
    ContentTransformed { from_type: IPLDContentType, to_type: IPLDContentType, cid: String },
    ContentAddressed { cid: String, content_type: IPLDContentType, size: usize },
}

/// Content transformer for IPLD
pub struct IPLDContentTransformer {
    codec_mapping: HashMap<IPLDContentType, IPLDCodec>,
}

impl IPLDContentTransformer {
    pub fn new() -> Self {
        let mut codec_mapping = HashMap::new();
        codec_mapping.insert(IPLDContentType::Text, IPLDCodec::Raw);
        codec_mapping.insert(IPLDContentType::Json, IPLDCodec::DagJson);
        codec_mapping.insert(IPLDContentType::Cbor, IPLDCodec::DagCbor);
        codec_mapping.insert(IPLDContentType::Image, IPLDCodec::Raw);
        codec_mapping.insert(IPLDContentType::Audio, IPLDCodec::Raw);
        codec_mapping.insert(IPLDContentType::Binary, IPLDCodec::Raw);

        Self { codec_mapping }
    }

    pub fn detect_content_type(&self, content: &[u8]) -> IPLDContentType {
        // Simple content type detection for testing
        if content.starts_with(b"{") || content.starts_with(b"[") {
            IPLDContentType::Json
        } else if content.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            IPLDContentType::Image // PNG
        } else if content.starts_with(&[0xFF, 0xD8, 0xFF]) {
            IPLDContentType::Image // JPEG
        } else if content.iter().all(|&b| b.is_ascii()) {
            IPLDContentType::Text
        } else {
            IPLDContentType::Binary
        }
    }

    pub fn select_codec(&self, content_type: &IPLDContentType) -> IPLDCodec {
        self.codec_mapping
            .get(content_type)
            .cloned()
            .unwrap_or(IPLDCodec::Raw)
    }

    pub fn transform_content(
        &self,
        content: &[u8],
        from_type: &IPLDContentType,
        codec: &IPLDCodec,
    ) -> Result<Vec<u8>, String> {
        match (from_type, codec) {
            (IPLDContentType::Json, IPLDCodec::DagJson) => {
                // Validate JSON and return as-is
                if std::str::from_utf8(content).is_ok() {
                    Ok(content.to_vec())
                } else {
                    Err("Invalid UTF-8 for JSON content".to_string())
                }
            }
            (IPLDContentType::Json, IPLDCodec::DagCbor) => {
                // Transform JSON to CBOR (mock)
                let mut cbor = vec![0xA1]; // CBOR map marker
                cbor.extend_from_slice(content);
                Ok(cbor)
            }
            (_, IPLDCodec::Raw) => {
                // Raw codec returns content as-is
                Ok(content.to_vec())
            }
            _ => Err(format!(
                "Unsupported transformation from {:?} to {:?}",
                from_type, codec
            )),
        }
    }

    pub fn calculate_cid(&self, content: &[u8], codec: &IPLDCodec) -> String {
        // Mock CID calculation based on content and codec
        let codec_prefix = match codec {
            IPLDCodec::DagJson => "bagj",
            IPLDCodec::DagCbor => "bagc",
            IPLDCodec::Raw => "bafk",
        };

        let hash = content.iter().fold(0u64, |acc, &b| {
            acc.wrapping_mul(31).wrapping_add(b as u64)
        });

        format!("{codec_prefix}{:032x}", hash)
    }
}

/// Content addressing system
pub struct ContentAddressingSystem {
    transformer: IPLDContentTransformer,
    stored_content: HashMap<String, (Vec<u8>, IPLDContentType, IPLDCodec)>,
}

impl ContentAddressingSystem {
    pub fn new() -> Self {
        Self {
            transformer: IPLDContentTransformer::new(),
            stored_content: HashMap::new(),
        }
    }

    pub fn process_content(
        &mut self,
        content: Vec<u8>,
    ) -> Result<(String, Vec<ContentAddressingEvent>), String> {
        let mut events = Vec::new();

        // Detect content type
        let content_type = self.transformer.detect_content_type(&content);
        events.push(ContentAddressingEvent::ContentReceived {
            content_type: content_type.clone(),
            size: content.len(),
        });

        // Select codec
        let codec = self.transformer.select_codec(&content_type);
        events.push(ContentAddressingEvent::CodecSelected {
            content_type: content_type.clone(),
            codec: codec.clone(),
        });

        // Transform content if needed
        let transformed_content = self.transformer.transform_content(
            &content,
            &content_type,
            &codec,
        )?;

        // Calculate CID
        let cid = self.transformer.calculate_cid(&transformed_content, &codec);
        
        if content != transformed_content {
            events.push(ContentAddressingEvent::ContentTransformed {
                from_type: content_type.clone(),
                to_type: content_type.clone(), // Type remains same, format changes
                cid: cid.clone(),
            });
        }

        // Store content
        self.stored_content.insert(
            cid.clone(),
            (transformed_content, content_type.clone(), codec),
        );

        events.push(ContentAddressingEvent::ContentAddressed {
            cid: cid.clone(),
            content_type,
            size: content.len(),
        });

        Ok((cid, events))
    }

    pub fn retrieve_content(&self, cid: &str) -> Option<&(Vec<u8>, IPLDContentType, IPLDCodec)> {
        self.stored_content.get(cid)
    }

    pub fn verify_content_consistency(&self, cid: &str) -> Result<bool, String> {
        let (content, _, codec) = self.stored_content.get(cid)
            .ok_or_else(|| format!("Content with CID {cid} not found"))?;

        let recalculated_cid = self.transformer.calculate_cid(content, codec);
        Ok(recalculated_cid == cid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_detection() {
        // Arrange
        let transformer = IPLDContentTransformer::new();

        // Act & Assert
        assert_eq!(
            transformer.detect_content_type(b"{\"key\": \"value\"}"),
            IPLDContentType::Json
        );
        
        assert_eq!(
            transformer.detect_content_type(b"plain text content"),
            IPLDContentType::Text
        );
        
        assert_eq!(
            transformer.detect_content_type(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A]),
            IPLDContentType::Image
        );
        
        assert_eq!(
            transformer.detect_content_type(&[0xFF, 0xFE, 0x00, 0x01]),
            IPLDContentType::Binary
        );
    }

    #[test]
    fn test_codec_selection() {
        // Arrange
        let transformer = IPLDContentTransformer::new();

        // Act & Assert
        assert_eq!(
            transformer.select_codec(&IPLDContentType::Json),
            IPLDCodec::DagJson
        );
        
        assert_eq!(
            transformer.select_codec(&IPLDContentType::Text),
            IPLDCodec::Raw
        );
        
        assert_eq!(
            transformer.select_codec(&IPLDContentType::Cbor),
            IPLDCodec::DagCbor
        );
    }

    #[test]
    fn test_content_processing_json() {
        // Arrange
        let mut system = ContentAddressingSystem::new();
        let json_content = b"{\"test\": \"data\"}".to_vec();

        // Act
        let (cid, events) = system.process_content(json_content.clone()).unwrap();

        // Assert
        assert_eq!(events.len(), 3); // Received, Selected, Addressed
        
        assert_eq!(events[0], ContentAddressingEvent::ContentReceived {
            content_type: IPLDContentType::Json,
            size: json_content.len(),
        });
        
        assert_eq!(events[1], ContentAddressingEvent::CodecSelected {
            content_type: IPLDContentType::Json,
            codec: IPLDCodec::DagJson,
        });
        
        assert!(cid.starts_with("bagj")); // DagJson prefix
    }

    #[test]
    fn test_content_retrieval() {
        // Arrange
        let mut system = ContentAddressingSystem::new();
        let content = b"retrieve this content".to_vec();

        // Act
        let (cid, _) = system.process_content(content.clone()).unwrap();
        let retrieved = system.retrieve_content(&cid);

        // Assert
        assert!(retrieved.is_some());
        let (stored_content, content_type, codec) = retrieved.unwrap();
        assert_eq!(stored_content, &content);
        assert_eq!(*content_type, IPLDContentType::Text);
        assert_eq!(*codec, IPLDCodec::Raw);
    }

    #[test]
    fn test_content_consistency() {
        // Arrange
        let mut system = ContentAddressingSystem::new();
        let content = b"consistent content".to_vec();

        // Act
        let (cid, _) = system.process_content(content).unwrap();
        let is_consistent = system.verify_content_consistency(&cid).unwrap();

        // Assert
        assert!(is_consistent);
    }

    #[test]
    fn test_json_to_cbor_transformation() {
        // Arrange
        let transformer = IPLDContentTransformer::new();
        let json_content = b"{\"transform\": \"me\"}";

        // Act
        let result = transformer.transform_content(
            json_content,
            &IPLDContentType::Json,
            &IPLDCodec::DagCbor,
        ).unwrap();

        // Assert
        assert!(result.starts_with(&[0xA1])); // CBOR map marker
        assert!(result.len() > json_content.len()); // CBOR adds prefix
    }

    #[test]
    fn test_multiple_content_types() {
        // Arrange
        let mut system = ContentAddressingSystem::new();
        
        let text_content = b"plain text".to_vec();
        let json_content = b"{\"type\": \"json\"}".to_vec();
        let binary_content = vec![0xFF, 0xFE, 0x00, 0x01, 0x02, 0x03];

        // Act
        let (text_cid, _) = system.process_content(text_content).unwrap();
        let (json_cid, _) = system.process_content(json_content).unwrap();
        let (binary_cid, _) = system.process_content(binary_content).unwrap();

        // Assert
        assert!(text_cid.starts_with("bafk")); // Raw codec
        assert!(json_cid.starts_with("bagj")); // DagJson codec
        assert!(binary_cid.starts_with("bafk")); // Raw codec

        // Verify all are stored
        assert!(system.retrieve_content(&text_cid).is_some());
        assert!(system.retrieve_content(&json_cid).is_some());
        assert!(system.retrieve_content(&binary_cid).is_some());
    }

    #[test]
    fn test_cid_uniqueness() {
        // Arrange
        let mut system = ContentAddressingSystem::new();
        
        let content1 = b"unique content 1".to_vec();
        let content2 = b"unique content 2".to_vec();
        let duplicate = b"unique content 1".to_vec();

        // Act
        let (cid1, _) = system.process_content(content1).unwrap();
        let (cid2, _) = system.process_content(content2).unwrap();
        let (cid_dup, _) = system.process_content(duplicate).unwrap();

        // Assert
        assert_ne!(cid1, cid2); // Different content = different CIDs
        assert_eq!(cid1, cid_dup); // Same content = same CID
    }
} 