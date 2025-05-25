// Components module - Pure data components with no logic
// Following the ECS principle: Components are just data

pub mod graph;
pub mod visual;
pub mod selection;
pub mod camera;
pub mod ui;
pub mod metadata;

// Re-export commonly used components
pub use graph::*;
pub use visual::*;
pub use selection::*;
pub use camera::*;
pub use ui::*;
pub use metadata::*;
