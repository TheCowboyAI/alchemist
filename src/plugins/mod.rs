//! Plugins for the Information Alchemist application

pub mod agent_integration;
pub mod agent_ui;
pub mod nats_event_bridge;

pub use agent_integration::AgentIntegrationPlugin;
pub use agent_ui::AgentUiPlugin;
pub use nats_event_bridge::NatsEventBridgePlugin; 