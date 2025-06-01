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
        nodes: Query<(Entity, &crate::contexts::graph_management::domain::Node)>,
        parents: Query<&ChildOf>,
    ) {
        // For each node, find its parent graph and establish relationship
        for (node_entity, node) in nodes.iter() {
            // Skip if already has a parent
            if parents.get(node_entity).is_ok() {
                continue;
            }

            if let Some((graph_entity, _)) = graphs
                .iter()
                .find(|(_, graph_id)| graph_id.0 == node.graph.0)
            {
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
    /// Maximum nodes allowed per graph
    const MAX_NODES_PER_GRAPH: usize = 10_000;

    /// Maximum edges allowed per node
    const MAX_EDGES_PER_NODE: usize = 100;

    /// Validates that an operation is allowed
    pub fn can_add_node(
        &self,
        graph_id: GraphIdentity,
        graphs: &Query<(&GraphIdentity, &GraphJourney)>,
        nodes: &Query<&crate::contexts::graph_management::domain::Node>,
    ) -> Result<(), GraphConstraintViolation> {
        // Check if graph exists
        let graph_found = graphs.iter().any(|(id, _)| id.0 == graph_id.0);
        if !graph_found {
            return Err(GraphConstraintViolation::GraphNotFound);
        }

        // Note: Since GraphJourney doesn't have deleted_at, we'll skip deletion check for now
        // In a real system, you'd track deletion status separately

        // Check node count limits
        let current_node_count = nodes
            .iter()
            .filter(|node| node.graph.0 == graph_id.0)
            .count();

        if current_node_count >= Self::MAX_NODES_PER_GRAPH {
            return Err(GraphConstraintViolation::NodeLimitExceeded {
                limit: Self::MAX_NODES_PER_GRAPH,
                current: current_node_count,
            });
        }

        // Check if graph is locked (could be based on some metadata)
        // For now, we'll assume graphs are not locked

        Ok(())
    }

    pub fn can_connect_nodes(
        &self,
        graph: GraphIdentity,
        source: NodeIdentity,
        target: NodeIdentity,
        nodes: &Query<&crate::contexts::graph_management::domain::Node>,
        edges: &Query<&crate::contexts::graph_management::domain::Edge>,
    ) -> Result<(), GraphConstraintViolation> {
        // Check for self-referencing edges
        if source == target {
            return Err(GraphConstraintViolation::SelfLoopNotAllowed);
        }

        // Check if nodes exist and get their data
        let source_node = nodes.iter().find(|n| n.identity.0 == source.0);
        let target_node = nodes.iter().find(|n| n.identity.0 == target.0);

        if source_node.is_none() {
            return Err(GraphConstraintViolation::NodeNotFound(source));
        }
        if target_node.is_none() {
            return Err(GraphConstraintViolation::NodeNotFound(target));
        }

        // Check if nodes are in the same graph
        let source_graph = source_node.unwrap().graph;
        let target_graph = target_node.unwrap().graph;

        if source_graph != target_graph || source_graph != graph {
            return Err(GraphConstraintViolation::NodesInDifferentGraphs);
        }

        // Check edge count limits for source node
        let source_edge_count = edges
            .iter()
            .filter(|e| e.relationship.source.0 == source.0)
            .count();

        if source_edge_count >= Self::MAX_EDGES_PER_NODE {
            return Err(GraphConstraintViolation::EdgeLimitExceeded {
                node: source,
                limit: Self::MAX_EDGES_PER_NODE,
                current: source_edge_count,
            });
        }

        // Check for duplicate edges (optional)
        let duplicate_exists = edges
            .iter()
            .any(|e| e.relationship.source.0 == source.0 && e.relationship.target.0 == target.0);

        if duplicate_exists {
            return Err(GraphConstraintViolation::DuplicateEdgeNotAllowed);
        }

        Ok(())
    }
}

/// Domain-specific constraint violations for graph operations
#[derive(Debug, Clone)]
pub enum GraphConstraintViolation {
    /// Graph not found
    GraphNotFound,

    /// Graph has been deleted
    GraphDeleted,

    /// Node limit exceeded for graph
    NodeLimitExceeded { limit: usize, current: usize },

    /// Graph is locked for modifications
    GraphLocked,

    /// Node not found
    NodeNotFound(NodeIdentity),

    /// Nodes are in different graphs
    NodesInDifferentGraphs,

    /// Attempted to create an edge from a node to itself
    SelfReferencingEdge { node: NodeIdentity },

    /// Self-loops are not allowed
    SelfLoopNotAllowed,

    /// Edge limit exceeded for node
    EdgeLimitExceeded {
        node: NodeIdentity,
        limit: usize,
        current: usize,
    },

    /// Duplicate edge not allowed
    DuplicateEdgeNotAllowed,

    /// Node exists without any connections
    DisconnectedNode { node: NodeIdentity },

    /// Graph contains a cycle when acyclic graph is required
    CyclicDependency { path: Vec<NodeIdentity> },

    /// Node category doesn't allow the requested edge type
    InvalidEdgeCategory {
        source: NodeIdentity,
        target: NodeIdentity,
        category: String,
    },
}
