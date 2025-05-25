// Bundles module - Component bundles for common patterns
// Bundles make it easy to spawn entities with common component combinations

pub mod node_bundle;
pub mod edge_bundle;

// Re-export commonly used bundles
pub use node_bundle::*;
pub use edge_bundle::*;
