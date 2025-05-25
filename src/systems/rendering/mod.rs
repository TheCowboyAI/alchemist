//! Rendering systems for visual representation
//!
//! These systems handle:
//! - Node mesh generation and updates
//! - Edge rendering and path updates
//! - Material and shader management
//! - Level of detail (LOD) optimization

pub mod node_rendering;
pub mod edge_rendering;
pub mod material_updates;
pub mod lod_system;

pub use node_rendering::*;
pub use edge_rendering::*;
pub use material_updates::*;
pub use lod_system::*;
