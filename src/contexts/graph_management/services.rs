use crate::contexts::graph_management::{domain::*, events::*};
use bevy::prelude::*;
use std::collections::HashMap;

// ============= Domain Services =============
// Services are named as verb phrases that reveal their intent

/// Service to create new graphs
pub struct CreateGraph;

impl CreateGraph {
    /// Creates a new graph with the given metadata
    pub fn execute(
        metadata: GraphMetadata,
        commands: &mut Commands,
        events: &mut EventWriter<GraphCreated>,
    ) -> GraphIdentity {
        let identity = GraphIdentity::new();
        let journey = GraphJourney::default();

        // Create graph entity
        let graph = crate::contexts::graph_management::domain::Graph {
            identity,
            metadata: metadata.clone(),
            journey: journey.clone(),
        };

        // Spawn the graph entity with Transform for hierarchy
        commands.spawn((
            GraphBundle {
                graph,
                identity,
                metadata: metadata.clone(),
                journey,
            },
            Transform::default(),
            GlobalTransform::default(),
        ));

        // Emit the domain event
        events.write(GraphCreated {
            graph: identity,
            metadata,
            timestamp: std::time::SystemTime::now(),
        });

        identity
    }
}

/// Service to add nodes to a graph
pub struct AddNodeToGraph;

impl AddNodeToGraph {
    /// Adds a new node to the specified graph
    pub fn execute(
        graph_id: GraphIdentity,
        content: NodeContent,
        position: SpatialPosition,
        commands: &mut Commands,
        events: &mut EventWriter<NodeAdded>,
    ) -> NodeIdentity {
        let node_id = NodeIdentity::new();

        // Create the node entity with all fields
        let node = crate::contexts::graph_management::domain::Node {
            identity: node_id,
            graph: graph_id,
            content: content.clone(),
            position,
        };

        // Spawn the node with components
        commands.spawn(NodeBundle {
            node,
            identity: node_id,
            content: content.clone(),
            position,
            transform: Transform::from_translation(position.coordinates_3d),
            global_transform: GlobalTransform::default(),
        });

        // Emit event
        events.write(NodeAdded {
            graph: graph_id,
            node: node_id,
            content,
            position,
        });

        node_id
    }
}

/// Service to establish parent-child relationships in the scene graph
pub struct EstablishGraphHierarchy;

impl EstablishGraphHierarchy {
    /// System that establishes parent-child relationships between graphs and nodes
    pub fn organize_hierarchy(
        mut commands: Commands,
        graphs: Query<(Entity, &GraphIdentity)>,
        nodes: Query<(Entity, &crate::contexts::graph_management::domain::Node), Without<bevy::prelude::Parent>>,
    ) {
        // For each node, find its parent graph and establish relationship
        for (node_entity, node) in nodes.iter() {
            if let Some((graph_entity, _)) = graphs.iter().find(|(_, graph_id)| graph_id.0 == node.graph.0) {
                commands.entity(graph_entity).add_child(node_entity);
            }
        }
    }
}

/// Service to connect nodes with edges
pub struct ConnectGraphNodes;

impl ConnectGraphNodes {
    /// Creates an edge between two nodes
    pub fn execute(
        graph_id: GraphIdentity,
        source: NodeIdentity,
        target: NodeIdentity,
        category: String,
        strength: f32,
        commands: &mut Commands,
        events: &mut EventWriter<EdgeConnected>,
    ) -> EdgeIdentity {
        let edge_id = EdgeIdentity::new();

        let relationship = EdgeRelationship {
            source,
            target,
            category,
            strength,
            properties: HashMap::new(),
        };

        let edge = crate::contexts::graph_management::domain::Edge {
            identity: edge_id,
            graph: graph_id,
            relationship: relationship.clone(),
        };

        // Spawn the edge entity
        commands.spawn(EdgeBundle {
            edge,
            identity: edge_id,
            relationship: relationship.clone(),
        });

        // Emit the domain event
        events.write(EdgeConnected {
            graph: graph_id,
            edge: edge_id,
            relationship,
        });

        edge_id
    }
}

/// Service to validate graph operations
pub struct ValidateGraph;

impl ValidateGraph {
    /// Validates that an operation is allowed
    pub fn can_add_node(&self, _graph_id: GraphIdentity) -> Result<(), GraphConstraintViolation> {
        // TODO: Implement domain rules
        Ok(())
    }

    pub fn can_connect_nodes(
        &self,
        source: NodeIdentity,
        target: NodeIdentity,
    ) -> Result<(), GraphConstraintViolation> {
        // Check for self-referencing edges
        if source == target {
            return Err(GraphConstraintViolation::SelfReferencingEdge { node: source });
        }

        // TODO: Implement additional domain rules
        Ok(())
    }
}

/// Domain-specific constraint violations for graph operations
#[derive(Debug, Clone)]
pub enum GraphConstraintViolation {
    /// Attempted to create an edge from a node to itself
    SelfReferencingEdge { node: NodeIdentity },

    /// Node exists without any connections
    DisconnectedNode { node: NodeIdentity },

    /// Graph contains a cycle when acyclic graph is required
    CyclicDependency { path: Vec<NodeIdentity> },

    /// Node category doesn't allow the requested edge type
    InvalidEdgeCategory {
        source: NodeIdentity,
        target: NodeIdentity,
        category: String
    },

    /// Maximum node count exceeded for graph
    NodeLimitExceeded { limit: usize, current: usize },

    /// Maximum edge count exceeded for node
    EdgeLimitExceeded { node: NodeIdentity, limit: usize },
}
