//! NATS configuration for CIM

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// NATS client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfig {
    /// NATS server URL
    pub url: String,

    /// Client name for identification
    pub client_name: String,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// Max reconnect attempts (None = infinite)
    pub max_reconnects: Option<u64>,

    /// JetStream configuration
    pub jetstream: JetStreamConfig,

    /// Security configuration
    pub security: SecurityConfig,
}

/// Security configuration for NATS
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    /// JWT authentication token
    pub jwt: Option<String>,

    /// User credentials file path
    pub credentials_path: Option<String>,

    /// TLS configuration
    pub tls: Option<TlsConfig>,

    /// Username/password authentication
    pub user_password: Option<UserPasswordAuth>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to CA certificate
    pub ca_cert_path: Option<String>,

    /// Path to client certificate
    pub client_cert_path: Option<String>,

    /// Path to client key
    pub client_key_path: Option<String>,

    /// Whether to verify server certificate
    pub verify_server: bool,
}

/// Username/password authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPasswordAuth {
    pub username: String,
    pub password: String,
}

/// JetStream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JetStreamConfig {
    /// Whether JetStream is enabled
    pub enabled: bool,

    /// Default stream configuration
    pub default_stream: StreamConfig,
}

/// Stream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// Stream name prefix
    pub name_prefix: String,

    /// Retention policy
    pub retention: RetentionPolicy,

    /// Maximum age of messages
    pub max_age: Duration,

    /// Maximum number of messages
    pub max_messages: Option<i64>,

    /// Maximum bytes
    pub max_bytes: Option<i64>,

    /// Duplicate window for deduplication
    pub duplicate_window: Duration,
}

/// Retention policy for streams
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RetentionPolicy {
    /// Limits-based retention
    Limits,
    /// Interest-based retention
    Interest,
    /// Work queue retention
    WorkQueue,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: "nats://localhost:4222".to_string(),
            client_name: "information-alchemist".to_string(),
            connection_timeout: Duration::from_secs(10),
            max_reconnects: Some(60),
            jetstream: JetStreamConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl NatsConfig {
    /// Create a localhost configuration
    pub fn localhost() -> Self {
        Self::default()
    }

    /// Create a configuration with custom URL
    pub fn with_url(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Set JWT authentication
    pub fn with_jwt(mut self, jwt: String) -> Self {
        self.security.jwt = Some(jwt);
        self
    }

    /// Set user credentials file
    pub fn with_credentials(mut self, path: String) -> Self {
        self.security.credentials_path = Some(path);
        self
    }

    /// Set username/password authentication
    pub fn with_user_password(mut self, username: String, password: String) -> Self {
        self.security.user_password = Some(UserPasswordAuth { username, password });
        self
    }

    /// Enable TLS with default settings
    pub fn with_tls(mut self) -> Self {
        self.security.tls = Some(TlsConfig {
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
            verify_server: true,
        });
        self
    }
}

impl Default for JetStreamConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_stream: StreamConfig::default(),
        }
    }
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            name_prefix: "event-store".to_string(),
            retention: RetentionPolicy::Limits,
            max_age: Duration::from_secs(365 * 24 * 60 * 60), // 1 year
            max_messages: None,
            max_bytes: None,
            duplicate_window: Duration::from_secs(120), // 2 minutes
        }
    }
}
