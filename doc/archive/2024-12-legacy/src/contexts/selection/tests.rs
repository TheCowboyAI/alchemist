#[cfg(test)]
mod tests {
    use super::super::{domain::*, events::*, services::*};
    use crate::contexts::graph_management::domain::{
        EdgeIdentity, EdgeRelationship, GraphIdentity, NodeContent, NodeIdentity, SpatialPosition,
    };
    use crate::contexts::selection::test_utils::create_test_app;
    use crate::contexts::visualization::services::PerformRaycast;
    use bevy::math::Ray3d;
    use bevy::prelude::*;

    // Helper function to create a test app with selection plugin
    fn setup_test_app() -> App {
        let mut app = create_test_app();
        app.init_resource::<SelectionState>()
            .add_event::<SelectionChanged>()
            .add_event::<SelectionCleared>()
            .add_event::<SelectionModeChanged>()
            .add_event::<NodeSelected>()
            .add_event::<NodeDeselected>()
            .add_event::<EdgeSelected>()
            .add_event::<EdgeDeselected>()
            .add_event::<AllSelected>()
            .add_event::<SelectionInverted>();
        app
    }

    #[test]
    fn test_selection_state_initialization() {
        let selection_state = SelectionState::default();

        assert_eq!(selection_state.selected_nodes.len(), 0);
        assert_eq!(selection_state.selected_edges.len(), 0);
        assert_eq!(selection_state.selection_count(), 0);
        assert!(matches!(
            selection_state.selection_mode,
            SelectionMode::Single
        ));
    }

    #[test]
    fn test_node_selection() {
        let mut selection_state = SelectionState::default();
        let node_id = NodeIdentity::new();

        // Test selecting a node
        selection_state.select_node(node_id);
        assert!(selection_state.is_node_selected(&node_id));
        assert_eq!(selection_state.selected_nodes.len(), 1);
        assert_eq!(selection_state.last_selected_node, Some(node_id));

        // Test deselecting a node
        selection_state.deselect_node(&node_id);
        assert!(!selection_state.is_node_selected(&node_id));
        assert_eq!(selection_state.selected_nodes.len(), 0);
        assert_eq!(selection_state.last_selected_node, None);
    }

    #[test]
    fn test_edge_selection() {
        let mut selection_state = SelectionState::default();
        let edge_id = EdgeIdentity::new();

        // Test selecting an edge
        selection_state.select_edge(edge_id);
        assert!(selection_state.is_edge_selected(&edge_id));
        assert_eq!(selection_state.selected_edges.len(), 1);
        assert_eq!(selection_state.last_selected_edge, Some(edge_id));

        // Test deselecting an edge
        selection_state.deselect_edge(&edge_id);
        assert!(!selection_state.is_edge_selected(&edge_id));
        assert_eq!(selection_state.selected_edges.len(), 0);
        assert_eq!(selection_state.last_selected_edge, None);
    }

    #[test]
    fn test_clear_selection() {
        let mut selection_state = SelectionState::default();
        let node_id = NodeIdentity::new();
        let edge_id = EdgeIdentity::new();

        // Add some selections
        selection_state.select_node(node_id);
        selection_state.select_edge(edge_id);
        assert_eq!(selection_state.selection_count(), 2);

        // Clear all selections
        selection_state.clear();
        assert_eq!(selection_state.selection_count(), 0);
        assert!(!selection_state.is_node_selected(&node_id));
        assert!(!selection_state.is_edge_selected(&edge_id));
        assert_eq!(selection_state.last_selected_node, None);
        assert_eq!(selection_state.last_selected_edge, None);
    }

    #[test]
    fn test_selection_mode_changes() {
        let mut app = setup_test_app();

        // Add selection mode change system
        app.add_systems(Update, ManageSelection::handle_selection_mode_changed);

        // Send mode change event
        let mut events = app
            .world_mut()
            .resource_mut::<Events<SelectionModeChanged>>();
        events.send(SelectionModeChanged {
            new_mode: SelectionMode::Multiple,
            previous_mode: SelectionMode::Single,
        });

        // Run the app
        app.update();

        // Check that mode was changed
        let selection_state = app.world().resource::<SelectionState>();
        assert!(matches!(
            selection_state.selection_mode,
            SelectionMode::Multiple
        ));
    }

    #[test]
    fn test_select_node_event_single_mode() {
        let mut app = setup_test_app();

        // Create test entities
        let entity1 = app.world_mut().spawn(()).id();
        let entity2 = app.world_mut().spawn(()).id();
        let node_id1 = NodeIdentity::new();
        let node_id2 = NodeIdentity::new();

        // Add selection systems
        app.add_systems(Update, ManageSelection::handle_node_selected);

        // Select first node
        app.world_mut()
            .resource_mut::<Events<NodeSelected>>()
            .send(NodeSelected {
                entity: entity1,
                node: node_id1,
                add_to_selection: false,
            });

        app.update();

        // Check first node is selected
        let selection_state = app.world().resource::<SelectionState>();
        assert!(selection_state.is_node_selected(&node_id1));
        assert_eq!(selection_state.selected_nodes.len(), 1);

        // Select second node (should replace first in Single mode)
        app.world_mut()
            .resource_mut::<Events<NodeSelected>>()
            .send(NodeSelected {
                entity: entity2,
                node: node_id2,
                add_to_selection: false,
            });

        app.update();

        // Check only second node is selected
        let selection_state = app.world().resource::<SelectionState>();
        assert!(!selection_state.is_node_selected(&node_id1));
        assert!(selection_state.is_node_selected(&node_id2));
        assert_eq!(selection_state.selected_nodes.len(), 1);
    }

    #[test]
    fn test_select_node_event_multiple_mode() {
        let mut app = setup_test_app();

        // Set to Multiple mode
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selection_mode = SelectionMode::Multiple;

        // Create test entities
        let entity1 = app.world_mut().spawn(()).id();
        let entity2 = app.world_mut().spawn(()).id();
        let node_id1 = NodeIdentity::new();
        let node_id2 = NodeIdentity::new();

        // Add selection systems
        app.add_systems(Update, ManageSelection::handle_node_selected);

        // Select first node
        app.world_mut()
            .resource_mut::<Events<NodeSelected>>()
            .send(NodeSelected {
                entity: entity1,
                node: node_id1,
                add_to_selection: true,
            });

        app.update();

        // Select second node with add_to_selection
        app.world_mut()
            .resource_mut::<Events<NodeSelected>>()
            .send(NodeSelected {
                entity: entity2,
                node: node_id2,
                add_to_selection: true,
            });

        app.update();

        // Check both nodes are selected
        let selection_state = app.world().resource::<SelectionState>();
        assert!(selection_state.is_node_selected(&node_id1));
        assert!(selection_state.is_node_selected(&node_id2));
        assert_eq!(selection_state.selected_nodes.len(), 2);
    }

    #[test]
    fn test_selection_cleared_event() {
        let mut app = setup_test_app();

        // Add some selections
        let node_id = NodeIdentity::new();
        let edge_id = EdgeIdentity::new();
        app.world_mut()
            .resource_mut::<SelectionState>()
            .select_node(node_id);
        app.world_mut()
            .resource_mut::<SelectionState>()
            .select_edge(edge_id);

        // Add clear selection system
        app.add_systems(Update, ManageSelection::handle_selection_cleared);

        // Send clear event
        app.world_mut()
            .resource_mut::<Events<SelectionCleared>>()
            .send(SelectionCleared);

        app.update();

        // Check all selections are cleared
        let selection_state = app.world().resource::<SelectionState>();
        assert_eq!(selection_state.selection_count(), 0);
    }

    #[test]
    fn test_selection_box_creation() {
        let start_pos = Vec2::new(100.0, 100.0);
        let selection_box = SelectionBox::new(start_pos);

        assert_eq!(selection_box.start, start_pos);
        assert_eq!(selection_box.end, start_pos);
        assert!(selection_box.active);
    }

    #[test]
    fn test_selection_box_bounds() {
        let mut selection_box = SelectionBox::new(Vec2::new(100.0, 100.0));
        selection_box.end = Vec2::new(200.0, 50.0);

        let (min, max) = selection_box.bounds();
        assert_eq!(min, Vec2::new(100.0, 50.0));
        assert_eq!(max, Vec2::new(200.0, 100.0));
    }

    #[test]
    fn test_selection_box_contains() {
        let mut selection_box = SelectionBox::new(Vec2::new(100.0, 100.0));
        selection_box.end = Vec2::new(200.0, 200.0);

        // Test points inside the box
        assert!(selection_box.contains(Vec2::new(150.0, 150.0)));
        assert!(selection_box.contains(Vec2::new(100.0, 100.0)));
        assert!(selection_box.contains(Vec2::new(200.0, 200.0)));

        // Test points outside the box
        assert!(!selection_box.contains(Vec2::new(50.0, 150.0)));
        assert!(!selection_box.contains(Vec2::new(250.0, 150.0)));
        assert!(!selection_box.contains(Vec2::new(150.0, 50.0)));
        assert!(!selection_box.contains(Vec2::new(150.0, 250.0)));
    }

    #[test]
    fn test_selection_highlight_default() {
        let highlight = SelectionHighlight::default();

        assert_eq!(highlight.original_color, Color::WHITE);
        assert_eq!(highlight.highlight_color, Color::srgb(1.0, 0.8, 0.2)); // Golden
        assert_eq!(highlight.highlight_intensity, 0.5);
    }

    #[test]
    fn test_deselect_node_event() {
        let mut app = setup_test_app();

        // Create test entity and select it
        let entity = app.world_mut().spawn(()).id();
        let node_id = NodeIdentity::new();
        app.world_mut()
            .resource_mut::<SelectionState>()
            .select_node(node_id);

        // Add deselection system
        app.add_systems(Update, ManageSelection::handle_node_deselected);

        // Send deselect event
        app.world_mut()
            .resource_mut::<Events<NodeDeselected>>()
            .send(NodeDeselected {
                entity,
                node: node_id,
            });

        app.update();

        // Check node is deselected
        let selection_state = app.world().resource::<SelectionState>();
        assert!(!selection_state.is_node_selected(&node_id));
    }

    #[test]
    #[ignore] // TODO: Fix event handling in tests
    fn test_select_all_event() {
        let mut app = setup_test_app();

        // Add the necessary components and systems
        app.add_systems(Update, AdvancedSelection::handle_all_selected);

        // Create test node entities with proper components
        let node1_id = NodeIdentity::new();
        let node2_id = NodeIdentity::new();

        let _node1 = app
            .world_mut()
            .spawn((
                crate::contexts::graph_management::domain::Node {
                    identity: node1_id,
                    graph: GraphIdentity::new(),
                    content: NodeContent {
                        label: "Node1".to_string(),
                        category: "test".to_string(),
                        properties: Default::default(),
                    },
                    position: SpatialPosition::at_3d(0.0, 0.0, 0.0),
                },
                Selectable,
            ))
            .id();

        let _node2 = app
            .world_mut()
            .spawn((
                crate::contexts::graph_management::domain::Node {
                    identity: node2_id,
                    graph: GraphIdentity::new(),
                    content: NodeContent {
                        label: "Node2".to_string(),
                        category: "test".to_string(),
                        properties: Default::default(),
                    },
                    position: SpatialPosition::at_3d(1.0, 0.0, 0.0),
                },
                Selectable,
            ))
            .id();

        // Create test edge entities with proper components
        let edge1_id = EdgeIdentity::new();
        let edge2_id = EdgeIdentity::new();

        let _edge1 = app
            .world_mut()
            .spawn((
                crate::contexts::graph_management::domain::Edge {
                    identity: edge1_id,
                    graph: GraphIdentity::new(),
                    relationship: EdgeRelationship {
                        source: node1_id,
                        target: node2_id,
                        category: "test".to_string(),
                        strength: 1.0,
                        properties: Default::default(),
                    },
                },
                Selectable,
            ))
            .id();

        let _edge2 = app
            .world_mut()
            .spawn((
                crate::contexts::graph_management::domain::Edge {
                    identity: edge2_id,
                    graph: GraphIdentity::new(),
                    relationship: EdgeRelationship {
                        source: node2_id,
                        target: node1_id,
                        category: "test".to_string(),
                        strength: 1.0,
                        properties: Default::default(),
                    },
                },
                Selectable,
            ))
            .id();

        // Send SelectAll event
        app.world_mut()
            .resource_mut::<Events<AllSelected>>()
            .send(AllSelected);

        // Run the system
        app.update();

        // Now verify that SelectNode and SelectEdge events were generated
        let select_node_events = app.world().resource::<Events<NodeSelected>>();
        let select_edge_events = app.world().resource::<Events<EdgeSelected>>();

        // Should have 2 SelectNode events
        assert_eq!(
            select_node_events.len(),
            2,
            "Should have generated 2 SelectNode events"
        );

        // Should have 2 SelectEdge events
        assert_eq!(
            select_edge_events.len(),
            2,
            "Should have generated 2 SelectEdge events"
        );

        // Verify the events contain the correct entities
        let node_events: Vec<_> = select_node_events.iter_current_update_events().collect();
        assert!(node_events.iter().any(|e| e.node == node1_id));
        assert!(node_events.iter().any(|e| e.node == node2_id));

        let edge_events: Vec<_> = select_edge_events.iter_current_update_events().collect();
        assert!(edge_events.iter().any(|e| e.edge == edge1_id));
        assert!(edge_events.iter().any(|e| e.edge == edge2_id));
    }

    #[test]
    fn test_selection_changed_event_fired() {
        let mut app = setup_test_app();

        // Add event tracking
        app.add_event::<SelectionChanged>()
            .add_systems(Update, ManageSelection::handle_node_selected);

        // Create and select a node
        let entity = app.world_mut().spawn(()).id();
        let node_id = NodeIdentity::new();

        app.world_mut()
            .resource_mut::<Events<NodeSelected>>()
            .send(NodeSelected {
                entity,
                node: node_id,
                add_to_selection: false,
            });

        app.update();

        // Check that SelectionChanged event was fired
        let selection_changed_events = app.world().resource::<Events<SelectionChanged>>();
        assert!(selection_changed_events.len() > 0);
    }

    #[test]
    fn test_ray_sphere_intersection_with_scale() {
        // Test basic ray-sphere intersection
        let ray = Ray3d {
            origin: Vec3::new(0.0, 0.0, -5.0),
            direction: Dir3::new(Vec3::new(0.0, 0.0, 1.0)).unwrap(),
        };

        // Test with normal radius
        let distance = PerformRaycast::ray_intersects_sphere(&ray, Vec3::ZERO, 1.0);
        assert!(distance.is_some());
        assert!((distance.unwrap() - 4.0).abs() < 0.001);

        // Test with scaled radius (simulating node scale)
        let scaled_radius = 1.0 * 2.0; // Double scale
        let distance = PerformRaycast::ray_intersects_sphere(&ray, Vec3::ZERO, scaled_radius);
        assert!(distance.is_some());
        assert!((distance.unwrap() - 3.0).abs() < 0.001);

        // Test ray missing sphere
        let miss_ray = Ray3d {
            origin: Vec3::new(5.0, 0.0, -5.0),
            direction: Dir3::new(Vec3::new(0.0, 0.0, 1.0)).unwrap(),
        };
        let miss = PerformRaycast::ray_intersects_sphere(&miss_ray, Vec3::ZERO, 1.0);
        assert!(miss.is_none());
    }

    #[test]
    #[ignore] // TODO: Fix animated transforms integration
    fn test_selection_with_animated_transforms() {
        let mut app = setup_test_app();

        // Create node with animated transform
        let _node_entity = app
            .world_mut()
            .spawn((
                NodeIdentity::new(),
                Transform::from_xyz(0.0, 0.0, 0.0),
                GlobalTransform::default(),
                // AnimatedTransform {
                //     start: Transform::from_xyz(0.0, 0.0, 0.0),
                //     end: Transform::from_xyz(10.0, 0.0, 0.0),
                //     duration: 1.0,
                //     elapsed: 0.0,
                // },
            ))
            .id();
    }

    #[test]
    fn test_box_selection_with_scaled_nodes() {
        let mut app = setup_test_app();

        // Create a selection box
        let selection_box = SelectionBox::new(Vec2::new(100.0, 100.0));
        let mut selection_box = selection_box;
        selection_box.end = Vec2::new(200.0, 200.0);

        // Test with a scaled node (screen radius calculation)
        let scale = 2.0;
        let screen_radius = 20.0 * scale; // 40.0

        // Node at center of box should be selected
        let node_screen_pos = Vec2::new(150.0, 150.0);

        // Check bounds with scaled radius
        let (min, max) = selection_box.bounds();
        let node_min_x = node_screen_pos.x - screen_radius;
        let node_max_x = node_screen_pos.x + screen_radius;
        let node_min_y = node_screen_pos.y - screen_radius;
        let node_max_y = node_screen_pos.y + screen_radius;

        // Should intersect
        let intersects =
            !(node_max_x < min.x || node_min_x > max.x || node_max_y < min.y || node_min_y > max.y);
        assert!(intersects);

        // Node just outside the box with large scale should still intersect
        let edge_node_pos = Vec2::new(220.0, 150.0); // 20 pixels outside box
        let edge_node_min_x = edge_node_pos.x - screen_radius; // 180.0
        let edge_node_max_x = edge_node_pos.x + screen_radius; // 260.0

        let edge_intersects = !(edge_node_max_x < min.x
            || edge_node_min_x > max.x
            || node_max_y < min.y
            || node_min_y > max.y);
        assert!(edge_intersects); // Should still intersect due to large radius
    }

    #[test]
    fn test_selection_highlight_no_highlight_in_storage() {
        let _app = setup_test_app();
    }
}
