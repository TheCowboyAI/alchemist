pub mod algorithms;
pub mod change_detection;
pub mod components;
pub mod events;
pub mod graph_data;
pub mod merkle_dag;
pub mod plugin;
pub mod rendering;
pub mod systems;
pub mod ui;

pub use algorithms::{GraphAlgorithms, demonstrate_algorithms};
pub use change_detection::{GraphChangeTracker, detect_component_changes, process_graph_changes};
pub use components::*;
pub use events::*;
pub use graph_data::GraphData;
pub use merkle_dag::{MerkleDag, MerkleEdge, MerkleNode};
pub use plugin::GraphPlugin;
pub use ui::{
    GraphInspectorState, graph_inspector_ui, handle_node_selection, update_selection_highlights,
};
