//! Secrets management abstractions

use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A reference to a secret
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SecretRef {
    /// Unique identifier for the secret
    pub id: String,
    /// Optional version identifier
    pub version: Option<String>,
}

impl SecretRef {
    /// Create a new secret reference
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: None,
        }
    }

    /// Create a secret reference with a specific version
    pub fn with_version(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: Some(version.into()),
        }
    }
}

/// Metadata about a secret
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecretMetadata {
    /// Unique identifier
    pub id: String,
    /// When the secret was created
    pub created_at: DateTime<Utc>,
    /// When the secret was last updated
    pub updated_at: DateTime<Utc>,
    /// Current version
    pub version: String,
    /// User-defined tags
    pub tags: HashMap<String, String>,
    /// Optional description
    pub description: Option<String>,
    /// Whether the secret is enabled
    pub enabled: bool,
    /// Rotation information
    pub rotation: Option<RotationInfo>,
}

/// Information about secret rotation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RotationInfo {
    /// When the secret was last rotated
    pub last_rotated: DateTime<Utc>,
    /// Next scheduled rotation
    pub next_rotation: Option<DateTime<Utc>>,
    /// Number of times rotated
    pub rotation_count: u32,
}

/// Trait for secret storage
#[async_trait]
pub trait SecretStore: Send + Sync {
    /// Store a secret
    async fn put_secret(
        &self,
        id: &str,
        value: &[u8],
        metadata: HashMap<String, String>,
    ) -> Result<SecretRef>;

    /// Retrieve a secret
    async fn get_secret(&self, secret_ref: &SecretRef) -> Result<Vec<u8>>;

    /// Delete a secret
    async fn delete_secret(&self, secret_ref: &SecretRef) -> Result<()>;

    /// List secrets with optional filter
    async fn list_secrets(&self, filter: Option<&str>) -> Result<Vec<SecretMetadata>>;

    /// Get metadata for a secret
    async fn get_metadata(&self, secret_ref: &SecretRef) -> Result<SecretMetadata>;

    /// Update metadata for a secret
    async fn update_metadata(
        &self,
        secret_ref: &SecretRef,
        metadata: HashMap<String, String>,
    ) -> Result<()>;

    /// Check if a secret exists
    async fn exists(&self, secret_ref: &SecretRef) -> Result<bool>;
}

/// Trait for secret rotation
#[async_trait]
pub trait SecretRotation: Send + Sync {
    /// Rotate a secret
    async fn rotate_secret(&self, secret_ref: &SecretRef) -> Result<SecretRef>;

    /// Get rotation policy
    async fn rotation_policy(&self, secret_ref: &SecretRef) -> Result<RotationPolicy>;

    /// Set rotation policy
    async fn set_rotation_policy(
        &self,
        secret_ref: &SecretRef,
        policy: RotationPolicy,
    ) -> Result<()>;

    /// Check if rotation is needed
    async fn needs_rotation(&self, secret_ref: &SecretRef) -> Result<bool>;
}

/// Rotation policy for secrets
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RotationPolicy {
    /// How often to rotate
    pub rotation_period: Duration,
    /// When the policy was last updated
    pub last_rotated: DateTime<Utc>,
    /// Whether automatic rotation is enabled
    pub auto_rotate: bool,
    /// Maximum versions to keep
    pub max_versions: Option<u32>,
}

impl RotationPolicy {
    /// Create a new rotation policy
    pub fn new(rotation_period: Duration) -> Self {
        Self {
            rotation_period,
            last_rotated: Utc::now(),
            auto_rotate: true,
            max_versions: Some(10),
        }
    }

    /// Check if rotation is due
    pub fn is_rotation_due(&self) -> bool {
        let next_rotation = self.last_rotated + self.rotation_period;
        Utc::now() >= next_rotation
    }
}

/// Trait for secret encryption at rest
#[async_trait]
pub trait SecretEncryption: Send + Sync {
    /// Encrypt a secret value
    async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>>;

    /// Decrypt a secret value
    async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>>;
}

/// Options for secret retrieval
#[derive(Clone, Debug, Default)]
pub struct GetSecretOptions {
    /// Specific version to retrieve
    pub version: Option<String>,
    /// Whether to decrypt the value
    pub decrypt: bool,
}

/// Options for secret storage
#[derive(Clone, Debug, Default)]
pub struct PutSecretOptions {
    /// Description of the secret
    pub description: Option<String>,
    /// Tags to apply
    pub tags: HashMap<String, String>,
    /// Whether to encrypt the value
    pub encrypt: bool,
    /// Rotation policy to apply
    pub rotation_policy: Option<RotationPolicy>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SecurityError;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Mock secret store for testing
    struct MockSecretStore {
        secrets: Arc<Mutex<HashMap<String, Vec<u8>>>>,
        metadata: Arc<Mutex<HashMap<String, SecretMetadata>>>,
    }

    impl MockSecretStore {
        fn new() -> Self {
            Self {
                secrets: Arc::new(Mutex::new(HashMap::new())),
                metadata: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl SecretStore for MockSecretStore {
        async fn put_secret(
            &self,
            id: &str,
            value: &[u8],
            tags: HashMap<String, String>,
        ) -> Result<SecretRef> {
            let mut secrets = self.secrets.lock().await;
            let mut metadata_store = self.metadata.lock().await;

            secrets.insert(id.to_string(), value.to_vec());

            let metadata = SecretMetadata {
                id: id.to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                version: "1".to_string(),
                tags,
                description: None,
                enabled: true,
                rotation: None,
            };

            metadata_store.insert(id.to_string(), metadata);

            Ok(SecretRef::new(id))
        }

        async fn get_secret(&self, secret_ref: &SecretRef) -> Result<Vec<u8>> {
            let secrets = self.secrets.lock().await;
            secrets
                .get(&secret_ref.id)
                .cloned()
                .ok_or_else(|| SecurityError::SecretNotFound(secret_ref.id.clone()))
        }

        async fn delete_secret(&self, secret_ref: &SecretRef) -> Result<()> {
            let mut secrets = self.secrets.lock().await;
            let mut metadata_store = self.metadata.lock().await;

            secrets.remove(&secret_ref.id);
            metadata_store.remove(&secret_ref.id);
            Ok(())
        }

        async fn list_secrets(&self, _filter: Option<&str>) -> Result<Vec<SecretMetadata>> {
            let metadata_store = self.metadata.lock().await;
            Ok(metadata_store.values().cloned().collect())
        }

        async fn get_metadata(&self, secret_ref: &SecretRef) -> Result<SecretMetadata> {
            let metadata_store = self.metadata.lock().await;
            metadata_store
                .get(&secret_ref.id)
                .cloned()
                .ok_or_else(|| SecurityError::SecretNotFound(secret_ref.id.clone()))
        }

        async fn update_metadata(
            &self,
            secret_ref: &SecretRef,
            tags: HashMap<String, String>,
        ) -> Result<()> {
            let mut metadata_store = self.metadata.lock().await;
            if let Some(metadata) = metadata_store.get_mut(&secret_ref.id) {
                metadata.tags = tags;
                metadata.updated_at = Utc::now();
                Ok(())
            } else {
                Err(SecurityError::SecretNotFound(secret_ref.id.clone()))
            }
        }

        async fn exists(&self, secret_ref: &SecretRef) -> Result<bool> {
            let secrets = self.secrets.lock().await;
            Ok(secrets.contains_key(&secret_ref.id))
        }
    }

    #[tokio::test]
    async fn test_secret_store() {
        let store = MockSecretStore::new();

        // Store a secret
        let secret_ref = store
            .put_secret("test-secret", b"secret-value", HashMap::new())
            .await
            .unwrap();

        // Retrieve the secret
        let value = store.get_secret(&secret_ref).await.unwrap();
        assert_eq!(value, b"secret-value");

        // Check existence
        assert!(store.exists(&secret_ref).await.unwrap());

        // Delete the secret
        store.delete_secret(&secret_ref).await.unwrap();

        // Verify deletion
        assert!(!store.exists(&secret_ref).await.unwrap());
    }

    #[test]
    fn test_rotation_policy() {
        let policy = RotationPolicy::new(Duration::days(30));
        assert!(policy.auto_rotate);
        assert_eq!(policy.max_versions, Some(10));

        // Check if rotation is due (should not be immediately after creation)
        assert!(!policy.is_rotation_due());
    }
}
