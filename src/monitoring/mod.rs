//! Event-based monitoring for CIM
//!
//! This module provides monitoring capabilities by subscribing to event streams
//! rather than using explicit logging.

pub mod event_monitor;

pub use event_monitor::run_event_monitor; 