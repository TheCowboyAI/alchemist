//! Codec registry and implementations for CIM-IPLD
//!
//! This module provides codec support for encoding and decoding content
//! in various formats compatible with IPLD.
//!
//! # Example
//!
//! ```
//! use cim_ipld::codec::{CodecRegistry, CimCodec};
//! use cim_ipld::{DagJsonCodec, DagCborCodec};
//! 
//! // Create a codec registry with standard codecs pre-registered
//! let registry = CodecRegistry::new();
//! 
//! // Get codec information
//! let dag_json = DagJsonCodec;
//! assert_eq!(dag_json.code(), 0x0129);
//! assert_eq!(dag_json.name(), "dag-json");
//! 
//! let dag_cbor = DagCborCodec;
//! assert_eq!(dag_cbor.code(), 0x71);
//! assert_eq!(dag_cbor.name(), "dag-cbor");
//! ```

pub mod ipld_codecs;

use crate::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Trait for CIM codecs
pub trait CimCodec: Send + Sync {
    /// Unique codec identifier (0x300000-0x3FFFFF range)
    fn code(&self) -> u64;

    /// Human-readable name for the codec
    fn name(&self) -> &str;
}

/// Registry for CIM codecs
pub struct CodecRegistry {
    codecs: HashMap<u64, Arc<dyn CimCodec>>,
    standard_codecs: HashMap<u64, Arc<dyn CimCodec>>,
}

impl CodecRegistry {
    /// Create a new registry with base codecs
    pub fn new() -> Self {
        let mut registry = Self {
            codecs: HashMap::new(),
            standard_codecs: HashMap::new(),
        };

        // Register base codecs
        registry.register_base_codecs();
        registry
    }

    /// Register base codecs for common content types
    fn register_base_codecs(&mut self) {
        // Register standard IPLD codecs
        let _ = ipld_codecs::register_ipld_codecs(self);
        
        // Register CIM-specific JSON codecs
        let _ = ipld_codecs::register_cim_json_codecs(self);
    }

    /// Register a custom codec
    pub fn register(&mut self, codec: Arc<dyn CimCodec>) -> Result<()> {
        let code = codec.code();

        // Validate codec range
        if !(0x300000..=0x3FFFFF).contains(&code) {
            return Err(Error::InvalidCodecRange(code));
        }

        self.codecs.insert(code, codec);
        Ok(())
    }

    /// Register a standard IPLD codec (no range validation)
    pub fn register_standard(&mut self, codec: Arc<dyn CimCodec>) -> Result<()> {
        let code = codec.code();
        self.standard_codecs.insert(code, codec);
        Ok(())
    }

    /// Get a codec by its code
    pub fn get(&self, code: u64) -> Option<&Arc<dyn CimCodec>> {
        self.codecs.get(&code)
            .or_else(|| self.standard_codecs.get(&code))
    }

    /// Check if a codec is registered
    pub fn contains(&self, code: u64) -> bool {
        self.codecs.contains_key(&code) || self.standard_codecs.contains_key(&code)
    }

    /// Get all registered codec codes
    pub fn codes(&self) -> Vec<u64> {
        let mut codes: Vec<u64> = self.codecs.keys().copied().collect();
        codes.extend(self.standard_codecs.keys().copied());
        codes.sort_unstable();
        codes.dedup();
        codes
    }
}

impl Default for CodecRegistry {
    fn default() -> Self {
        Self::new()
    }
}
