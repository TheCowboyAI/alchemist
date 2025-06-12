//! Visualization support for ContextGraphs

use crate::types::{NodeId, EdgeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Layout algorithms for graph visualization
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LayoutType {
    ForceDirected,
    Hierarchical,
    Circular,
    Grid,
    Random,
}

/// Visual style for nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeStyle {
    pub color: String,
    pub shape: String,
    pub size: f32,
}

/// Visual style for edges
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeStyle {
    pub color: String,
    pub width: f32,
    pub style: String, // solid, dashed, dotted
}

/// Interaction modes for graph manipulation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InteractionMode {
    Select,
    Pan,
    Zoom,
    AddNode,
    AddEdge,
    Delete,
}

/// Visualization hints for rendering
#[derive(Debug, Clone)]
pub struct VisualizationHints {
    pub layout_algorithm: LayoutType,
    pub node_styles: HashMap<NodeId, NodeStyle>,
    pub edge_styles: HashMap<EdgeId, EdgeStyle>,
    pub interaction_modes: Vec<InteractionMode>,
}
