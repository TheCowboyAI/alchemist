//! Morphisms: The arrows (mappings) between objects in our categories
//!
//! This module defines the morphisms (structure-preserving mappings) between:
//! - Objects in the CIM-ContextGraph category (nodes, edges, graphs)
//! - Objects in the Bevy ECS category (entities, components, systems)

use bevy::prelude::*;
use cim_contextgraph::{NodeId, EdgeId, ContextGraphId as GraphId};
use crate::events::*;

/// Morphism from domain node operations to visual node operations
pub trait NodeMorphism {
    /// Map domain node creation to visual entity spawn
    fn create_visual(&self, commands: &mut Commands, node_id: NodeId, graph_id: GraphId, position: Vec3) -> Entity;

    /// Map domain node deletion to visual entity despawn
    fn delete_visual(&self, commands: &mut Commands, entity: Entity);

    /// Map domain node update to visual component update
    fn update_visual(&self, commands: &mut Commands, entity: Entity, update: NodeUpdate);
}

/// Morphism from domain edge operations to visual edge operations
pub trait EdgeMorphism {
    /// Map domain edge creation to visual line/curve creation
    fn create_visual(&self, commands: &mut Commands, edge_id: EdgeId, source: Entity, target: Entity) -> Entity;

    /// Map domain edge deletion to visual removal
    fn delete_visual(&self, commands: &mut Commands, entity: Entity);

    /// Map domain edge update to visual update
    fn update_visual(&self, commands: &mut Commands, entity: Entity, update: EdgeUpdate);
}

/// Morphism from visual interactions to domain events
pub trait InteractionMorphism {
    /// Map mouse click to domain selection event
    fn map_click(&self, world_pos: Vec3, entity: Entity) -> SelectionChanged;

    /// Map drag operation to domain position update
    fn map_drag(&self, entity: Entity, delta: Vec3) -> NodePositionChanged;

    /// Map keyboard input to domain command
    fn map_keyboard(&self, key: KeyCode, modifiers: Modifiers) -> Option<DomainCommand>;
}

/// Composition of morphisms
pub struct MorphismComposition;

impl MorphismComposition {
    /// Compose two morphisms: (g ∘ f)(x) = g(f(x))
    pub fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
    where
        F: Fn(A) -> B,
        G: Fn(B) -> C,
    {
        move |x| g(f(x))
    }
}

/// Identity morphism (preserves structure exactly)
pub struct IdentityMorphism;

impl IdentityMorphism {
    pub fn map<T>(x: T) -> T {
        x
    }
}

/// Isomorphism verification
pub struct IsomorphismVerifier;

impl IsomorphismVerifier {
    /// Verify that F ∘ G = Id and G ∘ F = Id
    pub fn verify_isomorphism<A, B, F, G>(f: F, g: G, a: A, b: B) -> bool
    where
        A: PartialEq + Clone,
        B: PartialEq + Clone,
        F: Fn(A) -> B,
        G: Fn(B) -> A,
    {
        let a_clone = a.clone();
        let b_clone = b.clone();

        // Check F ∘ G = Id_B
        let b_result = f(g(b_clone));
        let b_preserved = b_result == b;

        // Check G ∘ F = Id_A
        let a_result = g(f(a_clone));
        let a_preserved = a_result == a;

        a_preserved && b_preserved
    }
}

/// Concrete implementations

pub struct StandardNodeMorphism;

impl NodeMorphism for StandardNodeMorphism {
    fn create_visual(&self, commands: &mut Commands, node_id: NodeId, graph_id: GraphId, position: Vec3) -> Entity {
        commands.spawn(crate::components::NodeVisualBundle::new(node_id, graph_id, position)).id()
    }

    fn delete_visual(&self, commands: &mut Commands, entity: Entity) {
        commands.entity(entity).despawn_recursive();
    }

    fn update_visual(&self, commands: &mut Commands, entity: Entity, update: NodeUpdate) {
        match update {
            NodeUpdate::Position(pos) => {
                commands.entity(entity).insert(Transform::from_translation(pos));
            }
            NodeUpdate::Selected(selected) => {
                if selected {
                    commands.entity(entity).insert(crate::components::Selected);
                } else {
                    commands.entity(entity).remove::<crate::components::Selected>();
                }
            }
        }
    }
}

/// Helper types for morphism parameters
#[derive(Debug, Clone)]
pub enum NodeUpdate {
    Position(Vec3),
    Selected(bool),
}

#[derive(Debug, Clone)]
pub enum EdgeUpdate {
    Highlighted(bool),
    Weight(f32),
}

#[derive(Debug, Clone)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

#[derive(Debug, Clone)]
pub enum DomainCommand {
    CreateNode { position: Vec3 },
    DeleteSelected,
    ConnectSelected,
    LayoutGraph,
}

/// System functions for morphism operations

/// System to create node visuals from events
pub fn create_node_visual(
    mut commands: Commands,
    mut events: EventReader<CreateNodeVisual>,
) {
    for event in events.read() {
        commands.spawn(crate::components::NodeVisualBundle::new(
            event.node_id,
            event.graph_id,
            event.position,
        ));
    }
}

/// System to remove node visuals from events
pub fn remove_node_visual(
    mut commands: Commands,
    mut events: EventReader<RemoveNodeVisual>,
    query: Query<(Entity, &crate::components::NodeVisual)>,
) {
    for event in events.read() {
        for (entity, node_visual) in query.iter() {
            if node_visual.node_id == event.node_id {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// System to create edge visuals from events
pub fn create_edge_visual(
    mut commands: Commands,
    mut events: EventReader<CreateEdgeVisual>,
    nodes: Query<(Entity, &crate::components::NodeVisual)>,
) {
    for event in events.read() {
        // Find source and target entities
        let mut source_entity = None;
        let mut target_entity = None;

        for (entity, node_visual) in nodes.iter() {
            if node_visual.node_id == event.source_id {
                source_entity = Some(entity);
            }
            if node_visual.node_id == event.target_id {
                target_entity = Some(entity);
            }
        }

        if let (Some(source), Some(target)) = (source_entity, target_entity) {
            commands.spawn(crate::components::EdgeVisualBundle::new(
                event.edge_id,
                event.graph_id,
                source,
                target,
            ));
        }
    }
}

/// System to remove edge visuals from events
pub fn remove_edge_visual(
    mut commands: Commands,
    mut events: EventReader<RemoveEdgeVisual>,
    query: Query<(Entity, &crate::components::EdgeVisual)>,
) {
    for event in events.read() {
        for (entity, edge_visual) in query.iter() {
            if edge_visual.edge_id == event.edge_id {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// System to update node positions
pub fn update_node_position(
    mut events: EventReader<NodePositionChanged>,
    mut query: Query<(&crate::components::NodeVisual, &mut Transform)>,
) {
    for event in events.read() {
        for (node_visual, mut transform) in query.iter_mut() {
            if node_visual.node_id == event.node_id {
                transform.translation = event.new_position;
            }
        }
    }
}

/// System to update node metadata
pub fn update_node_metadata(
    mut events: EventReader<NodeMetadataChanged>,
    mut query: Query<&mut crate::components::NodeVisual>,
) {
    for event in events.read() {
        for mut node_visual in query.iter_mut() {
            if node_visual.node_id == event.node_id {
                // Update metadata (this would be expanded based on actual metadata structure)
                // For now, we just acknowledge the event
            }
        }
    }
}

/// System to update edge metadata
pub fn update_edge_metadata(
    mut events: EventReader<EdgeMetadataChanged>,
    mut query: Query<&mut crate::components::EdgeVisual>,
) {
    for event in events.read() {
        for mut edge_visual in query.iter_mut() {
            if edge_visual.edge_id == event.edge_id {
                // Update metadata (this would be expanded based on actual metadata structure)
                // For now, we just acknowledge the event
            }
        }
    }
}
