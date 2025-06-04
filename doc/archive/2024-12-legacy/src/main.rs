use bevy::prelude::*;

mod contexts;

#[cfg(test)]
mod testing;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        // Add our domain contexts
        .add_plugins((
            contexts::event_store::plugin::EventStorePlugin,
            contexts::graph_management::plugin::GraphManagementPlugin,
            contexts::visualization::plugin::VisualizationPlugin,
            contexts::selection::plugin::SelectionPlugin,
        ))
        // Load default graph on startup
        .add_systems(Startup, load_default_graph);

    app.run();
}

/// Loads the default graph from file
fn load_default_graph(
    mut commands: Commands,
    mut storage: ResMut<contexts::graph_management::storage::GraphStorage>,
    mut graph_created_events: EventWriter<contexts::graph_management::events::GraphCreated>,
    mut node_added_events: EventWriter<contexts::graph_management::events::NodeAdded>,
    mut edge_connected_events: EventWriter<contexts::graph_management::events::EdgeConnected>,
) {
    use contexts::graph_management::importer::GraphImporter;
    use std::path::Path;

    // Load the default graph file
    let graph_path = Path::new("assets/models/default_graph.json");

    info!("Loading default graph from: {}", graph_path.display());

    // Load the graph
    match GraphImporter::load_from_file(
        graph_path,
        &mut commands,
        &mut storage,
        &mut graph_created_events,
        &mut node_added_events,
        &mut edge_connected_events,
    ) {
        Ok(graph_id) => {
            info!("Successfully loaded default graph: {:?}", graph_id);
        }
        Err(e) => {
            error!("Failed to load default graph: {}", e);
            // Fall back to creating a simple test graph
            create_simple_test_graph(&mut commands);
        }
    }

    info!("========== KEYBOARD CONTROLS ==========");
    info!("Graph Layout:");
    info!("  L - Apply force-directed layout");
    info!("");
    info!("Visualization Modes:");
    info!("  C - Convert to point cloud");
    info!("  Ctrl+1 - Change nodes to Mesh mode");
    info!("  Ctrl+2 - Change nodes to Point Cloud mode");
    info!("  Ctrl+3 - Change nodes to Wireframe mode");
    info!("  Ctrl+4 - Change nodes to Billboard mode");
    info!("");
    info!("Edge Types:");
    info!("  1 - Line edges");
    info!("  2 - Cylinder edges");
    info!("  3 - Arc edges");
    info!("  4 - Bezier edges");
    info!("");
    info!("File Operations:");
    info!("  Ctrl+O - Load graph from file dialog");
    info!("");
    info!("Camera:");
    info!("  Drag - Rotate camera");
    info!("  Wheel - Zoom in/out");
    info!("======================================");
}

/// Creates a simple test graph as fallback
fn create_simple_test_graph(commands: &mut Commands) {
    use contexts::graph_management::domain::*;
    use std::collections::HashMap;

    let graph_id = GraphIdentity::new();
    let metadata = GraphMetadata {
        name: "Simple Test Graph".to_string(),
        description: "Fallback graph when default fails to load".to_string(),
        domain: "test".to_string(),
        created: std::time::SystemTime::now(),
        modified: std::time::SystemTime::now(),
        tags: vec!["test".to_string()],
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
        Visibility::default(),
    ));

    // Create 3 nodes in a triangle
    let positions = [
        Vec3::new(0.0, 0.0, -3.0),
        Vec3::new(-3.0, 0.0, 3.0),
        Vec3::new(3.0, 0.0, 3.0),
    ];

    let mut node_ids = Vec::new();

    for (i, &pos) in positions.iter().enumerate() {
        let node_id = NodeIdentity::new();
        node_ids.push(node_id);

        let content = NodeContent {
            label: format!("Node {}", i + 1),
            category: "test".to_string(),
            properties: HashMap::new(),
        };

        let position = SpatialPosition {
            coordinates_3d: pos,
            coordinates_2d: Vec2::new(pos.x, pos.z),
        };

        commands.spawn(NodeBundle {
            node: Node {
                identity: node_id,
                graph: graph_id,
                content: content.clone(),
                position,
            },
            identity: node_id,
            content,
            position,
            transform: Transform::from_translation(pos),
            global_transform: GlobalTransform::default(),
        });
    }

    // Connect all nodes
    for i in 0..3 {
        for j in (i + 1)..3 {
            let edge_id = EdgeIdentity::new();
            let relationship = EdgeRelationship {
                source: node_ids[i],
                target: node_ids[j],
                category: "connects".to_string(),
                strength: 1.0,
                properties: HashMap::new(),
            };

            commands.spawn(EdgeBundle {
                edge: Edge {
                    identity: edge_id,
                    graph: graph_id,
                    relationship: relationship.clone(),
                },
                identity: edge_id,
                relationship,
            });
        }
    }

    info!("Created simple test graph with 3 nodes and 3 edges");
}
