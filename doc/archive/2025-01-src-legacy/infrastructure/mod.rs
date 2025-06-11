//! Infrastructure layer
//!
//! External integrations and persistence

pub mod nats;

pub use nats::{NatsClient, NatsConfig};
pub use nats::error::NatsError;
