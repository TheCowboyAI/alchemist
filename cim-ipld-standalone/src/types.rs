//! Core types for CIM-IPLD

use serde::{Deserialize, Serialize};

// Re-export CID for convenience
pub use cid::Cid;

/// Content type identifiers for CIM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentType {
    // Core CIM Types (0x300000-0x30FFFF)
    /// Generic event type
    Event,
    /// Graph structure
    Graph,
    /// Graph node
    Node,
    /// Graph edge
    Edge,
    /// Command message
    Command,
    /// Query message
    Query,

    // Document Types (0x310000-0x31FFFF)
    /// Markdown document
    Markdown,
    /// JSON document
    Json,
    /// YAML document
    Yaml,
    /// TOML document
    Toml,

    // Media Types (0x320000-0x32FFFF)
    /// Image file
    Image,
    /// Video file
    Video,
    /// Audio file
    Audio,

    // Extension point for custom types
    /// Custom content type with specific codec
    Custom(u64),
}

impl ContentType {
    /// Get the codec identifier for this content type
    pub fn codec(&self) -> u64 {
        match self {
            // Core types
            ContentType::Event => 0x300000,
            ContentType::Graph => 0x300001,
            ContentType::Node => 0x300002,
            ContentType::Edge => 0x300003,
            ContentType::Command => 0x300004,
            ContentType::Query => 0x300005,

            // Document types
            ContentType::Markdown => 0x310000,
            ContentType::Json => 0x310001,
            ContentType::Yaml => 0x310002,
            ContentType::Toml => 0x310003,

            // Media types
            ContentType::Image => 0x320000,
            ContentType::Video => 0x320001,
            ContentType::Audio => 0x320002,

            // Custom
            ContentType::Custom(codec) => *codec,
        }
    }

    /// Create a ContentType from a codec identifier
    pub fn from_codec(codec: u64) -> Option<Self> {
        match codec {
            0x300000 => Some(ContentType::Event),
            0x300001 => Some(ContentType::Graph),
            0x300002 => Some(ContentType::Node),
            0x300003 => Some(ContentType::Edge),
            0x300004 => Some(ContentType::Command),
            0x300005 => Some(ContentType::Query),

            0x310000 => Some(ContentType::Markdown),
            0x310001 => Some(ContentType::Json),
            0x310002 => Some(ContentType::Yaml),
            0x310003 => Some(ContentType::Toml),

            0x320000 => Some(ContentType::Image),
            0x320001 => Some(ContentType::Video),
            0x320002 => Some(ContentType::Audio),

            // Custom types in valid range
            c if (0x300000..=0x3FFFFF).contains(&c) => Some(ContentType::Custom(c)),
            _ => None,
        }
    }
}
