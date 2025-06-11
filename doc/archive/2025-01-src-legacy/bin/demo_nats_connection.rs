//! # NATS Connection Demo
//!
//! This demo tests the basic NATS connectivity and JetStream functionality.
//! It verifies that the system can establish a connection to NATS server and
//! access JetStream for event persistence.
//!
//! ## Test Flow
//!
//! ```mermaid
//! graph TD
//!     A[Start Demo] --> B[Load Configuration]
//!     B --> C{Config Valid?}
//!     C -->|No| D[Error: Invalid Config]
//!     C -->|Yes| E[Create NATS Client]
//!     E --> F[Connect to NATS Server]
//!     F --> G{Connection Success?}
//!     G -->|No| H[Error: Connection Failed]
//!     G -->|Yes| I[Access JetStream Context]
//!     I --> J{JetStream Available?}
//!     J -->|No| K[Error: JetStream Not Available]
//!     J -->|Yes| L[Log Success]
//!     L --> M[End Demo]
//!     D --> M
//!     H --> M
//!     K --> M
//!
//!     style A fill:#90EE90
//!     style M fill:#FFB6C1
//!     style D fill:#FF6B6B
//!     style H fill:#FF6B6B
//!     style K fill:#FF6B6B
//!     style L fill:#90EE90
//! ```
//!
//! ## What's Being Tested
//!
//! 1. **Configuration Loading**: Validates that NATS configuration can be loaded with default values
//! 2. **Client Creation**: Tests that a NATS client can be instantiated with the configuration
//! 3. **Server Connection**: Verifies network connectivity to the NATS server
//! 4. **JetStream Access**: Ensures JetStream is enabled and accessible for event persistence
//!
//! ## Expected Outcomes
//!
//! - ✅ Successfully connects to NATS server at configured URL
//! - ✅ JetStream context is available for event streaming
//! - ✅ No authentication errors (using default config)
//! - ✅ Logs success message confirming connectivity

use ia::infrastructure::nats::{NatsClient, NatsConfig};
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting NATS Connection Demo");

    // Create NATS configuration
    let config = NatsConfig::localhost();

    // Connect to NATS
    info!("Connecting to NATS at {}", config.url);
    let client = match NatsClient::new(config).await {
        Ok(client) => {
            info!("✓ Successfully connected to NATS");
            client
        }
        Err(e) => {
            error!("✗ Failed to connect to NATS: {}", e);
            return Err(e.into());
        }
    };

    // Verify JetStream is available
    match client.jetstream() {
        Ok(_) => {
            info!("✓ JetStream is enabled and available");
        }
        Err(e) => {
            error!("✗ JetStream is not available: {}", e);
            return Err(e.into());
        }
    }

    // Test basic health check
    match client.health_check().await {
        Ok(_) => {
            info!("✓ NATS health check passed");
        }
        Err(e) => {
            error!("✗ NATS health check failed: {}", e);
            return Err(e.into());
        }
    }

    info!("Demo completed successfully!");
    Ok(())
}
