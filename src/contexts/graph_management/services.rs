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

        let graph = Graph {
            identity,
            metadata: metadata.clone(),
            journey: journey.clone(),
        };

        // Spawn the graph entity
        commands.spawn(GraphBundle {
            graph,
            identity,
            metadata: metadata.clone(),
            journey,
        });

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

        let node = crate::contexts::graph_management::domain::Node {
            identity: node_id,
            graph: graph_id,
            content: content.clone(),
            position,
        };

        // Spawn the node entity
        commands.spawn(NodeBundle {
            node,
            identity: node_id,
            position,
            content: content.clone(),
            transform: Transform::from_translation(position.coordinates_3d),
            global_transform: GlobalTransform::default(),
        });

        // Emit the domain event
        events.write(NodeAdded {
            graph: graph_id,
            node: node_id,
            content,
            position,
        });

        node_id
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

        let edge = Edge {
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
    pub fn can_add_node(&self, _graph_id: GraphIdentity) -> Result<(), ValidationError> {
        // TODO: Implement domain rules
        Ok(())
    }

    pub fn can_connect_nodes(
        &self,
        _source: NodeIdentity,
        _target: NodeIdentity,
    ) -> Result<(), ValidationError> {
        // TODO: Implement domain rules (e.g., no self-loops, category constraints)
        Ok(())
    }
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidOperation(String),
    ConstraintViolation(String),
}
