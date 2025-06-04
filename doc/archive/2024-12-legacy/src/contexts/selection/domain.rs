use crate::contexts::graph_management::domain::{EdgeIdentity, NodeIdentity};
use bevy::prelude::*;
use std::collections::HashSet;

// ============= Selection Domain Entities =============

/// Represents the current selection state in the graph
#[derive(Resource, Default, Debug)]
pub struct SelectionState {
    pub selected_nodes: HashSet<NodeIdentity>,
    pub selected_edges: HashSet<EdgeIdentity>,
    pub selection_mode: SelectionMode,
    pub last_selected_node: Option<NodeIdentity>,
    pub last_selected_edge: Option<EdgeIdentity>,
}

impl SelectionState {
    /// Clear all selections
    pub fn clear(&mut self) {
        self.selected_nodes.clear();
        self.selected_edges.clear();
        self.last_selected_node = None;
        self.last_selected_edge = None;
    }

    /// Check if a node is selected
    pub fn is_node_selected(&self, node: &NodeIdentity) -> bool {
        self.selected_nodes.contains(node)
    }

    /// Check if an edge is selected
    pub fn is_edge_selected(&self, edge: &EdgeIdentity) -> bool {
        self.selected_edges.contains(edge)
    }

    /// Get the total number of selected items
    pub fn selection_count(&self) -> usize {
        self.selected_nodes.len() + self.selected_edges.len()
    }

    /// Add a node to selection
    pub fn select_node(&mut self, node: NodeIdentity) {
        self.selected_nodes.insert(node);
        self.last_selected_node = Some(node);
    }

    /// Remove a node from selection
    pub fn deselect_node(&mut self, node: &NodeIdentity) {
        self.selected_nodes.remove(node);
        if self.last_selected_node.as_ref() == Some(node) {
            self.last_selected_node = None;
        }
    }

    /// Add an edge to selection
    pub fn select_edge(&mut self, edge: EdgeIdentity) {
        self.selected_edges.insert(edge);
        self.last_selected_edge = Some(edge);
    }

    /// Remove an edge from selection
    pub fn deselect_edge(&mut self, edge: &EdgeIdentity) {
        self.selected_edges.remove(edge);
        if self.last_selected_edge.as_ref() == Some(edge) {
            self.last_selected_edge = None;
        }
    }
}

/// Different modes of selection behavior
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum SelectionMode {
    #[default]
    Single, // Only one item can be selected at a time
    Multiple, // Multiple items can be selected (Ctrl+Click)
    Box,      // Box selection mode
    Lasso,    // Lasso selection mode (future feature)
}

/// Component to mark entities as selectable
#[derive(Component, Default)]
pub struct Selectable;

/// Component to mark currently selected entities
#[derive(Component)]
pub struct Selected;

/// Component to store selection highlight information
#[derive(Component)]
pub struct SelectionHighlight {
    pub original_color: Color,
    pub highlight_color: Color,
    pub highlight_intensity: f32,
}

impl Default for SelectionHighlight {
    fn default() -> Self {
        Self {
            original_color: Color::WHITE,
            highlight_color: Color::srgb(1.0, 0.8, 0.2), // Golden
            highlight_intensity: 0.5,
        }
    }
}

/// Represents a selection box for box selection mode
#[derive(Component)]
pub struct SelectionBox {
    pub start: Vec2,
    pub end: Vec2,
    pub active: bool,
}

impl SelectionBox {
    pub fn new(start: Vec2) -> Self {
        Self {
            start,
            end: start,
            active: true,
        }
    }

    /// Get the normalized bounds of the selection box
    pub fn bounds(&self) -> (Vec2, Vec2) {
        let min_x = self.start.x.min(self.end.x);
        let max_x = self.start.x.max(self.end.x);
        let min_y = self.start.y.min(self.end.y);
        let max_y = self.start.y.max(self.end.y);

        (Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
    }

    /// Check if a screen position is within the selection box
    pub fn contains(&self, pos: Vec2) -> bool {
        let (min, max) = self.bounds();
        pos.x >= min.x && pos.x <= max.x && pos.y >= min.y && pos.y <= max.y
    }
}

/// Component to track hover state
#[derive(Component)]
pub struct Hovered;

/// Component to store original material for restoration
#[derive(Component)]
pub struct OriginalMaterial(pub Handle<StandardMaterial>);

/// Bundle for making entities selectable
#[derive(Bundle)]
pub struct SelectableBundle {
    pub selectable: Selectable,
    pub selection_highlight: SelectionHighlight,
}
