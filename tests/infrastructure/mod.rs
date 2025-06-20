//! Infrastructure tests for CIM
//!
//! These tests validate the foundational components:
//! - NATS JetStream connectivity
//! - Event publishing and consumption
//! - Event persistence and replay
//! - CID chain integrity

pub mod test_nats_connection;
