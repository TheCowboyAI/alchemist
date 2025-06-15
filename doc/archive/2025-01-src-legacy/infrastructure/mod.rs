//! Infrastructure layer
//!
//! External integrations and persistence

pub mod nats;

pub use nats::error::NatsError;
pub use nats::{NatsClient, NatsConfig};
