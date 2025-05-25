// Resources module - Shared state (minimize these)
// Resources should be used sparingly for truly global state

pub mod graph_state;
pub mod panel_state;
pub mod file_state;
pub mod app_config;
pub mod graph_data;

// Re-export commonly used resources
pub use graph_state::*;
pub use panel_state::*;
pub use file_state::*;
pub use app_config::*;
pub use graph_data::*;
