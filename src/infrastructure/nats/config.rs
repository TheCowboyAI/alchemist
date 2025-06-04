//! NATS configuration for CIM

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfig {
    /// NATS server URL
    pub url: String,

    /// Client name for identification
    pub client_name: String,

    /// Maximum reconnection attempts
    pub max_reconnects: Option<usize>,

    /// Size of the reconnect buffer
    pub reconnect_buffer_size: usize,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// JetStream configuration
    pub jetstream: JetStreamConfig,

    /// Security configuration
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JetStreamConfig {
    /// Enable JetStream
    pub enabled: bool,

    /// Domain for JetStream isolation
    pub domain: Option<String>,

    /// API prefix for JetStream
    pub api_prefix: Option<String>,

    /// Default stream configuration
    pub default_stream: StreamConfig,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionPolicy {
    Limits,
    Interest,
    WorkQueue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable TLS
    pub tls_enabled: bool,

    /// Path to CA certificate
    pub ca_cert_path: Option<String>,

    /// Client certificate path
    pub client_cert_path: Option<String>,

    /// Client key path
    pub client_key_path: Option<String>,

    /// JWT authentication token
    pub jwt_token: Option<String>,

    /// Username/password authentication
    pub credentials: Option<Credentials>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: "nats://localhost:4222".to_string(),
            client_name: "cim-client".to_string(),
            max_reconnects: Some(10),
            reconnect_buffer_size: 8 * 1024 * 1024, // 8MB
            connection_timeout: Duration::from_secs(10),
            jetstream: JetStreamConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl Default for JetStreamConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            domain: None,
            api_prefix: None,
            default_stream: StreamConfig::default(),
        }
    }
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            name_prefix: "CIM-EVENTS".to_string(),
            retention: RetentionPolicy::Limits,
            max_age: Duration::from_days(365),
            max_messages: None,
            max_bytes: None,
            duplicate_window: Duration::from_secs(120),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            tls_enabled: false,
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
            jwt_token: None,
            credentials: None,
        }
    }
}

impl NatsConfig {
    /// Create configuration for localhost development
    pub fn localhost() -> Self {
        Self::default()
    }

    /// Create configuration for production
    pub fn production(url: String) -> Self {
        Self {
            url,
            client_name: format!("cim-client-{}", uuid::Uuid::new_v4()),
            security: SecurityConfig {
                tls_enabled: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

// Helper trait to convert Duration to various formats
trait DurationExt {
    fn from_days(days: u64) -> Duration;
}

impl DurationExt for Duration {
    fn from_days(days: u64) -> Duration {
        Duration::from_secs(days * 24 * 60 * 60)
    }
}
