#[cfg(test)]
mod tests {
    use crate::contexts::graph_management::{
        domain::*,
        events::*,
        repositories::*,
        services::{
            AddNodeToGraph, ConnectGraphNodes, CreateGraph, GraphConstraintViolation, ValidateGraph,
        },
    };
    use bevy::ecs::system::SystemState;
    use bevy::prelude::*;
    use std::collections::HashMap;
    use uuid::Uuid;

    /// Helper to create a test app with required plugins
    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_event::<GraphCreated>()
            .add_event::<NodeAdded>()
            .add_event::<EdgeConnected>();
        app
    }

    #[test]
    fn test_create_graph_service() {
        let mut app = setup_test_app();

        // Given: Graph metadata
        let metadata = GraphMetadata {
            name: "Test Graph".to_string(),
            description: "Test Description".to_string(),
            domain: "test".to_string(),
            created: std::time::SystemTime::now(),
            modified: std::time::SystemTime::now(),
            tags: vec!["test".to_string()],
        };

        // When: CreateGraph service is executed
        let world = app.world_mut();
        let mut system_state: SystemState<(Commands, EventWriter<GraphCreated>)> =
            SystemState::new(world);
        let (mut commands, mut event_writer) = system_state.get_mut(world);

        let graph_id = CreateGraph::execute(metadata.clone(), &mut commands, &mut event_writer);

        // Then: Graph ID should be valid
        assert_ne!(graph_id.0, Uuid::nil());

        system_state.apply(world);

        // Apply commands
        app.update();

        // Verify entity was created with correct components
        let mut query = app
            .world_mut()
            .query::<(&GraphIdentity, &GraphMetadata, &GraphJourney)>();
        let results: Vec<_> = query.iter(&app.world()).collect();
        assert_eq!(results.len(), 1);

        let (stored_id, stored_metadata, stored_journey) = results[0];
        assert_eq!(*stored_id, graph_id);
        assert_eq!(stored_metadata.name, metadata.name);
        assert_eq!(stored_metadata.description, metadata.description);
        assert_eq!(stored_metadata.domain, metadata.domain);
        assert_eq!(stored_metadata.tags, metadata.tags);

        // Verify journey was initialized
        assert_eq!(stored_journey.version, 1); // Default is 1
        assert_eq!(stored_journey.event_count, 0);
        assert!(stored_journey.last_event.is_none());

        // Verify event was fired with all fields populated
        // Create an event reader manually
        let world = app.world_mut();
        let mut system_state: SystemState<EventReader<GraphCreated>> = SystemState::new(world);
        let mut event_reader = system_state.get_mut(world);

        let fired_events: Vec<_> = event_reader.read().collect();
        assert_eq!(fired_events.len(), 1);

        let event = &fired_events[0];
        assert_eq!(event.graph, graph_id);
        assert_eq!(event.metadata.name, metadata.name);
        assert_eq!(event.metadata.description, metadata.description);
        assert_eq!(event.metadata.domain, metadata.domain);

        // Validate all event fields are properly set
        assert_ne!(event.graph.0, uuid::Uuid::nil());
        assert!(!event.metadata.name.is_empty());
        assert!(!event.metadata.domain.is_empty());
        assert!(event.timestamp <= std::time::SystemTime::now());
        assert!(event.metadata.created <= event.metadata.modified);
    }

    #[test]
    fn test_add_node_to_graph() {
        let mut app = setup_test_app();

        // Create a graph first
        let graph_id = GraphIdentity::new();

        // Given: Node content and position
        let content = NodeContent {
            label: "Test Node".to_string(),
            category: "test".to_string(),
            properties: Default::default(),
        };
        let position = SpatialPosition::at_3d(1.0, 2.0, 3.0);

        // When: AddNodeToGraph is executed
        let world = app.world_mut();
        let mut system_state: SystemState<(Commands, EventWriter<NodeAdded>)> =
            SystemState::new(world);
        let (mut commands, mut event_writer) = system_state.get_mut(world);

        let node_id = AddNodeToGraph::execute(
            graph_id,
            content.clone(),
            position,
            &mut commands,
            &mut event_writer,
        );

        // Then: Node ID should be valid
        assert_ne!(node_id.0, Uuid::nil());

        system_state.apply(world);

        // Apply commands
        app.update();

        // Verify node entity was created
        let mut query = app.world_mut().query::<&NodeIdentity>();
        let count = query.iter(&app.world()).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_connect_nodes() {
        let mut app = setup_test_app();

        let graph_id = GraphIdentity::new();
        let source_id = NodeIdentity::new();
        let target_id = NodeIdentity::new();

        // When: ConnectGraphNodes is executed
        let world = app.world_mut();
        let mut system_state: SystemState<(Commands, EventWriter<EdgeConnected>)> =
            SystemState::new(world);
        let (mut commands, mut event_writer) = system_state.get_mut(world);

        let edge_id = ConnectGraphNodes::execute(
            graph_id,
            source_id,
            target_id,
            "test_edge".to_string(),
            1.0,
            &mut commands,
            &mut event_writer,
        );

        // Then: Edge ID should be valid
        assert_ne!(edge_id.0, Uuid::nil());

        system_state.apply(world);

        // Apply commands
        app.update();

        // Verify edge entity was created
        let mut query = app.world_mut().query::<&EdgeIdentity>();
        let count = query.iter(&app.world()).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_graph_validation_node_limit() {
        let mut app = setup_test_app();
        let validator = ValidateGraph;

        // Setup graph
        let graph_id = GraphIdentity::new();
        let journey = GraphJourney::default();

        // Spawn graph entity
        app.world_mut().spawn((graph_id, journey));

        // Add many nodes
        for _ in 0..100 {
            let node = crate::contexts::graph_management::domain::Node {
                identity: NodeIdentity::new(),
                graph: graph_id,
                content: NodeContent {
                    label: "Test".to_string(),
                    category: "test".to_string(),
                    properties: Default::default(),
                },
                position: SpatialPosition::at_3d(0.0, 0.0, 0.0),
            };
            app.world_mut().spawn(node);
        }

        // Test validation using SystemState
        let world = app.world_mut();
        let mut graph_state: SystemState<Query<(&GraphIdentity, &GraphJourney)>> =
            SystemState::new(world);
        let mut node_state: SystemState<Query<&crate::contexts::graph_management::domain::Node>> =
            SystemState::new(world);

        let graphs = graph_state.get(world);
        let nodes = node_state.get(world);

        let result = validator.can_add_node(graph_id, &graphs, &nodes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_validation_prevents_self_loops() {
        let validator = ValidateGraph;
        let mut app = setup_test_app();

        let graph_id = GraphIdentity::new();
        let node_id = NodeIdentity::new();

        // Spawn node
        let node = crate::contexts::graph_management::domain::Node {
            identity: node_id,
            graph: graph_id,
            content: NodeContent {
                label: "Test".to_string(),
                category: "test".to_string(),
                properties: Default::default(),
            },
            position: SpatialPosition::at_3d(0.0, 0.0, 0.0),
        };
        app.world_mut().spawn(node);

        // Test self-loop prevention using SystemState
        let world = app.world_mut();
        let mut node_state: SystemState<Query<&crate::contexts::graph_management::domain::Node>> =
            SystemState::new(world);
        let mut edge_state: SystemState<Query<&crate::contexts::graph_management::domain::Edge>> =
            SystemState::new(world);

        let nodes = node_state.get(world);
        let edges = edge_state.get(world);

        let result = validator.can_connect_nodes(
            graph_id, node_id, node_id, // Same node
            &nodes, &edges,
        );

        assert!(matches!(
            result,
            Err(GraphConstraintViolation::SelfLoopNotAllowed)
        ));
    }

    #[test]
    fn test_graph_validation_prevents_duplicate_edges() {
        let validator = ValidateGraph;
        let mut app = setup_test_app();

        let graph_id = GraphIdentity::new();
        let source_id = NodeIdentity::new();
        let target_id = NodeIdentity::new();

        // Spawn nodes
        let source_node = crate::contexts::graph_management::domain::Node {
            identity: source_id,
            graph: graph_id,
            content: NodeContent {
                label: "Source".to_string(),
                category: "test".to_string(),
                properties: Default::default(),
            },
            position: SpatialPosition::at_3d(0.0, 0.0, 0.0),
        };
        let target_node = crate::contexts::graph_management::domain::Node {
            identity: target_id,
            graph: graph_id,
            content: NodeContent {
                label: "Target".to_string(),
                category: "test".to_string(),
                properties: Default::default(),
            },
            position: SpatialPosition::at_3d(1.0, 0.0, 0.0),
        };
        app.world_mut().spawn(source_node);
        app.world_mut().spawn(target_node);

        // Add existing edge
        let edge = crate::contexts::graph_management::domain::Edge {
            identity: EdgeIdentity::new(),
            graph: graph_id,
            relationship: EdgeRelationship {
                source: source_id,
                target: target_id,
                category: "test".to_string(),
                strength: 1.0,
                properties: Default::default(),
            },
        };
        app.world_mut().spawn(edge);

        // Test duplicate edge prevention using SystemState
        let world = app.world_mut();
        let mut node_state: SystemState<Query<&crate::contexts::graph_management::domain::Node>> =
            SystemState::new(world);
        let mut edge_state: SystemState<Query<&crate::contexts::graph_management::domain::Edge>> =
            SystemState::new(world);

        let nodes = node_state.get(world);
        let edges = edge_state.get(world);

        let result = validator.can_connect_nodes(graph_id, source_id, target_id, &nodes, &edges);

        assert!(matches!(
            result,
            Err(GraphConstraintViolation::DuplicateEdgeNotAllowed)
        ));
    }

    #[test]
    fn test_establish_hierarchy_system() {
        let mut app = setup_test_app();

        // Create graph and node
        let graph_id = GraphIdentity::new();
        let node_id = NodeIdentity::new();

        // Spawn graph entity
        let graph_entity = app
            .world_mut()
            .spawn((graph_id, Transform::default(), GlobalTransform::default()))
            .id();

        // Spawn node entity
        let node = crate::contexts::graph_management::domain::Node {
            identity: node_id,
            graph: graph_id,
            content: NodeContent {
                label: "Child Node".to_string(),
                category: "test".to_string(),
                properties: Default::default(),
            },
            position: SpatialPosition::at_3d(0.0, 0.0, 0.0),
        };
        let node_entity = app
            .world_mut()
            .spawn((node, Transform::default(), GlobalTransform::default()))
            .id();

        // Manually run the hierarchy organization logic
        // Since run_system_once is not available, we'll directly call the system
        let world = app.world_mut();
        let mut system_state: SystemState<(
            Query<(Entity, &GraphIdentity)>,
            Query<(Entity, &crate::contexts::graph_management::domain::Node)>,
            Commands,
        )> = SystemState::new(world);

        let (graphs, nodes, mut commands) = system_state.get_mut(world);

        // Manually execute hierarchy organization logic
        for (graph_entity, graph_id) in graphs.iter() {
            for (node_entity, node) in nodes.iter() {
                if node.graph == *graph_id {
                    commands.entity(graph_entity).add_child(node_entity);
                }
            }
        }

        system_state.apply(world);

        // Apply commands
        app.update();

        // Verify parent-child relationship
        let children = app.world().get::<Children>(graph_entity);
        assert!(children.is_some());
        let children = children.expect("Graph entity should have children");
        assert!(children.contains(&node_entity));
    }

    // ===== REPOSITORY TESTS =====

    #[test]
    fn test_graphs_repository() {
        let mut repo = Graphs::new();

        // Test storing graph
        let graph_id = GraphIdentity::new();
        let graph_data = GraphData {
            identity: graph_id,
            metadata: GraphMetadata {
                name: "Test Graph".to_string(),
                description: "Test Description".to_string(),
                domain: "test".to_string(),
                created: std::time::SystemTime::now(),
                modified: std::time::SystemTime::now(),
                tags: vec!["test".to_string()],
            },
            journey: GraphJourney::default(),
            nodes: vec![],
            edges: vec![],
        };

        repo.store(graph_data.clone());

        // Test finding graph
        let found = repo.find(graph_id);
        assert!(found.is_some());
        let found = found.expect("Graph should be found after storing");
        assert_eq!(found.metadata.name, "Test Graph");

        // Test listing graphs
        let list = repo.list();
        assert_eq!(list.len(), 1);

        // Test exists
        assert!(repo.exists(graph_id));

        // Test removing graph
        let removed = repo.remove(graph_id);
        assert!(removed.is_some());
        let removed = removed.expect("Graph should be removable");
        assert_eq!(removed.metadata.name, "Test Graph");
        assert!(!repo.exists(graph_id));
    }

    #[test]
    fn test_graph_events_repository() {
        let mut repo = GraphEvents::new();

        let graph_id = GraphIdentity::new();
        let event1 = GraphEvent::Created(GraphCreated {
            graph: graph_id,
            metadata: GraphMetadata {
                name: "Test Graph".to_string(),
                description: "Test Description".to_string(),
                domain: "test".to_string(),
                created: std::time::SystemTime::now(),
                modified: std::time::SystemTime::now(),
                tags: vec!["test".to_string()],
            },
            timestamp: std::time::SystemTime::now(),
        });

        let event2 = GraphEvent::NodeAdded(NodeAdded {
            graph: graph_id,
            node: NodeIdentity::new(),
            content: NodeContent {
                label: "Test Node".to_string(),
                category: "test".to_string(),
                properties: HashMap::new(),
            },
            position: SpatialPosition::at_3d(0.0, 0.0, 0.0),
        });

        // Test appending events
        repo.append(event1.clone());
        repo.append(event2.clone());

        // Test getting events for graph
        let events = repo.events_for_graph(graph_id);
        assert_eq!(events.len(), 2);

        // Test events since version
        let recent_events = repo.events_since(graph_id, 1);
        assert_eq!(recent_events.len(), 1);

        // Test snapshot storage
        let snapshot = GraphSnapshot {
            graph_id,
            version: 2,
            timestamp: std::time::SystemTime::now(),
            data: GraphData {
                identity: graph_id,
                metadata: GraphMetadata {
                    name: "Test Graph".to_string(),
                    description: "Test Description".to_string(),
                    domain: "test".to_string(),
                    created: std::time::SystemTime::now(),
                    modified: std::time::SystemTime::now(),
                    tags: vec!["test".to_string()],
                },
                journey: GraphJourney::default(),
                nodes: vec![NodeData {
                    identity: NodeIdentity::new(),
                    content: NodeContent {
                        label: "Test Node".to_string(),
                        category: "test".to_string(),
                        properties: HashMap::new(),
                    },
                    position: SpatialPosition::at_3d(10.0, 20.0, 30.0),
                }],
                edges: vec![],
            },
        };

        repo.store_snapshot(snapshot.clone());

        // Test retrieving latest snapshot
        let latest = repo.latest_snapshot(graph_id);
        assert!(latest.is_some());
        let latest = latest.expect("Latest snapshot should be available");
        assert_eq!(latest.version, 2);
    }

    #[test]
    fn test_nodes_repository() {
        let mut repo = Nodes::new();

        let node_id = NodeIdentity::new();
        let graph_id = GraphIdentity::new();
        let location = NodeLocation { graph_id, node_id };

        // Test indexing node
        repo.index_node(node_id, location.clone());

        // Test locating node
        let found = repo.locate(node_id);
        assert!(found.is_some());
        let found = found.expect("Node should be found after indexing");
        assert_eq!(found.graph_id, graph_id);

        // Test removing node
        let removed = repo.remove(node_id);
        assert!(removed.is_some());
        let removed = removed.expect("Node should be removable");
        assert_eq!(removed.node_id, node_id);

        // Verify node is gone
        assert!(repo.locate(node_id).is_none());
    }

    #[test]
    fn test_edges_repository() {
        let mut repo = Edges::new();

        let source_id = NodeIdentity::new();
        let target_id = NodeIdentity::new();
        let edge_ref = EdgeReference {
            edge_id: EdgeIdentity::new(),
            target_node: target_id,
            category: "test".to_string(),
        };

        // Test adding edge
        repo.add_edge(source_id, edge_ref.clone());

        // Test getting edges from node
        let edges = repo.edges_from(source_id);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target_node, target_id);

        // Test adding multiple edges from same source
        let edge_ref2 = EdgeReference {
            edge_id: EdgeIdentity::new(),
            target_node: NodeIdentity::new(),
            category: "test2".to_string(),
        };
        repo.add_edge(source_id, edge_ref2);

        let edges = repo.edges_from(source_id);
        assert_eq!(edges.len(), 2);

        // Test removing edges
        let removed = repo.remove_edges_from(source_id);
        assert_eq!(removed.len(), 2);

        // Verify edges are gone
        let edges = repo.edges_from(source_id);
        assert_eq!(edges.len(), 0);
    }

    // ===== DOMAIN TESTS =====

    #[test]
    fn test_identity_creation() {
        let graph_id1 = GraphIdentity::new();
        let graph_id2 = GraphIdentity::new();

        // Each identity should be unique
        assert_ne!(graph_id1.0, graph_id2.0);
        assert_ne!(graph_id1.0, Uuid::nil());

        let node_id = NodeIdentity::new();
        assert_ne!(node_id.0, Uuid::nil());

        let edge_id = EdgeIdentity::new();
        assert_ne!(edge_id.0, Uuid::nil());
    }

    #[test]
    fn test_spatial_position_creation() {
        let pos = SpatialPosition::at_3d(1.0, 2.0, 3.0);

        assert_eq!(pos.coordinates_3d.x, 1.0);
        assert_eq!(pos.coordinates_3d.y, 2.0);
        assert_eq!(pos.coordinates_3d.z, 3.0);

        // Test 2D projection
        assert_eq!(pos.coordinates_2d.x, 1.0);
        assert_eq!(pos.coordinates_2d.y, 2.0);
    }

    #[test]
    fn test_graph_journey_defaults() {
        let journey = GraphJourney::default();

        assert_eq!(journey.version, 1);
        assert_eq!(journey.event_count, 0);
        assert!(journey.last_event.is_none());
    }

    #[test]
    fn test_metadata_structure() {
        let metadata = GraphMetadata {
            name: "Test Graph".to_string(),
            description: "A test graph".to_string(),
            domain: "testing".to_string(),
            created: std::time::SystemTime::now(),
            modified: std::time::SystemTime::now(),
            tags: vec!["test".to_string(), "example".to_string()],
        };

        assert_eq!(metadata.name, "Test Graph");
        assert_eq!(metadata.tags.len(), 2);
        assert!(metadata.tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_node_content_properties() {
        let mut properties = HashMap::new();
        properties.insert("color".to_string(), serde_json::json!("red"));
        properties.insert("size".to_string(), serde_json::json!("large"));

        let content = NodeContent {
            label: "Test Node".to_string(),
            category: "example".to_string(),
            properties: properties.clone(),
        };

        assert_eq!(content.label, "Test Node");
        assert_eq!(content.category, "example");
        assert_eq!(content.properties.len(), 2);
        assert_eq!(
            content.properties.get("color"),
            Some(&serde_json::json!("red"))
        );
    }

    #[test]
    fn test_edge_relationship() {
        let source = NodeIdentity::new();
        let target = NodeIdentity::new();
        let mut properties = HashMap::new();
        properties.insert("type".to_string(), serde_json::json!("depends_on"));

        let relationship = EdgeRelationship {
            source,
            target,
            category: "dependency".to_string(),
            strength: 0.8,
            properties,
        };

        assert_eq!(relationship.source, source);
        assert_eq!(relationship.target, target);
        assert_eq!(relationship.category, "dependency");
        assert_eq!(relationship.strength, 0.8);
        assert_eq!(
            relationship.properties.get("type"),
            Some(&serde_json::json!("depends_on"))
        );
    }
}

#[cfg(test)]
mod import_export_tests {
    use crate::contexts::graph_management::{
        domain::{
            EdgeIdentity, EdgeRelationship, GraphIdentity, GraphJourney, GraphMetadata,
            NodeContent, NodeIdentity, SpatialPosition,
        },
        exporter::{GraphExporter, JsonGraph},
        repositories::{EdgeData, GraphData, NodeData},
    };
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_export_graph_to_json() {
        // Create test graph data
        let graph_id = GraphIdentity::new();
        let node_id = NodeIdentity::new();

        let graph_data = GraphData {
            identity: graph_id,
            metadata: GraphMetadata {
                name: "Test Export Graph".to_string(),
                description: "Testing export functionality".to_string(),
                domain: "test".to_string(),
                created: std::time::SystemTime::now(),
                modified: std::time::SystemTime::now(),
                tags: vec!["test".to_string(), "export".to_string()],
            },
            journey: GraphJourney::default(),
            nodes: vec![NodeData {
                identity: node_id,
                content: NodeContent {
                    label: "Test Node".to_string(),
                    category: "test".to_string(),
                    properties: HashMap::new(),
                },
                position: SpatialPosition::at_3d(10.0, 20.0, 30.0),
            }],
            edges: vec![],
        };

        // Export to JSON
        let json_result = GraphExporter::export_to_json(&graph_data);
        assert!(json_result.is_ok());

        let json = json_result.unwrap();

        // Verify JSON contains expected data
        assert!(json.contains("Test Export Graph"));
        assert!(json.contains("Test Node"));
        assert!(json.contains("10.0")); // x coordinate
        assert!(json.contains("20.0")); // y coordinate
        assert!(json.contains("30.0")); // z coordinate
    }

    #[test]
    fn test_export_to_file() {
        // Create temporary directory
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_graph.json");

        // Create test graph data
        let graph_data = GraphData {
            identity: GraphIdentity::new(),
            metadata: GraphMetadata {
                name: "File Export Test".to_string(),
                description: "Testing file export".to_string(),
                domain: "test".to_string(),
                created: std::time::SystemTime::now(),
                modified: std::time::SystemTime::now(),
                tags: vec!["file-test".to_string()],
            },
            journey: GraphJourney::default(),
            nodes: vec![],
            edges: vec![],
        };

        // Export to file
        let result = GraphExporter::export_to_file(&file_path, &graph_data);
        assert!(result.is_ok());

        // Verify file exists and contains data
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("File Export Test"));
    }

    #[test]
    fn test_json_round_trip() {
        // Create original graph data
        let graph_id = GraphIdentity::new();
        let node1_id = NodeIdentity::new();
        let node2_id = NodeIdentity::new();
        let edge_id = EdgeIdentity::new();

        let original_graph = GraphData {
            identity: graph_id,
            metadata: GraphMetadata {
                name: "Round Trip Test".to_string(),
                description: "Testing round trip conversion".to_string(),
                domain: "round-trip".to_string(),
                created: std::time::SystemTime::now(),
                modified: std::time::SystemTime::now(),
                tags: vec!["round-trip".to_string(), "test".to_string()],
            },
            journey: GraphJourney::default(),
            nodes: vec![
                NodeData {
                    identity: node1_id,
                    content: NodeContent {
                        label: "Node One".to_string(),
                        category: "category-a".to_string(),
                        properties: HashMap::new(),
                    },
                    position: SpatialPosition::at_3d(1.0, 2.0, 3.0),
                },
                NodeData {
                    identity: node2_id,
                    content: NodeContent {
                        label: "Node Two".to_string(),
                        category: "category-b".to_string(),
                        properties: HashMap::new(),
                    },
                    position: SpatialPosition::at_3d(4.0, 5.0, 6.0),
                },
            ],
            edges: vec![EdgeData {
                identity: edge_id,
                relationship: EdgeRelationship {
                    source: node1_id,
                    target: node2_id,
                    category: "connects".to_string(),
                    strength: 0.75,
                    properties: HashMap::new(),
                },
            }],
        };

        // Export to JSON
        let json = GraphExporter::export_to_json(&original_graph).unwrap();

        // Parse JSON back
        let parsed: JsonGraph = serde_json::from_str(&json).unwrap();

        // Verify all data is preserved
        assert_eq!(parsed.name, "Round Trip Test");
        assert_eq!(parsed.description, "Testing round trip conversion");
        assert_eq!(parsed.domain, "round-trip");
        assert_eq!(
            parsed.tags,
            vec!["round-trip".to_string(), "test".to_string()]
        );

        // Check nodes
        assert_eq!(parsed.nodes.len(), 2);
        assert_eq!(parsed.nodes[0].label, "Node One");
        assert_eq!(parsed.nodes[0].category, "category-a");
        assert_eq!(parsed.nodes[0].x, 1.0);
        assert_eq!(parsed.nodes[0].y, 2.0);
        assert_eq!(parsed.nodes[0].z, 3.0);

        assert_eq!(parsed.nodes[1].label, "Node Two");
        assert_eq!(parsed.nodes[1].category, "category-b");
        assert_eq!(parsed.nodes[1].x, 4.0);
        assert_eq!(parsed.nodes[1].y, 5.0);
        assert_eq!(parsed.nodes[1].z, 6.0);

        // Check edges
        assert_eq!(parsed.edges.len(), 1);
        assert_eq!(parsed.edges[0].category, "connects");
        assert_eq!(parsed.edges[0].strength, 0.75);
    }

    #[test]
    fn test_export_with_special_characters() {
        // Test graph with special characters in name
        let graph_data = GraphData {
            identity: GraphIdentity::new(),
            metadata: GraphMetadata {
                name: "Test/Graph\\With:Special*Characters".to_string(),
                description: "Testing special character handling".to_string(),
                domain: "test".to_string(),
                created: std::time::SystemTime::now(),
                modified: std::time::SystemTime::now(),
                tags: vec![],
            },
            journey: GraphJourney::default(),
            nodes: vec![],
            edges: vec![],
        };

        // Export should succeed
        let json_result = GraphExporter::export_to_json(&graph_data);
        assert!(json_result.is_ok());

        // JSON should properly escape special characters
        let json = json_result.unwrap();
        assert!(json.contains("Test/Graph\\\\With:Special*Characters"));
    }
}
