//! Information Alchemist specific content types for CIM-IPLD

use cim_ipld::ContentType;

pub mod graph_content;
pub mod conceptual_content;
pub mod workflow_content;
pub mod event_content;

pub use graph_content::GraphContent;

pub use conceptual_content::ConceptualSpaceContent;
pub use workflow_content::WorkflowContent;
pub use event_content::{EventContent, EventChainMetadata};

/// Information Alchemist specific content types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IAContentType {
    /// Graph structure content
    Graph,
    /// Conceptual space content
    ConceptualSpace,
    /// Workflow definition content
    Workflow,
    /// Domain event content
    Event,
}

impl IAContentType {
    /// Get the IPLD codec for this content type
    pub fn codec(&self) -> u64 {
        match self {
            IAContentType::Graph => 0x300100,
            IAContentType::ConceptualSpace => 0x300103,
            IAContentType::Workflow => 0x300104,
            IAContentType::Event => 0x300105,
        }
    }

    /// Convert to CIM ContentType
    pub fn to_content_type(&self) -> ContentType {
        ContentType::Custom(self.codec())
    }
}

/// Registry for Information Alchemist content types
pub struct IAContentRegistry;

impl IAContentRegistry {
    /// Register all IA content types with the CIM codec registry
    pub fn register_codecs(_registry: &mut cim_ipld::CodecRegistry) {
        // Register each content type's codec
        // This will be implemented when we have the specific codec implementations
    }
}
