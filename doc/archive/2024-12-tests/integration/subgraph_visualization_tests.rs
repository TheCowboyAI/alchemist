//! Integration tests for subgraph visualization system

use bevy::prelude::*;
use ia::domain::value_objects::{NodeId, GraphId};
use ia::presentation::bevy_systems::{
    create_subgraph_origin, add_node_to_subgraph, move_subgraph,
    get_node_world_position, SubgraphSpatialMap, circular_layout,
};
use ia::presentation::bevy_systems::subgraph_visualization::{SubgraphOrigin, SubgraphMember};

#[test]
fn test_subgraph_creation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SubgraphSpatialMap>();

    app.update();

    // Create a subgraph origin
    let mut commands = app.world_mut().commands();
    let mut spatial_map = app.world_mut().resource_mut::<SubgraphSpatialMap>();
    let origin_entity = create_subgraph_origin(&mut commands, &mut spatial_map);

    app.update();

    // Verify origin was created
    let spatial_map = app.world().resource::<SubgraphSpatialMap>();
    assert_eq!(spatial_map.origins.len(), 1);
    assert!(spatial_map.positions.len() == 1);

    // Verify origin entity has correct components
    let origin = app.world().entity(origin_entity);
    assert!(origin.contains::<SubgraphOrigin>());
    assert!(origin.contains::<Transform>());
}

#[test]
fn test_add_nodes_to_subgraph() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SubgraphSpatialMap>();

    app.update();

    // Create a subgraph
    let graph_id = {
        let mut commands = app.world_mut().commands();
        let mut spatial_map = app.world_mut().resource_mut::<SubgraphSpatialMap>();
        let origin_entity = create_subgraph_origin(&mut commands, &mut spatial_map);

        // Get the graph ID
        spatial_map.origins.iter()
            .find(|(_, &entity)| entity == origin_entity)
            .map(|(id, _)| *id)
            .unwrap()
    };

    app.update();

    // Add nodes to the subgraph
    let node_ids: Vec<NodeId> = (0..3).map(|_| NodeId::new()).collect();
    let positions = vec![
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    ];

    for (i, (node_id, pos)) in node_ids.iter().zip(positions.iter()).enumerate() {
        let mut commands = app.world_mut().commands();
        let spatial_map = app.world().resource::<SubgraphSpatialMap>();
        add_node_to_subgraph(
            &mut commands,
            &spatial_map,
            graph_id,
            *node_id,
            *pos,
            format!("Node {}", i),
        );
    }

    app.update();

    // Verify nodes were created with correct components
    let mut node_count = 0;
    let mut query = app.world_mut().query::<&SubgraphMember>();
    for member in query.iter(&app.world()) {
        assert_eq!(member.graph_id, graph_id);
        node_count += 1;
    }
    assert_eq!(node_count, 3);
}

#[test]
fn test_move_subgraph() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SubgraphSpatialMap>();

    app.update();

    // Create a subgraph with nodes
    let (graph_id, node_entity) = {
        let mut commands = app.world_mut().commands();
        let mut spatial_map = app.world_mut().resource_mut::<SubgraphSpatialMap>();
        let origin_entity = create_subgraph_origin(&mut commands, &mut spatial_map);

        let graph_id = spatial_map.origins.iter()
            .find(|(_, &entity)| entity == origin_entity)
            .map(|(id, _)| *id)
            .unwrap();

        app.update();

        // Add a node
        let mut commands = app.world_mut().commands();
        let spatial_map = app.world().resource::<SubgraphSpatialMap>();
        let node_entity = add_node_to_subgraph(
            &mut commands,
            &spatial_map,
            graph_id,
            NodeId::new(),
            Vec3::new(5.0, 0.0, 0.0),
            "Test Node".to_string(),
        ).unwrap();

        (graph_id, node_entity)
    };

    app.update();

    // Get initial world position
    let initial_pos = {
        let query = app.world().query::<&GlobalTransform>();
        get_node_world_position(query, node_entity).unwrap()
    };
    assert_eq!(initial_pos, Vec3::new(5.0, 0.0, 0.0));

    // Move the subgraph
    {
        let mut spatial_map = app.world_mut().resource_mut::<SubgraphSpatialMap>();
        let mut query = app.world_mut().query::<&mut Transform>();
        move_subgraph(&mut spatial_map, &mut query, graph_id, Vec3::new(10.0, 0.0, 0.0));
    }

    app.update();

    // Verify the node moved with the subgraph
    let final_pos = {
        let query = app.world().query::<&GlobalTransform>();
        get_node_world_position(query, node_entity).unwrap()
    };
    assert_eq!(final_pos, Vec3::new(15.0, 0.0, 0.0)); // 10 + 5
}

#[test]
fn test_circular_layout_function() {
    let layout = circular_layout(10.0, 4);

    // Test positions
    let positions: Vec<Vec3> = (0..4).map(|i| layout(i)).collect();

    // All should be at radius 10
    for pos in &positions {
        assert!((pos.length() - 10.0).abs() < 0.001);
        assert_eq!(pos.y, 0.0);
    }

    // Should be evenly spaced
    let angle_between = 2.0 * std::f32::consts::PI / 4.0;
    for i in 0..4 {
        let next = (i + 1) % 4;
        let dot = positions[i].normalize().dot(positions[next].normalize());
        let expected_dot = angle_between.cos();
        assert!((dot - expected_dot).abs() < 0.001);
    }
}

#[test]
fn test_multiple_subgraphs() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SubgraphSpatialMap>();

    app.update();

    // Create multiple subgraphs
    let mut graph_ids = Vec::new();
    for _ in 0..3 {
        let mut commands = app.world_mut().commands();
        let mut spatial_map = app.world_mut().resource_mut::<SubgraphSpatialMap>();
        let origin_entity = create_subgraph_origin(&mut commands, &mut spatial_map);

        let graph_id = spatial_map.origins.iter()
            .find(|(_, &entity)| entity == origin_entity)
            .map(|(id, _)| *id)
            .unwrap();

        graph_ids.push(graph_id);
    }

    app.update();

    // Verify all subgraphs were created
    let spatial_map = app.world().resource::<SubgraphSpatialMap>();
    assert_eq!(spatial_map.origins.len(), 3);
    assert_eq!(spatial_map.positions.len(), 3);

    // Move each subgraph to different positions
    for (i, &graph_id) in graph_ids.iter().enumerate() {
        let position = Vec3::new(i as f32 * 10.0, 0.0, 0.0);
        let mut spatial_map = app.world_mut().resource_mut::<SubgraphSpatialMap>();
        let mut query = app.world_mut().query::<&mut Transform>();
        move_subgraph(&mut spatial_map, &mut query, graph_id, position);
    }

    app.update();

    // Verify positions
    let spatial_map = app.world().resource::<SubgraphSpatialMap>();
    for (i, &graph_id) in graph_ids.iter().enumerate() {
        let expected_pos = Vec3::new(i as f32 * 10.0, 0.0, 0.0);
        assert_eq!(spatial_map.positions[&graph_id], expected_pos);
    }
}
