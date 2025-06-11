//! NATS messaging infrastructure
//!
//! Provides async messaging capabilities for the domain

use async_nats::Client;
use tracing::info;

use self::error::NatsError;

pub mod client;
pub mod config;
pub mod error;

#[cfg(test)]
mod tests;

pub use client::NatsClient;
pub use config::{NatsConfig, SecurityConfig};
pub use error::Result;



/// Initialize NATS connection with default configuration for localhost
pub async fn connect_localhost() -> Result<Client> {
    let config = NatsConfig::localhost();
    connect_with_config(&config).await
}

/// Connect to NATS with custom configuration
pub async fn connect_with_config(config: &NatsConfig) -> Result<Client> {
    info!("Connecting to NATS at {}", config.url);

    let mut options = async_nats::ConnectOptions::new();

    if let Some(name) = &config.name {
        options = options.name(name);
    }

    // Set max reconnects if specified
    if let Some(max_reconnects) = config.max_reconnects {
        options = options.max_reconnects(max_reconnects as usize);
    }

    // Set connection timeout
    options = options.connection_timeout(config.connection_timeout);

    // Apply security configuration
    if let Some(jwt) = &config.security.jwt {
        // For JWT auth, we need a signing callback
        // This is a simplified version - in production you'd use proper key management
        options = options.jwt(jwt.clone(), |_nonce| async move {
            Err(async_nats::AuthError::new("JWT signing not implemented"))
        });
    }

    if let Some(creds_path) = &config.security.credentials_path {
        options = options
            .credentials_file(creds_path)
            .await
            .map_err(|e| NatsError::Connection(format!("Failed to load credentials: {e}")))?;
    }

    if let Some(user_pass) = &config.security.user_password {
        options = options.user_and_password(user_pass.username.clone(), user_pass.password.clone());
    }

    if let Some(tls) = &config.security.tls {
        if tls.verify_server {
            options = options.require_tls(true);
        }

        // Note: For production, you'd properly configure TLS with certificates
        // This is a simplified version
        if tls.ca_cert_path.is_some() || tls.client_cert_path.is_some() {
            info!("TLS certificate configuration would be applied here in production");
        }
    }

    let client = options.connect(&config.url).await
        .map_err(|e| NatsError::Connection(format!("Failed to connect: {}", e)))?;

    info!("Successfully connected to NATS");
    Ok(client)
}

/// Initialize NATS connection
pub async fn connect(config: NatsConfig) -> Result<NatsClient> {
    NatsClient::new(config).await
}
