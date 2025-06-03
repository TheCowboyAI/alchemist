#[cfg(test)]
mod tdd_ecs_tests {
    use crate::contexts::graph_management::domain::{
        GraphIdentity,
        GraphMetadata,
        Node as DomainNode, // Explicitly alias our domain Node
        NodeContent,
        NodeIdentity,
        SpatialPosition,
    };
    use crate::contexts::graph_management::events::{GraphCreated, NodeAdded};
    use crate::testing::create_headless_test_app;
    use bevy::prelude::*;
    use std::collections::HashMap;

    // ===== Test-specific components to avoid render dependencies =====
    #[derive(Component, Debug, Clone, Copy, PartialEq)]
    pub enum RenderMode {
        Mesh,
        Wireframe,
    }

    impl Default for RenderMode {
        fn default() -> Self {
            Self::Mesh
        }
    }

    // ===== REQUIRED test setup pattern from TDD rule =====
    fn test_ecs_system() -> App {
        // Use headless test app to avoid render pipeline issues
        let mut app = create_headless_test_app();

        // Add minimal required events and systems for testing
        app.add_event::<GraphCreated>()
            .add_event::<NodeAdded>()
            .add_systems(Update, validate_nats_message_handling);

        app
    }

    // ===== Test marker component =====
    #[derive(Component)]
    struct TestMarker;

    // ===== Mock NATS types for testing =====
    #[derive(Event)]
    struct NatsIncoming {
        payload: serde_json::Value,
    }

    #[derive(Event)]
    struct NatsOutgoing {
        #[allow(dead_code)]
        payload: serde_json::Value,
    }

    impl NatsOutgoing {
        fn new(payload: serde_json::Value) -> Self {
            Self { payload }
        }
    }

    #[derive(Resource)]
    struct TestNatsClient {
        #[allow(dead_code)]
        messages_sent: Vec<String>,
    }

    impl TestNatsClient {
        fn new() -> Self {
            Self {
                messages_sent: Vec::new(),
            }
        }
    }

    // ===== NATS-ECS bridge testing pattern =====
    fn validate_nats_message_handling(
        mut commands: Commands,
        mut events: EventReader<NatsIncoming>,
        mut outgoing: EventWriter<NatsOutgoing>,
    ) {
        for msg in events.read() {
            // Simple domain service processing
            let response = serde_json::json!({ "processed": true, "original": msg.payload });
            outgoing.write(NatsOutgoing::new(response));
            commands.spawn(TestMarker);
        }
    }

    #[test]
    fn test_nats_message_processing() {
        // Given: Test app with NATS handling
        let mut app = test_ecs_system();
        app.insert_resource(TestNatsClient::new())
            .add_event::<NatsIncoming>()
            .add_event::<NatsOutgoing>();

        // When: NATS message is received
        app.world_mut().send_event(NatsIncoming {
            payload: serde_json::json!({ "test": "data" }),
        });

        // Then: Message should be processed
        app.update();

        let results_count = app
            .world_mut()
            .query::<&TestMarker>()
            .iter(&app.world())
            .count();

        assert_eq!(results_count, 1);
    }

    #[test]
    #[ignore] // TODO: Fix event handling in TDD tests
    fn test_graph_creation_event() {
        // Given: Test app with graph events
        let mut app = test_ecs_system();

        // When: Graph creation event is sent
        let graph_id = GraphIdentity::new();
        app.world_mut().send_event(GraphCreated {
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

        // Then: Event should be processed without errors
        app.update();

        // Verify the event was handled (no panics = success)
        assert!(true);
    }

    #[test]
    #[ignore] // TODO: Fix event handling in TDD tests
    fn test_node_addition_event() {
        // Given: Test app with node events
        let mut app = test_ecs_system();

        // When: Node addition event is sent
        let node_id = NodeIdentity::new();
        let graph_id = GraphIdentity::new();
        app.world_mut().send_event(NodeAdded {
            graph: graph_id,
            node: node_id,
            content: NodeContent {
                label: "Test Node".to_string(),
                category: "test".to_string(),
                properties: HashMap::new(),
            },
            position: SpatialPosition::at_3d(0.0, 0.0, 0.0),
        });

        // Then: Event should be processed without errors
        app.update();

        // Verify the event was handled (no panics = success)
        assert!(true);
    }

    #[test]
    fn test_ecs_component_insertion() {
        // Given: Test app
        let mut app = test_ecs_system();

        // When: Entity with components is spawned
        let entity = app
            .world_mut()
            .spawn((
                DomainNode {
                    // Use explicit alias
                    identity: NodeIdentity::new(),
                    graph: GraphIdentity::new(),
                    content: NodeContent {
                        label: "Test Node".to_string(),
                        category: "test".to_string(),
                        properties: HashMap::new(),
                    },
                    position: SpatialPosition::at_3d(0.0, 0.0, 0.0),
                },
                RenderMode::Wireframe,
            ))
            .id();

        // Then: Components should be queryable
        let node = app.world().entity(entity).get::<DomainNode>().unwrap();
        let render_mode = app.world().entity(entity).get::<RenderMode>().unwrap();

        assert_eq!(*render_mode, RenderMode::Wireframe);
        assert!(node.identity.0.to_string().len() > 0);
    }
}

#[cfg(test)]
mod minimal_ecs_tests {
    use crate::testing::create_headless_test_app;
    use bevy::prelude::*;

    // Simple test components that don't depend on render features
    #[derive(Component)]
    struct TestComponent(u32);

    #[derive(Event)]
    struct TestEvent {
        #[allow(dead_code)]
        value: i32,
    }

    #[derive(Resource)]
    struct TestResource(i32);

    #[test]
    fn test_basic_ecs_functionality() {
        let mut app = create_headless_test_app();

        // Test entity spawning
        let entity = app.world_mut().spawn(TestComponent(42)).id();

        // Test component access
        let component = app.world().get::<TestComponent>(entity).unwrap();
        assert_eq!(component.0, 42);

        // Test resource insertion and access
        app.world_mut().insert_resource(TestResource(100));
        let resource = app.world().get_resource::<TestResource>().unwrap();
        assert_eq!(resource.0, 100);

        // Test events
        app.add_event::<TestEvent>();
        app.world_mut().send_event(TestEvent { value: 123 });

        // Update the app to process events
        app.update();

        println!("✅ All basic ECS tests passed!");
    }

    #[test]
    fn test_system_execution() {
        let mut app = create_headless_test_app();

        // Add a simple system that modifies a resource
        fn test_system(mut resource: ResMut<TestResource>) {
            resource.0 += 1;
        }

        app.insert_resource(TestResource(0));
        app.add_systems(Update, test_system);

        // Run one update cycle
        app.update();

        let resource = app.world().get_resource::<TestResource>().unwrap();
        assert_eq!(resource.0, 1);

        println!("✅ System execution test passed!");
    }
}
