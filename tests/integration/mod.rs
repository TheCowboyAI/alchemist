//! Integration tests for CIM architecture
//!
//! These tests verify end-to-end functionality across all layers:
//! - Command processing through aggregates
//! - Event persistence in NATS JetStream
//! - Projection updates from events
//! - Bevy ECS synchronization

pub mod cid_chain_tests;
pub mod domain_integration_tests;
pub mod end_to_end_workflow_tests;
pub mod event_flow_tests;
pub mod external_system_tests;
pub mod fixtures;
pub mod import_pipeline_tests;
pub mod nats_integration_tests;
pub mod performance_benchmarks;
pub mod projection_sync_tests;
pub mod query_handler_tests;
pub mod simple_test;

/// Common test configuration
pub struct TestConfig {
    pub nats_url: String,
    pub test_timeout: std::time::Duration,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            nats_url: "nats://localhost:4222".to_string(),
            test_timeout: std::time::Duration::from_secs(30),
        }
    }
}
