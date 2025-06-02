use crate::contexts::graph_management::domain::*;
use crate::contexts::graph_management::events::*;
use crate::contexts::graph_management::services::*;
use crate::contexts::graph_management::storage::*;
use bevy::prelude::*;

/// Plugin for the Graph Management bounded context
pub struct GraphManagementPlugin;

impl Plugin for GraphManagementPlugin {
    fn build(&self, app: &mut App) {
        // Register resources
        app.insert_resource(GraphStorage::new());

        // Register events
        app.add_event::<GraphCreated>()
            .add_event::<NodeAdded>()
            .add_event::<EdgeConnected>()
            .add_event::<NodeRemoved>()
            .add_event::<EdgeDisconnected>()
            .add_event::<GraphDeleted>()
            .add_event::<SubgraphExtracted>()
            .add_event::<InterSubgraphEdgeCreated>();

        // Register domain service systems
        app.add_systems(
            Update,
            (
                // Storage sync systems
                SyncGraphWithStorage::sync_graph_created,
                SyncGraphWithStorage::sync_node_added,
                SyncGraphWithStorage::sync_edge_connected,

                // Hierarchy system
                EstablishGraphHierarchy::organize_hierarchy,
            ),
        );

        // Add startup system to create example
        app.add_systems(Startup, create_example_graph);
    }
}

/// Creates an example graph on startup
fn create_example_graph(
    mut commands: Commands,
    mut graph_created: EventWriter<GraphCreated>,
    mut node_added: EventWriter<NodeAdded>,
    mut edge_connected: EventWriter<EdgeConnected>,
) {
    // Create a new graph using our DDD service
    let metadata = GraphMetadata {
        name: "Technology Graph".to_string(),
        description: "Example graph showing technology relationships".to_string(),
        domain: "technology".to_string(),
        created: std::time::SystemTime::now(),
        modified: std::time::SystemTime::now(),
        tags: vec!["example".to_string(), "demo".to_string()],
    };

    let graph_id = CreateGraph::execute(metadata, &mut commands, &mut graph_created);

    // Add nodes using our service
    let rust_node = AddNodeToGraph::execute(
        graph_id,
        NodeContent {
            label: "Rust".to_string(),
            category: "Language".to_string(),
            properties: Default::default(),
        },
        SpatialPosition::at_3d(-2.0, 0.0, 0.0),
        &mut commands,
        &mut node_added,
    );

    let bevy_node = AddNodeToGraph::execute(
        graph_id,
        NodeContent {
            label: "Bevy".to_string(),
            category: "Framework".to_string(),
            properties: Default::default(),
        },
        SpatialPosition::at_3d(2.0, 0.0, 0.0),
        &mut commands,
        &mut node_added,
    );

    let ecs_node = AddNodeToGraph::execute(
        graph_id,
        NodeContent {
            label: "ECS".to_string(),
            category: "Pattern".to_string(),
            properties: Default::default(),
        },
        SpatialPosition::at_3d(0.0, 2.0, 0.0),
        &mut commands,
        &mut node_added,
    );

    // Connect nodes
    ConnectGraphNodes::execute(
        graph_id,
        rust_node,
        bevy_node,
        "powers".to_string(),
        1.0,
        &mut commands,
        &mut edge_connected,
    );

    ConnectGraphNodes::execute(
        graph_id,
        bevy_node,
        ecs_node,
        "implements".to_string(),
        1.0,
        &mut commands,
        &mut edge_connected,
    );

    info!("Example graph created with DDD-compliant code!");
}
