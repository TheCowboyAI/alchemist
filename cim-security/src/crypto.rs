//! Core cryptographic abstractions

use crate::{error::Result, types::*};
use async_trait::async_trait;

/// Trait for signing operations
#[async_trait]
pub trait Signer: Send + Sync {
    /// Sign data and return signature
    async fn sign(&self, data: &[u8]) -> Result<Signature>;

    /// Get the public key for verification
    async fn public_key(&self) -> Result<PublicKey>;

    /// Get the algorithm used for signing
    fn algorithm(&self) -> Algorithm;
}

/// Trait for signature verification
#[async_trait]
pub trait Verifier: Send + Sync {
    /// Verify a signature against data
    async fn verify(&self, data: &[u8], signature: &Signature) -> Result<bool>;

    /// Get the algorithm used for verification
    fn algorithm(&self) -> Algorithm;
}

/// Trait for encryption operations
#[async_trait]
pub trait Encryptor: Send + Sync {
    /// Encrypt data for recipients
    async fn encrypt(&self, data: &[u8], recipients: &[PublicKey]) -> Result<EncryptedData>;

    /// Get the algorithm used for encryption
    fn algorithm(&self) -> Algorithm;
}

/// Trait for decryption operations
#[async_trait]
pub trait Decryptor: Send + Sync {
    /// Decrypt data
    async fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>>;

    /// Check if this decryptor can handle the given encrypted data
    fn can_decrypt(&self, encrypted: &EncryptedData) -> bool {
        self.algorithm().as_str() == encrypted.metadata.algorithm
    }

    /// Get the algorithm used for decryption
    fn algorithm(&self) -> Algorithm;
}

/// Trait for key derivation
#[async_trait]
pub trait KeyDerivation: Send + Sync {
    /// Derive a key from input material
    async fn derive(&self, input: &[u8], context: &[u8]) -> Result<DerivedKey>;

    /// Get the algorithm used for key derivation
    fn algorithm(&self) -> Algorithm;
}

/// Trait for key generation
#[async_trait]
pub trait KeyGenerator: Send + Sync {
    /// Generate a new key pair
    async fn generate(&self) -> Result<(PublicKey, PrivateKey)>;

    /// Get the algorithm for generated keys
    fn algorithm(&self) -> Algorithm;
}

/// Combined trait for signing and verification
#[async_trait]
pub trait SignerVerifier: Signer + Verifier {}

/// Combined trait for encryption and decryption
#[async_trait]
pub trait EncryptorDecryptor: Encryptor + Decryptor {}

/// Trait for cryptographic hash functions
#[async_trait]
pub trait Hasher: Send + Sync {
    /// Hash data
    async fn hash(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Get the hash algorithm
    fn algorithm(&self) -> Algorithm;
}

/// Trait for message authentication codes (MAC)
#[async_trait]
pub trait Mac: Send + Sync {
    /// Generate MAC for data
    async fn generate(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>>;

    /// Verify MAC for data
    async fn verify(&self, key: &[u8], data: &[u8], mac: &[u8]) -> Result<bool>;

    /// Get the MAC algorithm
    fn algorithm(&self) -> Algorithm;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SecurityError;
    use std::collections::HashMap;

    /// Mock signer for testing
    struct MockSigner {
        public_key: PublicKey,
        algorithm: Algorithm,
    }

    #[async_trait]
    impl Signer for MockSigner {
        async fn sign(&self, data: &[u8]) -> Result<Signature> {
            // Simple mock: just hash the data
            let mut sig = vec![0u8; 32];
            for (i, &byte) in data.iter().enumerate() {
                sig[i % 32] ^= byte;
            }
            Ok(Signature::new(sig))
        }

        async fn public_key(&self) -> Result<PublicKey> {
            Ok(self.public_key.clone())
        }

        fn algorithm(&self) -> Algorithm {
            self.algorithm.clone()
        }
    }

    #[tokio::test]
    async fn test_mock_signer() {
        let signer = MockSigner {
            public_key: PublicKey::new(vec![1, 2, 3, 4]),
            algorithm: Algorithm::new("mock"),
        };

        let data = b"test data";
        let signature = signer.sign(data).await.unwrap();
        assert_eq!(signature.as_bytes().len(), 32);

        let pubkey = signer.public_key().await.unwrap();
        assert_eq!(pubkey.as_bytes(), &[1, 2, 3, 4]);
    }
}
