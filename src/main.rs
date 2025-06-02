use bevy::prelude::*;

mod contexts;

#[cfg(test)]
mod testing;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        // Add our domain contexts
        .add_plugins((
            contexts::graph_management::plugin::GraphManagementPlugin,
            contexts::visualization::plugin::VisualizationPlugin,
            contexts::selection::plugin::SelectionPlugin,
        ))
        // Add test data setup
        .add_systems(Startup, setup_test_graph);

    app.run();
}

/// Creates a test graph with nodes and edges for demonstration
fn setup_test_graph(mut commands: Commands) {
    use contexts::graph_management::domain::*;
    use std::collections::HashMap;

    // Create test graph
    let graph_id = GraphIdentity::new();
    let metadata = GraphMetadata {
        name: "Test Graph".to_string(),
        description: "Demo graph for layout testing".to_string(),
        domain: "test".to_string(),
        created: std::time::SystemTime::now(),
        modified: std::time::SystemTime::now(),
        tags: vec!["demo".to_string(), "layout-test".to_string()],
    };

    // Spawn graph entity
    commands.spawn((
        GraphBundle {
            graph: Graph {
                identity: graph_id,
                metadata: metadata.clone(),
                journey: GraphJourney::default(),
            },
            identity: graph_id,
            metadata,
            journey: GraphJourney::default(),
        },
        Transform::default(),
        GlobalTransform::default(),
    ));

    // Create nodes in a circle to start
    let node_count = 8;
    let radius = 5.0;
    let mut node_ids = Vec::new();

    for i in 0..node_count {
        let angle = (i as f32 / node_count as f32) * std::f32::consts::TAU;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;

        let node_id = NodeIdentity::new();
        node_ids.push(node_id);

        let content = NodeContent {
            label: format!("Node {}", i + 1),
            category: "test".to_string(),
            properties: HashMap::new(),
        };

        let position = SpatialPosition::at_3d(x, 0.0, z);

        // Spawn node entity
        commands.spawn((
            NodeBundle {
                node: Node {
                    identity: node_id,
                    graph: graph_id,
                    content: content.clone(),
                    position,
                },
                identity: node_id,
                content,
                position,
                transform: Transform::from_translation(position.coordinates_3d),
                global_transform: GlobalTransform::default(),
            },
        ));
    }

    // Create edges to form interesting connections
    let edge_pairs = vec![
        (0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (6, 7), (7, 0), // Outer ring
        (0, 4), (1, 5), (2, 6), (3, 7), // Cross connections
        (0, 2), (4, 6), // Additional connections
    ];

    for (i, (source_idx, target_idx)) in edge_pairs.iter().enumerate() {
        let edge_id = EdgeIdentity::new();
        let relationship = EdgeRelationship {
            source: node_ids[*source_idx],
            target: node_ids[*target_idx],
            category: "connects".to_string(),
            strength: 1.0,
            properties: HashMap::new(),
        };

        // Spawn edge entity
        commands.spawn((
            EdgeBundle {
                edge: Edge {
                    identity: edge_id,
                    graph: graph_id,
                    relationship: relationship.clone(),
                },
                identity: edge_id,
                relationship,
            },
        ));
    }

    info!("Test graph created with {} nodes and {} edges", node_count, edge_pairs.len());
    info!("========== KEYBOARD CONTROLS ==========");
    info!("Graph Layout:");
    info!("  L - Apply force-directed layout");
    info!("");
    info!("Visualization Modes:");
    info!("  C - Convert to point cloud");
    info!("  Ctrl+1 - Change nodes to Spheres");
    info!("  Ctrl+2 - Change nodes to Cubes");
    info!("  Ctrl+3 - Change nodes to Wireframes");
    info!("  Ctrl+4 - Change nodes to Point Clouds");
    info!("");
    info!("File Operations:");
    info!("  Ctrl+O - Load graph from assets/models/CIM.json");
    info!("");
    info!("Camera:");
    info!("  Drag - Rotate camera");
    info!("  Wheel - Zoom in/out");
    info!("======================================");
}
