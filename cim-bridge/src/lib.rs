//! CIM Bridge - Standalone AI Provider Bridge
//! 
//! A UI-agnostic bridge service that connects various AI providers
//! (Ollama, OpenAI, Anthropic) through NATS messaging.

pub mod types;
pub mod providers;
pub mod services;
pub mod error;

pub use error::{BridgeError, Result};
pub use services::bridge::BridgeService;
pub use types::*; 