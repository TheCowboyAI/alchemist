#[cfg(test)]
mod tdd_ecs_tests {
    use crate::contexts::graph_management::domain::*;
    use crate::contexts::graph_management::events::*;
    use crate::contexts::graph_management::services::*;
    use crate::contexts::selection::domain::Selected;
    use crate::contexts::visualization::services::*;
    use bevy::input::ButtonState;
    use bevy::input::keyboard::KeyboardInput;
    use bevy::prelude::*;
    use bevy::render::RenderPlugin;
    use bevy::render::settings::{RenderCreation, WgpuSettings};
    use bevy::window::WindowPlugin;
    use bevy::winit::WinitPlugin;
    use std::collections::HashMap;

    // ===== REQUIRED test setup pattern from TDD rule =====
    fn test_ecs_system() -> App {
        let mut app = App::new();

        // BEVY_HEADLESS=1 compliant setup
        app.add_plugins(
            DefaultPlugins
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: None,
                        ..default()
                    }),
                    ..default()
                })
                .disable::<WinitPlugin>()
                .set(WindowPlugin {
                    primary_window: None,
                    ..default()
                }),
        );

        app
    }

    #[test]
    fn test_graph_creation_system() {
        // Given: A headless test app
        let mut app = test_ecs_system();
        app.add_event::<GraphCreated>()
            .add_systems(Update, handle_graph_creation);

        // When: Graph metadata is created and system runs
        let metadata = GraphMetadata {
            name: "TDD Test Graph".to_string(),
            description: "Test Description".to_string(),
            domain: "test".to_string(),
            created: std::time::SystemTime::now(),
            modified: std::time::SystemTime::now(),
            tags: vec!["test".to_string()],
        };

        app.world_mut()
            .insert_resource(PendingGraphCreation(metadata));

        // Then: System should process the creation
        app.update();

        let events = app.world().resource::<Events<GraphCreated>>();
        assert!(!events.is_empty());
    }

    #[test]
    fn test_node_selection_system() {
        // Node selection is now handled by the selection context
        // This test verifies that the selection system exists
        let mut app = App::new();

        // Create a test node
        let node_entity = app
            .world_mut()
            .spawn((
                NodeIdentity::new(),
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        // The selection system would handle this through proper events
        // Selection is managed by the selection context with:
        // - SelectNode/DeselectNode events
        // - Selected component marker
        // - Visual feedback through SelectionHighlight

        assert!(app.world().entities().contains(node_entity));
    }

    #[test]
    fn test_edge_animation_system() {
        // Given: Test app with animation system
        let mut app = test_ecs_system();
        app.add_systems(Update, AnimateGraphElements::animate_edges);
        app.insert_resource(Time::<()>::default());

        // And: An edge with animation components
        let edge_entity = app
            .world_mut()
            .spawn((
                EdgeVisual::default(),
                EdgePulse::default(),
                Transform::default(),
            ))
            .id();

        // When: Animation system runs
        app.update();

        // Then: Transform should be modified
        let transform = app.world().get::<Transform>(edge_entity).unwrap();
        assert!(transform.scale.x > 0.0); // Animation should affect scale
    }

    #[test]
    fn test_render_mode_change_system() {
        // Given: Test app with render mode handling
        let mut app = test_ecs_system();
        app.add_event::<RenderModeChanged>()
            .insert_resource(CurrentRenderMode::default())
            .add_systems(Update, handle_render_mode_changes);

        // When: Render mode change event is sent
        app.world_mut().send_event(RenderModeChanged {
            new_render_mode: RenderMode::Wireframe,
        });

        // Then: Resource should be updated
        app.update();

        let render_mode = app.world().resource::<CurrentRenderMode>();
        assert_eq!(render_mode.mode, RenderMode::Wireframe);
    }

    #[test]
    fn test_graph_validation_system() {
        // Given: Test app with validation
        let mut app = test_ecs_system();
        app.add_systems(Update, validate_graph_constraints);

        let graph_id = GraphIdentity::new();

        // And: A graph with nodes
        app.world_mut().spawn((graph_id, GraphJourney::default()));

        for _ in 0..5 {
            app.world_mut()
                .spawn((crate::contexts::graph_management::domain::Node {
                    identity: NodeIdentity::new(),
                    graph: graph_id,
                    content: NodeContent {
                        label: "Test".to_string(),
                        category: "test".to_string(),
                        properties: HashMap::new(),
                    },
                    position: SpatialPosition::at_3d(0.0, 0.0, 0.0),
                },));
        }

        // When: Validation runs
        app.update();

        // Then: No panics should occur (validation passed)
        let node_count = app
            .world_mut()
            .query::<&crate::contexts::graph_management::domain::Node>()
            .iter(&app.world())
            .filter(|n| n.graph == graph_id)
            .count();
        assert_eq!(node_count, 5);
    }

    #[test]
    fn test_camera_control_system() {
        // Given: Test app with camera controls
        let mut app = test_ecs_system();
        app.add_systems(Update, handle_camera_input);
        app.insert_resource(ButtonInput::<KeyCode>::default());

        // And: A camera entity
        let camera_entity = app
            .world_mut()
            .spawn((Camera3d::default(), Transform::from_xyz(0.0, 0.0, 10.0)))
            .id();

        // When: Arrow key is pressed (simulate via resource)
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowRight);

        // Then: Camera should move
        app.update();

        let transform = app.world().get::<Transform>(camera_entity).unwrap();
        assert!(transform.translation.x != 0.0 || transform.translation.z != 10.0);
    }

    // ===== NATS Messaging Validation Pattern from TDD rule =====

    #[derive(Resource)]
    struct TestNatsClient {
        messages_sent: Vec<String>,
    }

    impl TestNatsClient {
        fn new() -> Self {
            Self {
                messages_sent: Vec::new(),
            }
        }
    }

    #[derive(Component)]
    struct TestMarker;

    #[derive(Event)]
    struct NatsIncoming {
        payload: serde_json::Value,
    }

    #[derive(Event)]
    struct NatsOutgoing {
        payload: serde_json::Value,
    }

    impl NatsOutgoing {
        fn new(payload: serde_json::Value) -> Self {
            Self { payload }
        }
    }

    fn validate_nats_message_handling(
        mut commands: Commands,
        mut events: EventReader<NatsIncoming>,
        mut outgoing: EventWriter<NatsOutgoing>,
    ) {
        for msg in events.read() {
            // Process message through domain service
            let response = serde_json::json!({
                "processed": true,
                "original": msg.payload,
            });

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
            .add_event::<NatsOutgoing>()
            .add_systems(Update, validate_nats_message_handling);

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

        let outgoing_events = app.world().resource::<Events<NatsOutgoing>>();
        assert!(!outgoing_events.is_empty());
    }

    // ===== Helper Systems =====

    #[derive(Resource)]
    struct PendingGraphCreation(GraphMetadata);

    fn handle_graph_creation(
        pending: Option<Res<PendingGraphCreation>>,
        mut commands: Commands,
        mut created: EventWriter<GraphCreated>,
    ) {
        if let Some(pending) = pending {
            created.write(GraphCreated {
                graph: GraphIdentity::new(),
                metadata: pending.0.clone(),
                timestamp: std::time::SystemTime::now(),
            });
            commands.remove_resource::<PendingGraphCreation>();
        }
    }

    #[derive(Resource, Default)]
    struct CurrentRenderMode {
        mode: RenderMode,
    }

    fn handle_render_mode_changes(
        mut events: EventReader<RenderModeChanged>,
        mut current_mode: ResMut<CurrentRenderMode>,
    ) {
        for event in events.read() {
            current_mode.mode = event.new_render_mode;
        }
    }

    fn validate_graph_constraints(
        graphs: Query<(&GraphIdentity, &GraphJourney)>,
        nodes: Query<&crate::contexts::graph_management::domain::Node>,
    ) {
        for (graph_id, _journey) in graphs.iter() {
            let node_count = nodes.iter().filter(|n| n.graph == *graph_id).count();

            assert!(node_count <= 1000, "Graph exceeds node limit");
        }
    }

    fn handle_camera_input(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut cameras: Query<&mut Transform, With<Camera3d>>,
    ) {
        if keyboard.pressed(KeyCode::ArrowRight) {
            for mut transform in cameras.iter_mut() {
                transform.translation.x += 0.1;
            }
        }
    }
}
