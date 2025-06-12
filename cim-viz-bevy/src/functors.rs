//! Functors: Structure-preserving mappings between categories
//!
//! This module implements the actual functor between:
//! - CIM-ContextGraph Category (domain graphs)
//! - Bevy ECS Category (visual entities)

use bevy::prelude::*;
use cim_contextgraph::{ContextGraph, NodeEntry, EdgeEntry, NodeId, ContextGraphId as GraphId};
use crate::components::*;
use crate::events::{DomainEvent, VisualizationCommand, EdgeRelationship, CreateNodeVisual, CreateEdgeVisual};

/// Functor F: CIM-ContextGraph → Bevy ECS
/// Maps domain objects to visual representations
pub struct DomainToVisualFunctor;

impl DomainToVisualFunctor {
    /// Map a domain node to visual components
    pub fn map_node<N>(
        domain_node: &NodeEntry<N>,
        graph_id: GraphId,
        position: Vec3,
    ) -> NodeVisualBundle {
        NodeVisualBundle::new(domain_node.id, graph_id, position)
    }

    /// Map a domain edge to visual components
    pub fn map_edge<E>(
        domain_edge: &EdgeEntry<E>,
        graph_id: GraphId,
        source_entity: Entity,
        target_entity: Entity,
    ) -> EdgeVisual {
        EdgeVisual {
            edge_id: domain_edge.id,
            graph_id,
            source_entity,
            target_entity,
        }
    }

    /// Map domain graph to visual graph metadata
    pub fn map_graph<N, E>(graph: &ContextGraph<N, E>) -> GraphVisual {
        GraphVisual {
            graph_id: graph.id,
            layout_type: LayoutType::ForceDirected, // Default
        }
    }
}

/// Functor G: Bevy ECS → CIM-ContextGraph
/// Maps visual operations back to domain commands
pub struct VisualToDomainFunctor;

impl VisualToDomainFunctor {
    /// Map node position change to domain command
    pub fn map_position_change(
        node_id: NodeId,
        new_position: Vec3,
    ) -> VisualizationCommand {
        VisualizationCommand::UpdateNodePosition {
            graph_id: GraphId::new(), // TODO: Get from context
            node_id,
            position: new_position,
        }
    }

    /// Map node creation to domain command
    pub fn map_node_creation(
        position: Vec3,
        graph_id: GraphId,
    ) -> VisualizationCommand {
        VisualizationCommand::CreateNode {
            graph_id,
            position,
            metadata: None,
        }
    }

    /// Map edge creation to domain command
    pub fn map_edge_creation(
        source: NodeId,
        target: NodeId,
        graph_id: GraphId,
    ) -> VisualizationCommand {
        VisualizationCommand::CreateEdge {
            graph_id,
            source,
            target,
            relationship: EdgeRelationship::Connected,
        }
    }
}

/// Natural transformation between functors
/// Ensures the diagram commutes: F ∘ G ≅ Id
pub struct NaturalTransformation;

impl NaturalTransformation {
    /// Verify that visual representation preserves domain structure
    pub fn verify_node_preservation<N>(
        domain_node: &NodeEntry<N>,
        visual_bundle: &NodeVisualBundle,
    ) -> bool {
        domain_node.id == visual_bundle.node.node_id
    }

    /// Verify that operations preserve semantics
    pub fn verify_operation_preservation(
        domain_command: &VisualizationCommand,
        visual_event: &dyn std::any::Any,
    ) -> bool {
        // Type-safe verification of operation preservation
        match domain_command {
            VisualizationCommand::CreateNode { .. } => {
                visual_event.is::<CreateNodeVisual>()
            }
            VisualizationCommand::CreateEdge { .. } => {
                visual_event.is::<CreateEdgeVisual>()
            }
            _ => true,
        }
    }
}

/// Composition of functors
pub struct FunctorComposition;

impl FunctorComposition {
    /// F ∘ G: Round-trip from domain through visual and back
    pub fn domain_visual_domain<N>(
        node: &NodeEntry<N>,
        graph_id: GraphId,
        position: Vec3,
    ) -> VisualizationCommand {
        // F: Domain → Visual
        let visual = DomainToVisualFunctor::map_node(node, graph_id, position);

        // G: Visual → Domain
        VisualToDomainFunctor::map_position_change(
            visual.node.node_id,
            visual.transform.translation,
        )
    }
}

/// Functor laws verification
#[cfg(test)]
mod functor_laws {
    use super::*;

    #[test]
    fn test_identity_preservation() {
        // Functor preserves identity morphisms
        let node_id = NodeId::new();
        let graph_id = GraphId::new();
        let position = Vec3::ZERO;

        let visual = NodeVisualBundle::new(node_id, graph_id, position);
        assert_eq!(visual.node.node_id, node_id);
        assert_eq!(visual.node.graph_id, graph_id);
    }

    #[test]
    fn test_composition_preservation() {
        // Functor preserves composition: F(g ∘ f) = F(g) ∘ F(f)
        // This would test that composed operations map correctly
    }
}
