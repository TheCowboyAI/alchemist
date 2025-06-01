#[cfg(test)]
mod tests {
    use super::super::services::*;
    use bevy::prelude::*;
    use bevy::math::primitives::*;

    /// Helper to setup test app with minimal requirements
    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_event::<NodeSelected>()
            .add_event::<NodeDeselected>()
            .add_event::<RenderModeChanged>()
            .add_event::<EdgeTypeChanged>()
            .add_event::<VisualizationUpdateRequested>()
            .add_event::<ConvertToPointCloud>()
            .insert_resource(CurrentVisualizationSettings::default());
        app
    }

    #[test]
    fn test_raycasting_sphere_intersection() {
        // Test ray-sphere intersection calculation
        let ray = Ray3d {
            origin: Vec3::new(0.0, 5.0, 0.0),
            direction: Direction3d::new_unchecked(Vec3::NEG_Y),
        };
        let sphere_center = Vec3::ZERO;
        let sphere_radius = 1.0;

        let hit = PerformRaycast::ray_intersects_sphere(&ray, sphere_center, sphere_radius);

        assert!(hit.is_some());
        assert!((hit.unwrap() - 4.0).abs() < 0.001); // Should hit at distance 4
    }

    #[test]
    fn test_raycasting_miss() {
        // Test ray missing sphere
        let ray = Ray3d {
            origin: Vec3::new(5.0, 0.0, 0.0),
            direction: Direction3d::new_unchecked(Vec3::Y),
        };
        let sphere_center = Vec3::ZERO;
        let sphere_radius = 1.0;

        let hit = PerformRaycast::ray_intersects_sphere(&ray, sphere_center, sphere_radius);

        assert!(hit.is_none());
    }

    #[test]
    fn test_render_mode_defaults() {
        let settings = CurrentVisualizationSettings::default();

        assert_eq!(settings.render_mode, RenderMode::Mesh);
        assert_eq!(settings.edge_type, EdgeType::Cylinder);
    }

    #[test]
    fn test_render_mode_change_event() {
        let mut app = setup_test_app();

        // Send render mode change event
        app.world_mut().send_event(RenderModeChanged {
            new_render_mode: RenderMode::PointCloud,
        });

        // Update the app
        app.update();

        // Read events
        let events = app.world().resource::<Events<RenderModeChanged>>();
        let mut reader = events.get_reader();
        let events_vec: Vec<_> = reader.read(events).collect();

        assert_eq!(events_vec.len(), 1);
        assert_eq!(events_vec[0].new_render_mode, RenderMode::PointCloud);
    }

    #[test]
    fn test_edge_type_change_event() {
        let mut app = setup_test_app();

        // Send edge type change event
        app.world_mut().send_event(EdgeTypeChanged {
            new_edge_type: EdgeType::Line,
        });

        // Update the app
        app.update();

        // Read events
        let events = app.world().resource::<Events<EdgeTypeChanged>>();
        let mut reader = events.get_reader();
        let events_vec: Vec<_> = reader.read(events).collect();

        assert_eq!(events_vec.len(), 1);
        assert_eq!(events_vec[0].new_edge_type, EdgeType::Line);
    }

    #[test]
    fn test_node_selection_event() {
        let mut app = setup_test_app();

        let entity = Entity::from_raw(42);
        let node_id = NodeIdentity::new();

        // Send selection event
        app.world_mut().send_event(NodeSelected {
            entity,
            node: node_id,
        });

        // Update the app
        app.update();

        // Verify event was sent
        let events = app.world().resource::<Events<NodeSelected>>();
        let mut reader = events.get_reader();
        let events_vec: Vec<_> = reader.read(events).collect();

        assert_eq!(events_vec.len(), 1);
        assert_eq!(events_vec[0].entity, entity);
        assert_eq!(events_vec[0].node, node_id);
    }

    #[test]
    fn test_visualization_capability_defaults() {
        let capability = VisualizationCapability::default();

        assert_eq!(capability.render_mode, RenderMode::Mesh);
        assert!(!capability.supports_instancing);
        assert!(capability.level_of_detail.is_none());
        assert!(capability.point_cloud_density.is_none());
    }

    #[test]
    fn test_point_cloud_component_creation() {
        let points = vec![Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z];
        let colors = vec![Color::WHITE, Color::srgb(1.0, 0.0, 0.0), Color::srgb(0.0, 1.0, 0.0), Color::srgb(0.0, 0.0, 1.0)];

        let node_cloud = NodePointCloud {
            points: points.clone(),
            colors: colors.clone(),
            point_size: 2.0,
            density: 1.0,
        };

        assert_eq!(node_cloud.points.len(), 4);
        assert_eq!(node_cloud.colors.len(), 4);
        assert_eq!(node_cloud.point_size, 2.0);
        assert_eq!(node_cloud.density, 1.0);
    }

    #[test]
    fn test_edge_point_cloud_component() {
        let points = vec![Vec3::ZERO, Vec3::X];
        let colors = vec![Color::WHITE; 2];

        let edge_cloud = EdgePointCloud {
            points: points.clone(),
            colors: colors.clone(),
            point_size: 1.0,
            density: 0.5,
        };

        assert_eq!(edge_cloud.points.len(), 2);
        assert_eq!(edge_cloud.colors.len(), 2);
        assert_eq!(edge_cloud.point_size, 1.0);
        assert_eq!(edge_cloud.density, 0.5);
    }

    #[test]
    fn test_convert_to_point_cloud_event() {
        let mut app = setup_test_app();

        let entity = Entity::from_raw(123);

        // Send conversion event
        app.world_mut().send_event(ConvertToPointCloud {
            entity,
            density: 10.0,
        });

        // Update the app
        app.update();

        // Verify event was sent
        let events = app.world().resource::<Events<ConvertToPointCloud>>();
        let mut reader = events.get_reader();
        let events_vec: Vec<_> = reader.read(events).collect();

        assert_eq!(events_vec.len(), 1);
        assert_eq!(events_vec[0].entity, entity);
        assert_eq!(events_vec[0].density, 10.0);
    }

    #[test]
    fn test_closest_hit_selection() {
        // Multiple hits, should select closest
        let mut hits = vec![
            (Entity::from_raw(1), 5.0),
            (Entity::from_raw(2), 2.0),
            (Entity::from_raw(3), 8.0),
        ];

        // Sort by distance
        hits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // Closest should be entity 2
        assert_eq!(hits[0].0, Entity::from_raw(2));
        assert_eq!(hits[0].1, 2.0);
    }
}
