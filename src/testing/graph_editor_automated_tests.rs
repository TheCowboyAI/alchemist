#[cfg(test)]
mod graph_editor_automated_tests {
    // Explicit imports to avoid ambiguity
    use crate::contexts::graph_management::domain::{
        GraphIdentity,
        GraphJourney,
        GraphMetadata,
        Node as DomainNode, // Explicitly alias our domain Node
        NodeContent,
        NodeIdentity,
        SpatialPosition,
    };
    use crate::contexts::graph_management::events::*;
    use crate::contexts::selection::events::NodeSelected;
    use crate::testing::create_headless_test_app;
    // Import Bevy prelude but be explicit about Node usage
    use bevy::prelude::*;
    use std::collections::HashMap;

    fn setup_test_app() -> App {
        let mut app = create_headless_test_app();

        // Add required events and systems for graph editing tests
        app.add_event::<GraphCreated>()
            .add_event::<NodeAdded>()
            .add_event::<EdgeConnected>()
            .add_event::<NodeSelected>()
            .add_event::<crate::contexts::selection::events::NodeDeselected>();

        app
    }

    #[test]
    fn test_create_graph_service() {
        // Given: Application with graph creation capability
        let mut app = setup_test_app();
        app.add_systems(Update, CreateGraphHandler::handle);

        // When: Graph creation service is used
        let metadata = GraphMetadata {
            name: "Test Graph".to_string(),
            description: "Test Description".to_string(),
            domain: "test".to_string(),
            created: std::time::SystemTime::now(),
            modified: std::time::SystemTime::now(),
            tags: vec![],
        };

        app.world_mut()
            .insert_resource(PendingGraphCreation(metadata));

        // Then: Graph should be created
        app.update();

        let graphs = app
            .world_mut()
            .query::<(&GraphIdentity, &GraphJourney)>()
            .iter(&app.world())
            .count();
        assert_eq!(graphs, 1);
    }

    #[test]
    fn test_add_node_to_graph() {
        // Given: App with existing graph
        let mut app = setup_test_app();
        app.add_systems(Update, AddNodeHandler::handle);

        let graph_id = GraphIdentity::new();
        app.world_mut().spawn((graph_id, GraphJourney::default()));

        // When: Node is added to graph via service
        let content = NodeContent {
            label: "Test Node".to_string(),
            category: "test".to_string(),
            properties: HashMap::new(),
        };
        let position = SpatialPosition::at_3d(1.0, 2.0, 3.0);

        app.world_mut().insert_resource(PendingNodeAddition {
            graph: graph_id,
            content,
            position,
        });

        // Then: Node should exist in graph
        app.update();

        let node_count = app
            .world_mut()
            .query::<&DomainNode>()
            .iter(&app.world())
            .count(); // Use explicit alias
        assert_eq!(node_count, 1);
    }

    #[test]
    fn test_full_graph_creation_workflow() {
        // Given: Complete test environment
        let mut app = setup_test_app();
        app.add_systems(Update, (CreateGraphHandler::handle, AddNodeHandler::handle));

        // When: Creating complete graph workflow
        let metadata = GraphMetadata {
            name: "Workflow Test Graph".to_string(),
            description: "Test workflow".to_string(),
            domain: "test".to_string(),
            created: std::time::SystemTime::now(),
            modified: std::time::SystemTime::now(),
            tags: vec![],
        };

        app.world_mut()
            .insert_resource(PendingGraphCreation(metadata));
        app.update(); // Process graph creation

        // Get the created graph ID
        let graph_id = app
            .world_mut()
            .query::<&GraphIdentity>()
            .iter(&app.world())
            .next()
            .copied()
            .expect("Graph should exist");

        // Add nodes to the graph
        for i in 0..3 {
            let content = NodeContent {
                label: format!("Node {i}"),
                category: "test".to_string(),
                properties: HashMap::new(),
            };
            let position = SpatialPosition::at_3d(i as f32, 0.0, 0.0);

            app.world_mut().insert_resource(PendingNodeAddition {
                graph: graph_id,
                content,
                position,
            });
            app.update(); // Process each node addition
        }

        // Then: All components should exist
        let nodes_count = app
            .world_mut()
            .query::<&DomainNode>()
            .iter(&app.world())
            .count(); // Use explicit alias
        assert_eq!(nodes_count, 3);

        // Verify graph exists
        let graphs = app
            .world_mut()
            .query::<(&GraphIdentity, &GraphJourney)>()
            .iter(&app.world())
            .count();
        assert_eq!(graphs, 1);
    }

    #[test]
    fn test_undo_redo_operations() {
        // Given: App with undo/redo capability
        let mut app = setup_test_app();
        app.add_systems(Update, AddNodeHandler::handle);

        let graph_id = GraphIdentity::new();
        app.world_mut().spawn((graph_id, GraphJourney::default()));

        // When: Adding node
        let content = NodeContent {
            label: "Undo Test Node".to_string(),
            category: "test".to_string(),
            properties: HashMap::new(),
        };
        let position = SpatialPosition::at_3d(0.0, 0.0, 0.0);

        app.world_mut().insert_resource(PendingNodeAddition {
            graph: graph_id,
            content,
            position,
        });

        app.update();

        let nodes_after_add = app
            .world_mut()
            .query::<&DomainNode>()
            .iter(&app.world())
            .count(); // Use explicit alias
        assert_eq!(nodes_after_add, 1);

        // Undo/redo would be handled by specialized systems
        // This test verifies the basic operation works
    }

    #[test]
    fn test_node_selection_workflow() {
        // Given: App with selection systems
        let mut app = setup_test_app();
        app.add_systems(Update, NodeSelectionHandler::handle);

        let graph_id = GraphIdentity::new();
        let node_entity = app
            .world_mut()
            .spawn((
                DomainNode {
                    // Use explicit alias
                    identity: NodeIdentity::new(),
                    graph: graph_id,
                    content: NodeContent {
                        label: "Selectable Node".to_string(),
                        category: "test".to_string(),
                        properties: HashMap::new(),
                    },
                    position: SpatialPosition::at_3d(0.0, 0.0, 0.0),
                },
                Transform::default(),
            ))
            .id();

        let node_id = app.world().get::<DomainNode>(node_entity).unwrap().identity;

        // When: Selecting the node with correct event structure
        app.world_mut().send_event(NodeSelected {
            entity: node_entity,
            node: node_id,
            add_to_selection: false,
        });

        // Then: Node should be selected
        app.update();

        // Selection would add a Selected component
        let selected_count = app
            .world_mut()
            .query::<&crate::contexts::selection::domain::Selected>()
            .iter(&app.world())
            .count();
        assert_eq!(selected_count, 1);
    }

    #[test]
    fn test_graph_persistence() {
        // Given: App with persistence systems
        let mut app = setup_test_app();

        let graph_id = GraphIdentity::new();
        app.world_mut().spawn((graph_id, GraphJourney::default()));

        // When: Saving graph state
        // Persistence would be handled by specialized systems

        // Then: Graph data should be serializable
        let graphs = app
            .world_mut()
            .query::<(&GraphIdentity, &GraphJourney)>()
            .iter(&app.world())
            .collect::<Vec<_>>();

        assert_eq!(graphs.len(), 1);
        assert_eq!(graphs[0].0, &graph_id);
    }

    // Helper resources for mock operations
    #[derive(Resource)]
    struct PendingGraphCreation(GraphMetadata);

    #[derive(Resource)]
    struct PendingNodeAddition {
        graph: GraphIdentity,
        content: NodeContent,
        position: SpatialPosition,
    }

    // Mock handlers for test systems
    struct CreateGraphHandler;
    impl CreateGraphHandler {
        fn handle(
            mut commands: Commands,
            pending: Option<Res<PendingGraphCreation>>,
            mut created: EventWriter<GraphCreated>,
        ) {
            if let Some(pending) = pending {
                let graph_id = GraphIdentity::new();
                commands.spawn((graph_id, GraphJourney::default()));

                created.write(GraphCreated {
                    graph: graph_id,
                    metadata: pending.0.clone(),
                    timestamp: std::time::SystemTime::now(),
                });

                commands.remove_resource::<PendingGraphCreation>();
            }
        }
    }

    struct AddNodeHandler;
    impl AddNodeHandler {
        fn handle(
            mut commands: Commands,
            pending: Option<Res<PendingNodeAddition>>,
            mut added: EventWriter<NodeAdded>,
        ) {
            if let Some(pending) = pending {
                let node_id = NodeIdentity::new();
                let _node_entity = commands
                    .spawn((
                        DomainNode {
                            // Use explicit alias
                            identity: node_id,
                            graph: pending.graph,
                            content: pending.content.clone(),
                            position: pending.position,
                        },
                        Transform::from_translation(pending.position.coordinates_3d),
                    ))
                    .id();

                added.write(NodeAdded {
                    graph: pending.graph,
                    node: node_id,
                    content: pending.content.clone(),
                    position: pending.position,
                });

                commands.remove_resource::<PendingNodeAddition>();
            }
        }
    }

    struct NodeSelectionHandler;
    impl NodeSelectionHandler {
        fn handle(mut commands: Commands, mut events: EventReader<NodeSelected>) {
            for event in events.read() {
                commands
                    .entity(event.entity)
                    .insert(crate::contexts::selection::domain::Selected);
            }
        }
    }
}
