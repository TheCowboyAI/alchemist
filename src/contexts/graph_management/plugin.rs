use crate::contexts::graph_management::domain::*;
use crate::contexts::graph_management::events::*;
use crate::contexts::graph_management::event_adapter::capture_graph_events;
use crate::contexts::graph_management::exporter::{
    ExportGraphEvent, GraphExportedEvent, display_export_feedback, handle_export_request,
    process_export_events,
};
use crate::contexts::graph_management::importer::import_graph_from_file;
use crate::contexts::graph_management::services::*;
use crate::contexts::graph_management::storage::*;
use bevy::prelude::*;

/// System sets for proper ordering
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GraphManagementSet {
    /// Systems that import or create graph data
    Import,
    /// Systems that export graph data
    Export,
    /// Systems that sync with storage
    Storage,
    /// Systems that organize hierarchy
    Hierarchy,
}

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
            .add_event::<InterSubgraphEdgeCreated>()
            // Export events
            .add_event::<ExportGraphEvent>()
            .add_event::<GraphExportedEvent>();

        // Register domain service systems
        app.add_systems(
            Update,
            (
                // Import system runs first
                import_graph_from_file,
                // Event capture for event sourcing
                capture_graph_events,
                // Export systems
                handle_export_request,
                process_export_events,
                display_export_feedback,
                // Storage sync systems
                SyncGraphWithStorage::sync_graph_created,
                SyncGraphWithStorage::sync_node_added,
                SyncGraphWithStorage::sync_edge_connected,
                // Hierarchy system
                EstablishGraphHierarchy::organize_hierarchy,
            ),
        );

        // Remove the startup system to avoid creating duplicate graphs
        // app.add_systems(Startup, create_example_graph);
    }
}

/// Creates an example graph (not used as startup system anymore)
#[allow(dead_code)]
fn create_example_graph(
    mut commands: Commands,
    mut graph_created: EventWriter<GraphCreated>,
    mut node_added: EventWriter<NodeAdded>,
    mut edge_connected: EventWriter<EdgeConnected>,
) {
    // Create graph
    let graph_id = GraphIdentity::new();
    let metadata = GraphMetadata {
        name: "Example Graph".to_string(),
        description: "A simple example graph".to_string(),
        domain: "example".to_string(),
        created: std::time::SystemTime::now(),
        modified: std::time::SystemTime::now(),
        tags: vec!["example".to_string(), "demo".to_string()],
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
            metadata: metadata.clone(),
            journey: GraphJourney::default(),
        },
        Transform::default(),
        GlobalTransform::default(),
    ));

    // Emit graph created event
    graph_created.write(GraphCreated {
        graph: graph_id,
        metadata,
        timestamp: std::time::SystemTime::now(),
    });

    // Create nodes
    let node1 = NodeIdentity::new();
    let node2 = NodeIdentity::new();
    let node3 = NodeIdentity::new();

    // Node 1
    node_added.write(NodeAdded {
        graph: graph_id,
        node: node1,
        content: NodeContent {
            label: "Node 1".to_string(),
            category: "default".to_string(),
            properties: Default::default(),
        },
        position: SpatialPosition::at_3d(-2.0, 0.0, 0.0),
    });

    // Node 2
    node_added.write(NodeAdded {
        graph: graph_id,
        node: node2,
        content: NodeContent {
            label: "Node 2".to_string(),
            category: "default".to_string(),
            properties: Default::default(),
        },
        position: SpatialPosition::at_3d(2.0, 0.0, 0.0),
    });

    // Node 3
    node_added.write(NodeAdded {
        graph: graph_id,
        node: node3,
        content: NodeContent {
            label: "Node 3".to_string(),
            category: "default".to_string(),
            properties: Default::default(),
        },
        position: SpatialPosition::at_3d(0.0, 2.0, 0.0),
    });

    // Create edges
    let edge1 = EdgeIdentity::new();
    let edge2 = EdgeIdentity::new();

    // Edge 1: Node 1 -> Node 2
    edge_connected.write(EdgeConnected {
        graph: graph_id,
        edge: edge1,
        relationship: EdgeRelationship {
            source: node1,
            target: node2,
            category: "connects".to_string(),
            strength: 1.0,
            properties: Default::default(),
        },
    });

    // Edge 2: Node 2 -> Node 3
    edge_connected.write(EdgeConnected {
        graph: graph_id,
        edge: edge2,
        relationship: EdgeRelationship {
            source: node2,
            target: node3,
            category: "connects".to_string(),
            strength: 1.0,
            properties: Default::default(),
        },
    });
}
