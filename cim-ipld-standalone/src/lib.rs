//! CIM-IPLD: Content-addressed storage for the Composable Information Machine
//! 
//! This crate provides content-addressed storage using IPLD (InterPlanetary Linked Data)
//! for the Composable Information Machine. It supports various content types, codecs,
//! and provides a chain-based content management system.
//!
//! # Examples
//!
//! ## Basic Content Creation and CID Generation
//!
//! ```
//! use cim_ipld::{DagJsonCodec, CodecOperations};
//! use cid::Cid;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create some content
//! let content = serde_json::json!({
//!     "message": "Hello, IPLD!",
//!     "timestamp": "2024-01-01T00:00:00Z"
//! });
//! 
//! // Encode content
//! let encoded = DagJsonCodec::encode(&content)?;
//! 
//! // Generate CID using blake3
//! let hash = blake3::hash(&encoded);
//! let hash_bytes = hash.as_bytes();
//! 
//! // Create multihash manually with BLAKE3 code (0x1e)
//! let mut multihash_bytes = Vec::new();
//! multihash_bytes.push(0x1e); // BLAKE3-256 code
//! multihash_bytes.push(hash_bytes.len() as u8);
//! multihash_bytes.extend_from_slice(hash_bytes);
//! 
//! let mh = multihash::Multihash::from_bytes(&multihash_bytes)?;
//! let cid = Cid::new_v1(0x0129, mh); // 0x0129 is DAG-JSON codec
//! 
//! println!("Content CID: {}", cid);
//! # Ok(())
//! # }
//! ```
//!
//! ## Working with Different Content Types
//!
//! ```
//! use cim_ipld::{TextDocument, DocumentMetadata};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a text document with metadata
//! let doc = TextDocument {
//!     content: "Hello, world!".to_string(),
//!     metadata: DocumentMetadata {
//!         title: Some("My Document".to_string()),
//!         author: Some("Test Author".to_string()),
//!         ..Default::default()
//!     },
//! };
//! 
//! // TextDocument is a structured type for text content
//! assert_eq!(doc.content, "Hello, world!");
//! assert_eq!(doc.metadata.title, Some("My Document".to_string()));
//! # Ok(())
//! # }
//! ```
//!
//! ## Content Chain Operations
//!
//! ```
//! use cim_ipld::{ContentChain, TextDocument, DocumentMetadata};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a chain for text documents
//! let mut chain = ContentChain::new();
//! 
//! // Add first document
//! let doc1 = TextDocument {
//!     content: "First document".to_string(),
//!     metadata: DocumentMetadata {
//!         title: Some("Document 1".to_string()),
//!         ..Default::default()
//!     },
//! };
//! 
//! chain.append(doc1)?;
//! 
//! // Add second document (automatically linked to first)
//! let doc2 = TextDocument {
//!     content: "Second document".to_string(),
//!     metadata: DocumentMetadata {
//!         title: Some("Document 2".to_string()),
//!         ..Default::default()
//!     },
//! };
//! 
//! chain.append(doc2)?;
//! 
//! // Verify chain properties
//! assert_eq!(chain.len(), 2);
//! 
//! // Get the head to check sequence
//! let head = chain.head().unwrap();
//! assert_eq!(head.sequence, 1);
//! # Ok(())
//! # }
//! ```

pub mod chain;
pub mod codec;
pub mod content_types;
pub mod error;
pub mod traits;
pub mod types;
pub mod object_store;

// Re-exports for convenience
pub use cid::Cid;
pub use multihash::Multihash;

pub use chain::{ChainedContent, ContentChain};
pub use codec::{CimCodec, CodecRegistry};
pub use codec::ipld_codecs::{
    standard, cim_json,
    DagCborCodec, DagJsonCodec, RawCodec, JsonCodec,
    AlchemistJsonCodec, WorkflowGraphJsonCodec, ContextGraphJsonCodec,
    CodecOperations, types as codec_types,
};
pub use error::{Error, Result};
pub use traits::TypedContent;
pub use types::ContentType;

// Re-export content types
pub use content_types::{
    // Document types
    PdfDocument, DocxDocument, MarkdownDocument, TextDocument,
    DocumentMetadata,
    
    // Image types  
    JpegImage, PngImage,
    ImageMetadata,
    
    // Audio types
    WavAudio, Mp3Audio, AacAudio, FlacAudio, OggAudio,
    AudioMetadata,
    
    // Video types
    MovVideo, MkvVideo, Mp4Video,
    VideoMetadata,
    
    // Utilities
    detect_content_type, content_type_name,
};
