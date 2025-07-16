//! Error types for CIM-IPLD

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to serialize content: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("CBOR serialization error: {0}")]
    CborError(String),

    #[error("Invalid CID: {0}")]
    InvalidCid(String),

    #[error("Chain validation failed: expected {expected}, got {actual}")]
    ChainValidationError { expected: String, actual: String },

    #[error("Sequence validation failed: expected {expected}, got {actual}")]
    SequenceValidationError { expected: u64, actual: u64 },

    #[error("Invalid codec range: {0}. Must be in range 0x300000-0x3FFFFF")]
    InvalidCodecRange(u64),

    #[error("Codec not found: {0}")]
    CodecNotFound(u64),

    #[error("Content type mismatch: expected {expected:?}, got {actual:?}")]
    ContentTypeMismatch { expected: String, actual: String },

    #[error("Multihash error: {0}")]
    MultihashError(String),

    #[error("Invalid content: {0}")]
    InvalidContent(String),
}

pub type Result<T> = std::result::Result<T, Error>;
