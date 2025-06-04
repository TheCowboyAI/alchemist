//! Repository Integration Tests
//! Tests that verify repositories properly persist data and handle events

use crate::contexts::graph_management::{domain::*, events::*, repositories::*};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_repository_persistence() {
        let mut repo = Graphs::new();

        // Create test data
        let graph_id = GraphIdentity::new();
        let metadata = GraphMetadata {
            name: "Integration Test Graph".to_string(),
            description: "Testing full persistence".to_string(),
            domain: "test-domain".to_string(),
            created: std::time::SystemTime::now(),
            modified: std::time::SystemTime::now(),
            tags: vec!["test".to_string(), "integration".to_string()],
        };

        let graph_data = GraphData {
            identity: graph_id,
            metadata: metadata.clone(),
            journey: GraphJourney::default(),
            nodes: vec![],
            edges: vec![],
        };

        // Test store
        repo.store(graph_data.clone());
        assert_eq!(repo.count(), 1);

        // Test find
        let found = repo.find(graph_id).expect("Graph should be found");
        assert_eq!(found.identity, graph_id);
        assert_eq!(found.metadata.name, metadata.name);
        assert_eq!(found.metadata.domain, metadata.domain);
        assert_eq!(found.metadata.tags.len(), 2);

        // Test find_by_domain
        let domain_graphs = repo.find_by_domain("test-domain");
        assert_eq!(domain_graphs.len(), 1);
        assert_eq!(domain_graphs[0].identity, graph_id);

        // Test update_metadata
        let new_metadata = GraphMetadata {
            name: "Updated Graph Name".to_string(),
            description: "Updated description".to_string(),
            ..metadata
        };
        assert!(repo.update_metadata(graph_id, new_metadata.clone()));

        let updated = repo.find(graph_id).expect("Graph should still exist");
        assert_eq!(updated.metadata.name, "Updated Graph Name");
        assert_eq!(updated.metadata.description, "Updated description");

        // Test exists
        assert!(repo.exists(graph_id));

        // Test remove
        let removed = repo.remove(graph_id).expect("Should remove graph");
        assert_eq!(removed.identity, graph_id);
        assert!(!repo.exists(graph_id));
        assert_eq!(repo.count(), 0);
    }

    #[test]
    fn test_event_sourcing_with_snapshots() {
        let mut event_store = GraphEvents::new();
        let graph_id = GraphIdentity::new();

        // Create a series of events
        let created_event = GraphEvent::Created(GraphCreated {
            graph: graph_id,
            metadata: GraphMetadata {
                name: "Event Sourced Graph".to_string(),
                description: "Testing event sourcing".to_string(),
                domain: "events".to_string(),
                created: std::time::SystemTime::now(),
                modified: std::time::SystemTime::now(),
                tags: vec!["event-sourced".to_string()],
            },
            timestamp: std::time::SystemTime::now(),
        });

        let node1_id = NodeIdentity::new();
        let node_added_event = GraphEvent::NodeAdded(NodeAdded {
            graph: graph_id,
            node: node1_id,
            content: NodeContent {
                label: "Node 1".to_string(),
                category: "test".to_string(),
                properties: HashMap::from([("key".to_string(), serde_json::json!("value"))]),
            },
            position: SpatialPosition::at_3d(10.0, 20.0, 0.0),
        });

        // Append events
        event_store.append(created_event);
        event_store.append(node_added_event);
        assert_eq!(event_store.total_events(), 2);

        // Test events_for_graph
        let graph_events = event_store.events_for_graph(graph_id);
        assert_eq!(graph_events.len(), 2);

        // Verify event order and content
        match &graph_events[0] {
            GraphEvent::Created(e) => {
                assert_eq!(e.graph, graph_id);
                assert_eq!(e.metadata.name, "Event Sourced Graph");
            }
            _ => panic!("First event should be Created"),
        }

        match &graph_events[1] {
            GraphEvent::NodeAdded(e) => {
                assert_eq!(e.graph, graph_id);
                assert_eq!(e.node, node1_id);
                assert_eq!(e.content.label, "Node 1");
                assert_eq!(e.position.coordinates_3d.x, 10.0);
            }
            _ => panic!("Second event should be NodeAdded"),
        }

        // Test snapshot storage
        let snapshot = GraphSnapshot {
            graph_id,
            version: 2,
            timestamp: std::time::SystemTime::now(),
            data: GraphData {
                identity: graph_id,
                metadata: GraphMetadata {
                    name: "Event Sourced Graph".to_string(),
                    description: "After snapshot".to_string(),
                    domain: "events".to_string(),
                    created: std::time::SystemTime::now(),
                    modified: std::time::SystemTime::now(),
                    tags: vec!["snapshot".to_string()],
                },
                journey: GraphJourney {
                    version: 2,
                    event_count: 2,
                    last_event: Some(uuid::Uuid::new_v4()),
                },
                nodes: vec![NodeData {
                    identity: node1_id,
                    content: NodeContent {
                        label: "Node 1".to_string(),
                        category: "test".to_string(),
                        properties: HashMap::new(),
                    },
                    position: SpatialPosition::at_3d(10.0, 20.0, 0.0),
                }],
                edges: vec![],
            },
        };

        event_store.store_snapshot(snapshot);

        // Verify snapshot retrieval
        let latest = event_store
            .latest_snapshot(graph_id)
            .expect("Should have snapshot");
        assert_eq!(latest.version, 2);
        assert_eq!(latest.data.nodes.len(), 1);
        assert_eq!(latest.data.journey.event_count, 2);

        // Test events_since
        let recent_events = event_store.events_since(graph_id, 1);
        assert_eq!(recent_events.len(), 1);
        match recent_events[0] {
            GraphEvent::NodeAdded(_) => {}
            _ => panic!("Event after version 1 should be NodeAdded"),
        }
    }

    #[test]
    fn test_node_index_operations() {
        let mut node_index = Nodes::new();

        let graph1 = GraphIdentity::new();
        let graph2 = GraphIdentity::new();
        let node1 = NodeIdentity::new();
        let node2 = NodeIdentity::new();
        let node3 = NodeIdentity::new();

        // Index nodes
        node_index.index_node(
            node1,
            NodeLocation {
                graph_id: graph1,
                node_id: node1,
            },
        );
        node_index.index_node(
            node2,
            NodeLocation {
                graph_id: graph1,
                node_id: node2,
            },
        );
        node_index.index_node(
            node3,
            NodeLocation {
                graph_id: graph2,
                node_id: node3,
            },
        );

        assert_eq!(node_index.count(), 3);

        // Test locate
        let loc1 = node_index.locate(node1).expect("Node1 should be indexed");
        assert_eq!(loc1.graph_id, graph1);
        assert_eq!(loc1.node_id, node1);

        // Test contains
        assert!(node_index.contains(node1));
        assert!(node_index.contains(node2));
        assert!(node_index.contains(node3));

        // Test nodes_in_graph
        let graph1_nodes = node_index.nodes_in_graph(graph1);
        assert_eq!(graph1_nodes.len(), 2);
        assert!(graph1_nodes.contains(&node1));
        assert!(graph1_nodes.contains(&node2));

        let graph2_nodes = node_index.nodes_in_graph(graph2);
        assert_eq!(graph2_nodes.len(), 1);
        assert!(graph2_nodes.contains(&node3));

        // Test remove
        let removed = node_index.remove(node1).expect("Should remove node1");
        assert_eq!(removed.node_id, node1);
        assert!(!node_index.contains(node1));
        assert_eq!(node_index.count(), 2);
    }

    #[test]
    fn test_edge_adjacency_operations() {
        let mut edge_index = Edges::new();

        let node1 = NodeIdentity::new();
        let node2 = NodeIdentity::new();
        let node3 = NodeIdentity::new();
        let edge1 = EdgeIdentity::new();
        let edge2 = EdgeIdentity::new();

        // Add edges
        edge_index.add_edge(
            node1,
            EdgeReference {
                edge_id: edge1,
                target_node: node2,
                category: "depends_on".to_string(),
            },
        );

        edge_index.add_edge(
            node1,
            EdgeReference {
                edge_id: edge2,
                target_node: node3,
                category: "relates_to".to_string(),
            },
        );

        // Test out_degree
        assert_eq!(edge_index.out_degree(node1), 2);
        assert_eq!(edge_index.out_degree(node2), 0);

        // Test edges_from
        let edges_from_node1 = edge_index.edges_from(node1);
        assert_eq!(edges_from_node1.len(), 2);

        // Test has_edge
        assert!(edge_index.has_edge(node1, node2));
        assert!(edge_index.has_edge(node1, node3));
        assert!(!edge_index.has_edge(node2, node1));

        // Test edges_by_category
        let depends_edges = edge_index.edges_by_category(node1, "depends_on");
        assert_eq!(depends_edges.len(), 1);
        assert_eq!(depends_edges[0].target_node, node2);

        let relates_edges = edge_index.edges_by_category(node1, "relates_to");
        assert_eq!(relates_edges.len(), 1);
        assert_eq!(relates_edges[0].target_node, node3);

        // Test source_nodes
        let sources = edge_index.source_nodes();
        assert_eq!(sources.len(), 1);
        assert!(sources.contains(&node1));

        // Test remove_edges_from
        let removed = edge_index.remove_edges_from(node1);
        assert_eq!(removed.len(), 2);
        assert_eq!(edge_index.out_degree(node1), 0);
        assert!(!edge_index.has_edge(node1, node2));
    }

    #[test]
    fn test_full_repository_workflow() {
        // This test simulates a complete workflow using all repositories together
        let mut graphs = Graphs::new();
        let mut events = GraphEvents::new();
        let mut nodes = Nodes::new();
        let mut edges = Edges::new();

        // Create a graph
        let graph_id = GraphIdentity::new();
        let created_event = GraphEvent::Created(GraphCreated {
            graph: graph_id,
            metadata: GraphMetadata {
                name: "Workflow Test Graph".to_string(),
                description: "Full workflow test".to_string(),
                domain: "workflow".to_string(),
                created: std::time::SystemTime::now(),
                modified: std::time::SystemTime::now(),
                tags: vec!["workflow".to_string()],
            },
            timestamp: std::time::SystemTime::now(),
        });

        // Process the event
        events.append(created_event.clone());

        // Create initial graph data
        let mut graph_data = GraphData {
            identity: graph_id,
            metadata: match &created_event {
                GraphEvent::Created(e) => e.metadata.clone(),
                _ => unreachable!(),
            },
            journey: GraphJourney::default(),
            nodes: vec![],
            edges: vec![],
        };

        graphs.store(graph_data.clone());

        // Add nodes
        let node1 = NodeIdentity::new();
        let node2 = NodeIdentity::new();

        let node1_event = GraphEvent::NodeAdded(NodeAdded {
            graph: graph_id,
            node: node1,
            content: NodeContent {
                label: "Start Node".to_string(),
                category: "process".to_string(),
                properties: HashMap::new(),
            },
            position: SpatialPosition::at_3d(0.0, 0.0, 0.0),
        });

        let node2_event = GraphEvent::NodeAdded(NodeAdded {
            graph: graph_id,
            node: node2,
            content: NodeContent {
                label: "End Node".to_string(),
                category: "process".to_string(),
                properties: HashMap::new(),
            },
            position: SpatialPosition::at_3d(100.0, 0.0, 0.0),
        });

        // Process node events
        events.append(node1_event.clone());
        events.append(node2_event.clone());

        // Update graph data
        if let GraphEvent::NodeAdded(e) = &node1_event {
            graph_data.nodes.push(NodeData {
                identity: e.node,
                content: e.content.clone(),
                position: e.position,
            });
            nodes.index_node(
                e.node,
                NodeLocation {
                    graph_id: e.graph,
                    node_id: e.node,
                },
            );
        }

        if let GraphEvent::NodeAdded(e) = &node2_event {
            graph_data.nodes.push(NodeData {
                identity: e.node,
                content: e.content.clone(),
                position: e.position,
            });
            nodes.index_node(
                e.node,
                NodeLocation {
                    graph_id: e.graph,
                    node_id: e.node,
                },
            );
        }

        // Connect nodes
        let edge_id = EdgeIdentity::new();
        let edge_event = GraphEvent::EdgeConnected(EdgeConnected {
            graph: graph_id,
            edge: edge_id,
            relationship: EdgeRelationship {
                source: node1,
                target: node2,
                category: "flows_to".to_string(),
                strength: 1.0,
                properties: HashMap::new(),
            },
        });

        events.append(edge_event.clone());

        // Update edge data
        if let GraphEvent::EdgeConnected(e) = &edge_event {
            graph_data.edges.push(EdgeData {
                identity: e.edge,
                relationship: e.relationship.clone(),
            });
            edges.add_edge(
                e.relationship.source,
                EdgeReference {
                    edge_id: e.edge,
                    target_node: e.relationship.target,
                    category: e.relationship.category.clone(),
                },
            );
        }

        // Update graph journey
        graph_data.journey.version = 1;
        graph_data.journey.event_count = 4;
        graphs.store(graph_data.clone());

        // Verify final state
        assert_eq!(events.event_count_for_graph(graph_id), 4);
        assert_eq!(nodes.nodes_in_graph(graph_id).len(), 2);
        assert!(edges.has_edge(node1, node2));

        let final_graph = graphs.find(graph_id).expect("Graph should exist");
        assert_eq!(final_graph.nodes.len(), 2);
        assert_eq!(final_graph.edges.len(), 1);
        assert_eq!(final_graph.journey.event_count, 4);

        // Create snapshot
        let snapshot = GraphSnapshot {
            graph_id,
            version: 4,
            timestamp: std::time::SystemTime::now(),
            data: graph_data,
        };
        events.store_snapshot(snapshot);

        // Verify we can retrieve from snapshot
        let latest_snapshot = events
            .latest_snapshot(graph_id)
            .expect("Should have snapshot");
        assert_eq!(latest_snapshot.version, 4);
        assert_eq!(latest_snapshot.data.nodes.len(), 2);
        assert_eq!(latest_snapshot.data.edges.len(), 1);
    }
}
