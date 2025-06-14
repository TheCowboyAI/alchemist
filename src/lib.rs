//! # CIM Keys - Cryptographic Key Management and PKI Support
//!
//! This crate provides comprehensive cryptographic key management for the CIM architecture,
//! including YubiKey hardware token support, PKI infrastructure, and various cryptographic
//! operations.
//!
//! ## Features
//!
//! - **YubiKey Support**: Hardware token integration for secure key storage
//! - **GPG Integration**: OpenPGP operations and key management
//! - **SSH Keys**: SSH key generation, management, and authentication
//! - **X.509/TLS**: Certificate generation, validation, and TLS operations
//! - **PKI Infrastructure**: Complete PKI setup with CA, intermediate, and end-entity certificates
//! - **Encryption/Decryption**: Symmetric and asymmetric encryption operations
//! - **Digital Signatures**: Sign and verify data with various algorithms
//!
//! ## Architecture
//!
//! The crate is organized into several modules:
//! - `yubikey`: YubiKey hardware token operations
//! - `gpg`: GPG/OpenPGP functionality
//! - `ssh`: SSH key management
//! - `tls`: TLS/X.509 certificate operations
//! - `pki`: PKI infrastructure and CA operations
//! - `storage`: Secure key storage and retrieval

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod error;
pub mod types;
pub mod traits;

#[cfg(feature = "yubikey-support")]
pub mod yubikey;

#[cfg(feature = "gpg-support")]
pub mod gpg;

pub mod ssh;
pub mod tls;
pub mod pki;
pub mod storage;

// Re-export commonly used types
pub use error::{KeyError, Result};
pub use types::*;
pub use traits::*;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::error::{KeyError, Result};
    pub use crate::traits::*;
    pub use crate::types::*;

    #[cfg(feature = "yubikey-support")]
    pub use crate::yubikey::YubiKeyManager;

    pub use crate::ssh::SshKeyManager;
    pub use crate::tls::TlsManager;
    pub use crate::pki::PkiManager;
}
