//! Storage layer for graph persistence using Daggy
//!
//! This module provides persistent storage for graphs using the Daggy library,
//! enabling graph serialization, deserialization, and event replay capabilities.

use crate::contexts::graph_management::domain::*;
use crate::contexts::graph_management::events::*;
use bevy::prelude::*;
use daggy::{Dag, EdgeIndex, NodeIndex};
use std::collections::HashMap;

/// Node data stored in Daggy
#[derive(Clone, Debug)]
pub struct NodeData {
    pub identity: NodeIdentity,
    pub content: NodeContent,
    pub position: SpatialPosition,
}

/// Edge data stored in Daggy
#[derive(Clone, Debug)]
pub struct EdgeData {
    pub identity: EdgeIdentity,
    pub relationship: EdgeRelationship,
}

/// Primary graph storage using Daggy
#[derive(Resource, Default)]
pub struct GraphStorage {
    graphs: HashMap<GraphIdentity, Dag<NodeData, EdgeData>>,
    node_indices: HashMap<(GraphIdentity, NodeIdentity), NodeIndex>,
    edge_indices: HashMap<(GraphIdentity, EdgeIdentity), EdgeIndex>,
}

impl GraphStorage {
    /// Creates a new empty storage
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new graph in storage
    pub fn create_graph(&mut self, identity: GraphIdentity) -> Result<(), StorageError> {
        if self.graphs.contains_key(&identity) {
            return Err(StorageError::GraphAlreadyExists(identity));
        }

        self.graphs.insert(identity, Dag::new());
        Ok(())
    }

    /// Adds a node to a graph
    pub fn add_node(
        &mut self,
        graph_id: GraphIdentity,
        node_data: NodeData,
    ) -> Result<NodeIndex, StorageError> {
        let dag = self
            .graphs
            .get_mut(&graph_id)
            .ok_or(StorageError::GraphNotFound(graph_id))?;

        let node_identity = node_data.identity;
        let node_index = dag.add_node(node_data);

        self.node_indices
            .insert((graph_id, node_identity), node_index);
        Ok(node_index)
    }

    /// Adds an edge between nodes
    pub fn add_edge(
        &mut self,
        graph_id: GraphIdentity,
        source: NodeIdentity,
        target: NodeIdentity,
        edge_data: EdgeData,
    ) -> Result<EdgeIndex, StorageError> {
        let source_index = self
            .node_indices
            .get(&(graph_id, source))
            .ok_or(StorageError::NodeNotFound(source))?;

        let target_index = self
            .node_indices
            .get(&(graph_id, target))
            .ok_or(StorageError::NodeNotFound(target))?;

        let dag = self
            .graphs
            .get_mut(&graph_id)
            .ok_or(StorageError::GraphNotFound(graph_id))?;

        let edge_identity = edge_data.identity;
        let edge_index = dag
            .add_edge(*source_index, *target_index, edge_data)
            .map_err(|_| StorageError::CycleDetected)?;

        self.edge_indices
            .insert((graph_id, edge_identity), edge_index);
        Ok(edge_index)
    }

    /// Gets a graph from storage
    pub fn get_graph(&self, graph_id: GraphIdentity) -> Option<&Dag<NodeData, EdgeData>> {
        self.graphs.get(&graph_id)
    }

    /// Gets all nodes in a graph
    pub fn get_nodes(&self, graph_id: GraphIdentity) -> Vec<NodeData> {
        self.graphs
            .get(&graph_id)
            .map(|dag| {
                dag.raw_nodes()
                    .iter()
                    .map(|node| node.weight.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Gets all edges in a graph
    pub fn get_edges(
        &self,
        graph_id: GraphIdentity,
    ) -> Vec<(NodeIdentity, NodeIdentity, EdgeData)> {
        self.graphs
            .get(&graph_id)
            .map(|dag| {
                dag.raw_edges()
                    .iter()
                    .filter_map(|edge| {
                        let source_idx = NodeIndex::new(edge.source().index());
                        let target_idx = NodeIndex::new(edge.target().index());

                        let source_data = dag.node_weight(source_idx)?;
                        let target_data = dag.node_weight(target_idx)?;

                        Some((
                            source_data.identity,
                            target_data.identity,
                            edge.weight.clone(),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Removes a graph from storage
    pub fn remove_graph(&mut self, graph_id: GraphIdentity) -> Result<(), StorageError> {
        self.graphs
            .remove(&graph_id)
            .ok_or(StorageError::GraphNotFound(graph_id))?;

        // Clean up indices
        self.node_indices.retain(|(g, _), _| g != &graph_id);
        self.edge_indices.retain(|(g, _), _| g != &graph_id);

        Ok(())
    }

    /// Clear all graphs from storage
    pub fn clear(&mut self) {
        self.graphs.clear();
    }
}

/// Errors that can occur during storage operations
#[derive(Debug, Clone, PartialEq)]
pub enum StorageError {
    GraphNotFound(GraphIdentity),
    GraphAlreadyExists(GraphIdentity),
    NodeNotFound(NodeIdentity),
    EdgeNotFound(EdgeIdentity),
    CycleDetected,
}

/// Service for syncing ECS entities with Daggy storage
pub struct SyncGraphWithStorage;

impl SyncGraphWithStorage {
    /// Syncs graph creation events to storage
    pub fn sync_graph_created(
        mut storage: ResMut<GraphStorage>,
        mut events: EventReader<GraphCreated>,
    ) {
        for event in events.read() {
            if let Err(e) = storage.create_graph(event.graph) {
                warn!("Failed to sync graph creation: {:?}", e);
            }
        }
    }

    /// Syncs node addition events to storage
    pub fn sync_node_added(mut storage: ResMut<GraphStorage>, mut events: EventReader<NodeAdded>) {
        for event in events.read() {
            let node_data = NodeData {
                identity: event.node,
                content: event.content.clone(),
                position: event.position,
            };

            if let Err(e) = storage.add_node(event.graph, node_data) {
                warn!("Failed to sync node addition: {:?}", e);
            }
        }
    }

    /// Syncs edge connection events to storage
    pub fn sync_edge_connected(
        mut storage: ResMut<GraphStorage>,
        mut events: EventReader<EdgeConnected>,
    ) {
        for event in events.read() {
            let edge_data = EdgeData {
                identity: event.edge,
                relationship: event.relationship.clone(),
            };

            let result = storage.add_edge(
                event.graph,
                event.relationship.source,
                event.relationship.target,
                edge_data,
            );

            if let Err(e) = result {
                warn!("Failed to sync edge connection: {:?}", e);
            }
        }
    }

    /// Loads a graph from storage back into ECS
    pub fn load_from_storage(
        storage: &GraphStorage,
        graph_id: GraphIdentity,
        commands: &mut Commands,
        event_writer: &mut EventWriter<GraphCreated>,
    ) -> Result<Entity, StorageError> {
        // Verify graph exists
        let _dag = storage
            .get_graph(graph_id)
            .ok_or(StorageError::GraphNotFound(graph_id))?;

        // Create graph metadata
        let metadata = GraphMetadata {
            name: "Loaded Graph".to_string(),
            description: "Graph loaded from storage".to_string(),
            domain: "default".to_string(),
            created: std::time::SystemTime::now(),
            modified: std::time::SystemTime::now(),
            tags: vec![],
        };

        // Create graph entity
        let graph_entity = commands
            .spawn((
                GraphBundle {
                    graph: Graph {
                        identity: graph_id,
                        metadata: metadata.clone(),
                        journey: GraphJourney::default(),
                    },
                    identity: graph_id,
                    metadata: metadata.clone(),
                    journey: GraphJourney::default(),
                },
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        // Emit creation event
        event_writer.write(GraphCreated {
            graph: graph_id,
            metadata,
            timestamp: std::time::SystemTime::now(),
        });

        // Load nodes
        let nodes = storage.get_nodes(graph_id);
        let mut node_entities = HashMap::new();

        for node_data in nodes {
            let node_entity = commands
                .spawn((NodeBundle {
                    node: crate::contexts::graph_management::domain::Node {
                        identity: node_data.identity,
                        graph: graph_id,
                        content: node_data.content.clone(),
                        position: node_data.position,
                    },
                    identity: node_data.identity,
                    content: node_data.content,
                    position: node_data.position,
                    transform: Transform::from_translation(node_data.position.coordinates_3d),
                    global_transform: GlobalTransform::default(),
                },))
                .id();

            node_entities.insert(node_data.identity, node_entity);
        }

        // Load edges
        let edges = storage.get_edges(graph_id);

        for (source_id, target_id, edge_data) in edges {
            if let (Some(&_source_entity), Some(&_target_entity)) =
                (node_entities.get(&source_id), node_entities.get(&target_id))
            {
                commands.spawn((EdgeBundle {
                    edge: crate::contexts::graph_management::domain::Edge {
                        identity: edge_data.identity,
                        graph: graph_id,
                        relationship: edge_data.relationship.clone(),
                    },
                    identity: edge_data.identity,
                    relationship: edge_data.relationship,
                },));
            }
        }

        Ok(graph_entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_graph_identity() -> GraphIdentity {
        GraphIdentity(Uuid::new_v4())
    }

    fn create_test_node_data() -> NodeData {
        NodeData {
            identity: NodeIdentity(Uuid::new_v4()),
            content: NodeContent {
                label: "Test Node".to_string(),
                category: "test".to_string(),
                properties: HashMap::new(),
            },
            position: SpatialPosition::at_3d(0.0, 0.0, 0.0),
        }
    }

    fn create_test_edge_data(source: NodeIdentity, target: NodeIdentity) -> EdgeData {
        EdgeData {
            identity: EdgeIdentity(Uuid::new_v4()),
            relationship: EdgeRelationship {
                source,
                target,
                category: "hierarchy".to_string(),
                strength: 1.0,
                properties: HashMap::new(),
            },
        }
    }

    #[test]
    fn test_create_graph() {
        let mut storage = GraphStorage::new();
        let graph_id = create_test_graph_identity();

        // Should succeed on first creation
        assert!(storage.create_graph(graph_id).is_ok());

        // Should fail on duplicate
        assert_eq!(
            storage.create_graph(graph_id),
            Err(StorageError::GraphAlreadyExists(graph_id))
        );
    }

    #[test]
    fn test_add_node_to_graph() {
        let mut storage = GraphStorage::new();
        let graph_id = create_test_graph_identity();
        let node_data = create_test_node_data();

        // Should fail if graph doesn't exist
        assert!(matches!(
            storage.add_node(graph_id, node_data.clone()),
            Err(StorageError::GraphNotFound(_))
        ));

        // Create graph and add node
        storage.create_graph(graph_id).unwrap();
        let node_index = storage.add_node(graph_id, node_data.clone()).unwrap();

        // Verify node was added
        assert_eq!(node_index.index(), 0);
        let nodes = storage.get_nodes(graph_id);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].identity, node_data.identity);
    }

    #[test]
    fn test_add_edge_between_nodes() {
        let mut storage = GraphStorage::new();
        let graph_id = create_test_graph_identity();

        // Create graph and add two nodes
        storage.create_graph(graph_id).unwrap();

        let node1 = create_test_node_data();
        let node2 = create_test_node_data();
        let node1_id = node1.identity;
        let node2_id = node2.identity;

        storage.add_node(graph_id, node1).unwrap();
        storage.add_node(graph_id, node2).unwrap();

        // Add edge
        let edge_data = create_test_edge_data(node1_id, node2_id);
        let edge_index = storage
            .add_edge(graph_id, node1_id, node2_id, edge_data.clone())
            .unwrap();

        // Verify edge was added
        assert_eq!(edge_index.index(), 0);
        let edges = storage.get_edges(graph_id);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].0, node1_id);
        assert_eq!(edges[0].1, node2_id);
        assert_eq!(edges[0].2.identity, edge_data.identity);
    }

    #[test]
    fn test_edge_requires_existing_nodes() {
        let mut storage = GraphStorage::new();
        let graph_id = create_test_graph_identity();

        storage.create_graph(graph_id).unwrap();

        let node1 = create_test_node_data();
        let node2 = create_test_node_data();
        let edge_data = create_test_edge_data(node1.identity, node2.identity);

        // Should fail when source node doesn't exist
        assert!(matches!(
            storage.add_edge(graph_id, node1.identity, node2.identity, edge_data.clone()),
            Err(StorageError::NodeNotFound(_))
        ));

        // Add source node
        storage.add_node(graph_id, node1.clone()).unwrap();

        // Should fail when target node doesn't exist
        assert!(matches!(
            storage.add_edge(graph_id, node1.identity, node2.identity, edge_data),
            Err(StorageError::NodeNotFound(_))
        ));
    }

    #[test]
    fn test_remove_graph() {
        let mut storage = GraphStorage::new();
        let graph_id = create_test_graph_identity();

        // Create graph with nodes
        storage.create_graph(graph_id).unwrap();
        let node = create_test_node_data();
        storage.add_node(graph_id, node).unwrap();

        // Remove graph
        assert!(storage.remove_graph(graph_id).is_ok());

        // Verify graph and indices are cleaned up
        assert!(storage.get_graph(graph_id).is_none());
        assert!(storage.node_indices.is_empty());
        assert!(storage.edge_indices.is_empty());
    }

    #[test]
    fn test_sync_services() {
        use bevy::app::App;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<GraphCreated>();
        app.add_event::<NodeAdded>();
        app.add_event::<EdgeConnected>();
        app.insert_resource(GraphStorage::new());

        // Test graph creation sync
        let graph_id = create_test_graph_identity();

        let metadata = GraphMetadata {
            name: "Test Graph".to_string(),
            description: "Test".to_string(),
            domain: "test".to_string(),
            created: std::time::SystemTime::now(),
            modified: std::time::SystemTime::now(),
            tags: vec![],
        };

        app.world_mut().send_event(GraphCreated {
            graph: graph_id,
            metadata,
            timestamp: std::time::SystemTime::now(),
        });

        app.add_systems(Update, SyncGraphWithStorage::sync_graph_created);
        app.update();

        let storage = app.world().resource::<GraphStorage>();
        assert!(storage.get_graph(graph_id).is_some());
    }

    #[test]
    fn test_load_from_storage() {
        let mut storage = GraphStorage::new();
        let graph_id = create_test_graph_identity();

        // Set up storage with graph data
        storage.create_graph(graph_id).unwrap();

        let node1 = create_test_node_data();
        let node2 = create_test_node_data();
        let node1_id = node1.identity;
        let node2_id = node2.identity;
        storage.add_node(graph_id, node1).unwrap();
        storage.add_node(graph_id, node2).unwrap();

        let edge = create_test_edge_data(node1_id, node2_id);
        storage
            .add_edge(graph_id, node1_id, node2_id, edge)
            .unwrap();

        // Load from storage - basic test without full ECS setup
        let result = storage.get_graph(graph_id);
        assert!(result.is_some());

        let nodes = storage.get_nodes(graph_id);
        assert_eq!(nodes.len(), 2);

        let edges = storage.get_edges(graph_id);
        assert_eq!(edges.len(), 1);
    }
}
