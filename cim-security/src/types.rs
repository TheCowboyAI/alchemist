//! Common types used throughout the security module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Opaque signature type
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Signature(Vec<u8>);

impl Signature {
    /// Create a new signature from bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Get the signature bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert to bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

/// Opaque public key type
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKey(Vec<u8>);

impl PublicKey {
    /// Create a new public key from bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Get the public key bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert to bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

/// Opaque private key type (for internal use only)
#[derive(Clone, Debug)]
pub struct PrivateKey(Vec<u8>);

impl PrivateKey {
    /// Create a new private key from bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Get the private key bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Metadata for encrypted data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    /// Algorithm used for encryption
    pub algorithm: String,
    /// Key identifier (if applicable)
    pub key_id: Option<String>,
    /// Additional metadata
    pub properties: HashMap<String, String>,
}

/// Encrypted data with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedData {
    /// The encrypted ciphertext
    pub ciphertext: Vec<u8>,
    /// Metadata about the encryption
    pub metadata: EncryptionMetadata,
}

/// Derived key from key derivation function
#[derive(Clone, Debug)]
pub struct DerivedKey(Vec<u8>);

impl DerivedKey {
    /// Create a new derived key from bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Get the derived key bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert to bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

/// Algorithm identifier for cryptographic operations
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Algorithm(String);

impl Algorithm {
    /// Create a new algorithm identifier
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get the algorithm name
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Algorithm {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Algorithm {
    fn from(s: String) -> Self {
        Self(s)
    }
}
