#[cfg(test)]
mod automated_graph_tests {
    use bevy::prelude::*;
    use crate::contexts::graph_management::domain::*;
    use crate::contexts::graph_management::services::*;
    use crate::contexts::visualization::services::*;

    /// Automated test for graph creation and node addition workflow
    #[test]
    fn test_automated_graph_creation_workflow() {
        let mut app = setup_headless_test_app();

        // Simulate creating a graph via UI
        simulate_create_graph_button_click(&mut app);
        app.update();

        // Verify graph was created
        let graphs = app.world().resource::<Graphs>();
        assert_eq!(graphs.len(), 1);

        // Simulate adding nodes
        simulate_add_node_at_position(&mut app, Vec2::new(100.0, 100.0));
        simulate_add_node_at_position(&mut app, Vec2::new(200.0, 200.0));
        app.update();

        // Verify nodes were added
        let nodes = app.world().resource::<Nodes>();
        assert_eq!(nodes.len(), 2);

        // Simulate connecting nodes
        simulate_drag_edge_between_nodes(&mut app, 0, 1);
        app.update();

        // Verify edge was created
        let edges = app.world().resource::<Edges>();
        assert_eq!(edges.len(), 1);
    }

    /// Test keyboard navigation
    #[test]
    fn test_keyboard_navigation() {
        let mut app = setup_headless_test_app();

        // Add test data
        create_test_graph_with_nodes(&mut app, 3);

        // Simulate keyboard navigation
        simulate_key_press(&mut app, KeyCode::Tab);
        app.update();

        // Verify selection changed
        let selected = app.world().query::<&Selected>().iter(&app.world()).count();
        assert_eq!(selected, 1);

        // Test arrow key navigation
        simulate_key_press(&mut app, KeyCode::ArrowRight);
        app.update();

        // Verify camera moved (would check transform)
    }

    /// Test render mode switching
    #[test]
    fn test_render_mode_switching() {
        let mut app = setup_headless_test_app();

        // Press 'W' for wireframe mode
        simulate_key_press(&mut app, KeyCode::KeyW);
        app.update();

        // Verify render mode changed
        let events = app.world().resource::<Events<RenderModeChanged>>();
        assert!(!events.is_empty());
    }

    /// Test multi-selection with drag
    #[test]
    fn test_box_selection() {
        let mut app = setup_headless_test_app();

        // Create nodes in specific positions
        create_nodes_at_positions(&mut app, vec![
            Vec2::new(100.0, 100.0),
            Vec2::new(150.0, 150.0),
            Vec2::new(200.0, 200.0),
        ]);

        // Simulate box selection drag
        simulate_box_selection(&mut app,
            Vec2::new(90.0, 90.0),
            Vec2::new(160.0, 160.0)
        );
        app.update();

        // Verify correct nodes selected
        let selected = app.world().query::<&Selected>().iter(&app.world()).count();
        assert_eq!(selected, 2); // Should select first two nodes
    }

    /// Test undo/redo functionality
    #[test]
    fn test_undo_redo() {
        let mut app = setup_headless_test_app();

        // Perform actions
        simulate_add_node_at_position(&mut app, Vec2::new(100.0, 100.0));
        app.update();

        let nodes_before = app.world().resource::<Nodes>().len();

        // Undo
        simulate_key_combo(&mut app, &[KeyCode::ControlLeft, KeyCode::KeyZ]);
        app.update();

        let nodes_after_undo = app.world().resource::<Nodes>().len();
        assert_eq!(nodes_after_undo, nodes_before - 1);

        // Redo
        simulate_key_combo(&mut app, &[KeyCode::ControlLeft, KeyCode::KeyY]);
        app.update();

        let nodes_after_redo = app.world().resource::<Nodes>().len();
        assert_eq!(nodes_after_redo, nodes_before);
    }

    // Helper functions

    fn setup_headless_test_app() -> App {
        use bevy::render::settings::{RenderCreation, WgpuSettings};
        use bevy::render::RenderPlugin;
        use bevy::window::WindowPlugin;
        use bevy::winit::WinitPlugin;

        let mut app = App::new();

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
                })
        );

        // Add our plugins
        app.add_plugins((
            crate::contexts::graph_management::plugin::GraphManagementPlugin,
            crate::contexts::visualization::plugin::VisualizationPlugin,
        ));

        app
    }

    fn simulate_create_graph_button_click(app: &mut App) {
        app.world_mut().send_event(CreateGraph {
            metadata: GraphMetadata::new("Test Graph"),
        });
    }

    fn simulate_add_node_at_position(app: &mut App, position: Vec2) {
        // Simulate mouse click at position
        app.world_mut().send_event(CursorMoved {
            window: Entity::PLACEHOLDER,
            position,
            delta: None,
        });

        app.world_mut().send_event(MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Pressed,
            window: Entity::PLACEHOLDER,
        });
    }

    fn simulate_drag_edge_between_nodes(app: &mut App, from_idx: usize, to_idx: usize) {
        // This would simulate dragging from one node to another
        // Implementation depends on your edge creation UI
    }

    fn simulate_key_press(app: &mut App, key: KeyCode) {
        app.world_mut().send_event(KeyboardInput {
            key_code: key,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state: ButtonState::Pressed,
            window: Entity::PLACEHOLDER,
        });

        app.world_mut().send_event(KeyboardInput {
            key_code: key,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state: ButtonState::Released,
            window: Entity::PLACEHOLDER,
        });
    }

    fn simulate_key_combo(app: &mut App, keys: &[KeyCode]) {
        // Press all keys
        for &key in keys {
            app.world_mut().send_event(KeyboardInput {
                key_code: key,
                logical_key: Key::Unidentified(NativeKey::Unidentified),
                state: ButtonState::Pressed,
                window: Entity::PLACEHOLDER,
            });
        }

        // Release all keys in reverse order
        for &key in keys.iter().rev() {
            app.world_mut().send_event(KeyboardInput {
                key_code: key,
                logical_key: Key::Unidentified(NativeKey::Unidentified),
                state: ButtonState::Released,
                window: Entity::PLACEHOLDER,
            });
        }
    }

    fn simulate_box_selection(app: &mut App, start: Vec2, end: Vec2) {
        // Start drag
        app.world_mut().send_event(CursorMoved {
            window: Entity::PLACEHOLDER,
            position: start,
            delta: None,
        });

        app.world_mut().send_event(MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Pressed,
            window: Entity::PLACEHOLDER,
        });

        // Drag to end
        app.world_mut().send_event(CursorMoved {
            window: Entity::PLACEHOLDER,
            position: end,
            delta: Some(end - start),
        });

        // Release
        app.world_mut().send_event(MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Released,
            window: Entity::PLACEHOLDER,
        });
    }

    fn create_test_graph_with_nodes(app: &mut App, node_count: usize) {
        let graph_id = GraphIdentity::new();
        app.world_mut().spawn(Graph {
            identity: graph_id,
            metadata: GraphMetadata::new("Test Graph"),
        });

        for i in 0..node_count {
            app.world_mut().spawn(Node {
                identity: NodeIdentity::new(),
                position: NodePosition {
                    x: (i as f32) * 100.0,
                    y: 0.0,
                    z: 0.0,
                },
                graph: graph_id,
                properties: NodeProperties::default(),
            });
        }
    }

    fn create_nodes_at_positions(app: &mut App, positions: Vec<Vec2>) {
        let graph_id = GraphIdentity::new();
        app.world_mut().spawn(Graph {
            identity: graph_id,
            metadata: GraphMetadata::new("Test Graph"),
        });

        for pos in positions {
            app.world_mut().spawn(Node {
                identity: NodeIdentity::new(),
                position: NodePosition {
                    x: pos.x,
                    y: pos.y,
                    z: 0.0,
                },
                graph: graph_id,
                properties: NodeProperties::default(),
            });
        }
    }
}
