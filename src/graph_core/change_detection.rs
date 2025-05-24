use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use petgraph::graph::{EdgeIndex as PetEdgeIndex, NodeIndex as PetNodeIndex};
use std::collections::HashSet;
use uuid::Uuid;

/// Tracks changes to the graph for efficient rendering updates
#[derive(Resource, Default)]
pub struct GraphChangeTracker {
    /// Nodes that have been modified this frame
    pub modified_nodes: HashSet<PetNodeIndex>,
    /// Edges that have been modified this frame
    pub modified_edges: HashSet<PetEdgeIndex>,
    /// Nodes that were added this frame
    pub added_nodes: HashSet<PetNodeIndex>,
    /// Edges that were added this frame
    pub added_edges: HashSet<PetEdgeIndex>,
    /// Nodes that were removed this frame
    pub removed_nodes: HashSet<Uuid>,
    /// Edges that were removed this frame
    pub removed_edges: HashSet<Uuid>,
    /// Whether the entire graph needs re-layout
    pub needs_full_layout: bool,
}

impl GraphChangeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark a node as modified
    pub fn mark_node_modified(&mut self, node_idx: PetNodeIndex) {
        self.modified_nodes.insert(node_idx);
    }

    /// Mark an edge as modified
    pub fn mark_edge_modified(&mut self, edge_idx: PetEdgeIndex) {
        self.modified_edges.insert(edge_idx);
    }

    /// Mark a node as added
    pub fn mark_node_added(&mut self, node_idx: PetNodeIndex) {
        self.added_nodes.insert(node_idx);
    }

    /// Mark an edge as added
    pub fn mark_edge_added(&mut self, edge_idx: PetEdgeIndex) {
        self.added_edges.insert(edge_idx);
    }

    /// Mark a node as removed
    pub fn mark_node_removed(&mut self, node_id: Uuid) {
        self.removed_nodes.insert(node_id);
    }

    /// Mark an edge as removed
    pub fn mark_edge_removed(&mut self, edge_id: Uuid) {
        self.removed_edges.insert(edge_id);
    }

    /// Request a full graph re-layout
    pub fn request_full_layout(&mut self) {
        self.needs_full_layout = true;
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.modified_nodes.is_empty()
            || !self.modified_edges.is_empty()
            || !self.added_nodes.is_empty()
            || !self.added_edges.is_empty()
            || !self.removed_nodes.is_empty()
            || !self.removed_edges.is_empty()
            || self.needs_full_layout
    }

    /// Clear all tracked changes
    pub fn clear(&mut self) {
        self.modified_nodes.clear();
        self.modified_edges.clear();
        self.added_nodes.clear();
        self.added_edges.clear();
        self.removed_nodes.clear();
        self.removed_edges.clear();
        self.needs_full_layout = false;
    }
}

/// System to process graph changes and update visual entities
pub fn process_graph_changes(
    _commands: Commands,
    mut change_tracker: ResMut<GraphChangeTracker>,
    graph_data: Res<super::GraphData>,
    mut node_query: Query<(&mut Transform, &super::components::GraphNode)>,
    mut edge_query: Query<
        (&mut Transform, &super::components::GraphEdge),
        Without<super::components::GraphNode>,
    >,
) {
    if !change_tracker.has_changes() {
        return;
    }

    // Process removed entities
    for node_id in &change_tracker.removed_nodes {
        // Entity removal handled by other systems
        debug!("Node {} was removed", node_id);
    }

    for edge_id in &change_tracker.removed_edges {
        // Entity removal handled by other systems
        debug!("Edge {} was removed", edge_id);
    }

    // Process modified nodes
    for &node_idx in &change_tracker.modified_nodes {
        if let Some(entity) = graph_data.get_node_entity(node_idx) {
            if let Ok((mut transform, _)) = node_query.get_mut(entity) {
                // Update position if changed
                if let Some(node_data) = graph_data.graph.node_weight(node_idx) {
                    transform.translation = node_data.position;
                }
            }
        }
    }

    // Process modified edges
    for &edge_idx in &change_tracker.modified_edges {
        if let Some(entity) = graph_data.get_edge_entity(edge_idx) {
            if let Ok((mut transform, _edge)) = edge_query.get_mut(entity) {
                // Recalculate edge position/rotation based on connected nodes
                if let Some((source_entity, target_entity)) = graph_data.get_edge_entities(edge_idx)
                {
                    if let (Ok((source_transform, _)), Ok((target_transform, _))) =
                        (node_query.get(source_entity), node_query.get(target_entity))
                    {
                        // Update edge transform
                        let start = source_transform.translation;
                        let end = target_transform.translation;
                        let midpoint = (start + end) / 2.0;
                        let direction = (end - start).normalize();

                        transform.translation = midpoint;
                        transform.rotation = Quat::from_rotation_arc(Vec3::Z, direction);
                        transform.scale = Vec3::new(0.02, 0.02, (end - start).length());
                    }
                }
            }
        }
    }

    // Process full layout if needed
    if change_tracker.needs_full_layout {
        info!("Full graph layout requested");
        // Trigger layout algorithm
        // This would integrate with a layout system
    }

    // Clear the tracker for next frame
    change_tracker.clear();
}

/// System to batch mesh updates for performance
pub fn batch_mesh_updates(
    change_tracker: Res<GraphChangeTracker>,
    _meshes: ResMut<Assets<Mesh>>,
    _query: Query<(&Mesh3d, &super::components::GraphNode)>,
) {
    if change_tracker.modified_nodes.is_empty() {
        return;
    }

    // Batch process mesh updates
    let modified_count = change_tracker.modified_nodes.len();
    if modified_count > 10 {
        debug!("Batching {} mesh updates", modified_count);
        // Implement batched mesh generation
    }
}

/// Component to track per-entity change state
#[derive(Component, Default)]
pub struct ChangeState {
    /// Frame when this entity was last modified
    pub last_modified_frame: u32,
    /// Whether this entity needs visual update
    pub needs_update: bool,
}

/// System to mark entities as changed based on component changes
pub fn detect_component_changes(
    mut param_set: ParamSet<(
        Query<
            (Entity, &mut ChangeState, Ref<Transform>),
            With<super::components::GraphNode>,
        >,
        Query<
            (Entity, &mut ChangeState, Ref<Transform>),
            With<super::components::GraphEdge>,
        >,
    )>,
    frame_count: Res<FrameCount>,
) {
    // Check nodes for changes
    for (entity, mut change_state, transform) in &mut param_set.p0() {
        if transform.is_changed() && !transform.is_added() {
            change_state.needs_update = true;
            change_state.last_modified_frame = frame_count.0;
            debug!("Node {:?} transform changed", entity);
        }
    }

    // Check edges for changes
    for (entity, mut change_state, transform) in &mut param_set.p1() {
        if transform.is_changed() && !transform.is_added() {
            change_state.needs_update = true;
            change_state.last_modified_frame = frame_count.0;
            debug!("Edge {:?} transform changed", entity);
        }
    }
}

/// Marker component for entities that need LOD updates
#[derive(Component)]
pub struct NeedsLodUpdate;

/// Level of detail settings based on camera distance
#[derive(Component)]
pub struct LodLevel {
    pub level: u8,
    pub distance: f32,
}

/// System to update LOD based on camera distance
pub fn update_lod_levels(
    camera_query: Query<&Transform, With<Camera>>,
    mut node_query: Query<(&Transform, &mut LodLevel, Entity), With<super::components::GraphNode>>,
    mut commands: Commands,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let camera_pos = camera_transform.translation;

    for (transform, mut lod, entity) in &mut node_query {
        let distance = camera_pos.distance(transform.translation);

        // Simple LOD levels
        let new_level = match distance {
            d if d < 10.0 => 0,  // High detail
            d if d < 50.0 => 1,  // Medium detail
            d if d < 100.0 => 2, // Low detail
            _ => 3,              // Very low detail / billboard
        };

        if new_level != lod.level {
            lod.level = new_level;
            lod.distance = distance;
            commands.entity(entity).insert(NeedsLodUpdate);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_tracker() {
        let mut tracker = GraphChangeTracker::new();

        assert!(!tracker.has_changes());

        tracker.mark_node_added(PetNodeIndex::new(0));
        assert!(tracker.has_changes());

        tracker.clear();
        assert!(!tracker.has_changes());
    }

    #[test]
    fn test_change_detection() {
        let mut tracker = GraphChangeTracker::new();

        tracker.mark_node_modified(PetNodeIndex::new(1));
        tracker.mark_edge_modified(PetEdgeIndex::new(0));

        assert!(tracker.modified_nodes.contains(&PetNodeIndex::new(1)));
        assert!(tracker.modified_edges.contains(&PetEdgeIndex::new(0)));
    }
}
