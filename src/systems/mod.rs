//! # Systems Module for Alchemist Graph Editor
//!
//! This module contains all ECS systems organized by functionality.
//! Each system has a single responsibility and communicates through events.
//!
//! ## System Categories
//!
//! - **Graph Systems**: Node/edge manipulation, selection, validation
//! - **Rendering Systems**: Visual representation and updates
//! - **Camera Systems**: View control and navigation
//! - **UI Systems**: User interface and interaction handling
//! - **I/O Systems**: File operations and data persistence
//!
//! ## Design Principles
//!
//! 1. **Single Responsibility**: Each system does one thing well
//! 2. **Event-Driven**: Systems communicate through events, not direct calls
//! 3. **Data Locality**: Systems only access components they need
//! 4. **Testability**: Systems can be tested in isolation
//! 5. **Performance**: Systems are optimized for their specific tasks

pub mod graph;
pub mod rendering;
pub mod camera;
pub mod ui;
pub mod io;

// Re-export commonly used systems
pub use graph::*;
pub use rendering::*;
pub use camera::*;
pub use ui::*;
pub use io::*;
