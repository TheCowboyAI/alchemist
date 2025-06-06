//! Integration tests for CIM architecture
//!
//! These tests verify end-to-end functionality across all layers:
//! - Command submission through Bevy ECS
//! - Event processing through NATS
//! - Event storage in JetStream
//! - Projection updates
//! - Query handling

pub mod fixtures;
pub mod event_flow_tests;
pub mod nats_integration_tests;
pub mod cid_chain_tests;
pub mod projection_tests;
pub mod error_recovery_tests;
pub mod end_to_end_tests;

// Re-export common test utilities
pub use fixtures::*;
