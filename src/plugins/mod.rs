//! Plugins for the Information Alchemist application

pub mod agent_integration;
pub mod agent_ui;
pub mod camera_controller;
pub mod graph_editor;
pub mod nats_event_bridge;

pub use agent_integration::AgentIntegrationPlugin;
pub use agent_ui::AgentUiPlugin;
pub use camera_controller::CameraControllerPlugin;
pub use graph_editor::GraphEditorPlugin;
pub use nats_event_bridge::NatsEventBridgePlugin;
