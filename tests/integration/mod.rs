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
pub mod graph_import_test;
pub mod graph_import_integration_test;
pub mod system_parameter_conflict_test;
pub mod bevy_system_conflict_test;
pub mod import_functionality_test;
pub mod distributed_event_store_tests;
pub mod simple_import_test;
pub mod visualization_test;

// Re-export common test utilities
pub use fixtures::*;
