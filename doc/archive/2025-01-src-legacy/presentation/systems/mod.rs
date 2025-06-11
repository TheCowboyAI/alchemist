//! Presentation systems

pub mod camera_controller;
pub mod event_consumer_example;
pub mod graph_events;
pub mod graph_import_processor;
pub mod import_system;
pub mod subgraph_spatial_map;
pub mod subgraph_visualization;
pub mod voronoi_tessellation;
pub mod conceptual_visualization;
pub mod node_interaction;
pub mod context_bridge_visualization;
pub mod workflow_visualization;
pub mod subgraph_collapse_expand;
pub mod subgraph_drag_drop;
pub mod subgraph_merge_split;

pub use camera_controller::*;
pub use event_consumer_example::*;
pub use graph_events::*;
pub use graph_import_processor::*;
pub use import_system::{ImportPlugin, display_import_help, import_file_to_graph, ImportState};
pub use subgraph_spatial_map::*;
pub use subgraph_visualization::*;
pub use voronoi_tessellation::*;
pub use conceptual_visualization::*;
pub use node_interaction::*;
pub use context_bridge_visualization::*;
pub use workflow_visualization::*;

use bevy::prelude::*;
use crate::application::EventNotification;
use crate::presentation::events::{ImportResultEvent, ImportRequestEvent};
use crate::domain::events::{DomainEvent, GraphEvent};
use tracing::info;

/// System that forwards import requests from EventNotification to ImportRequestEvent
pub fn forward_import_requests(
    mut events: EventReader<EventNotification>,
    mut import_requests: EventWriter<ImportRequestEvent>,
) {
    let event_count = events.len();
    if event_count > 0 {
        info!("forward_import_requests: Processing {} EventNotifications", event_count);
    }

    for notification in events.read() {
        info!("Checking event for import request: {:?}", notification.event);
        if matches!(&notification.event, DomainEvent::Graph(GraphEvent::GraphImportRequested { .. })) {
            info!("Forwarding import request event");
            import_requests.write(ImportRequestEvent {
                event: notification.event.clone(),
            });
        }
    }
}

/// System that forwards import results to the main event stream
pub fn forward_import_results(
    mut import_events: EventReader<ImportResultEvent>,
    mut event_writer: EventWriter<EventNotification>,
) {
    for import_event in import_events.read() {
        event_writer.write(EventNotification {
            event: import_event.event.clone(),
        });
    }
}

pub struct SystemsPlugin;

impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                ImportPlugin,
                SubgraphSpatialMapPlugin,
            ))
            .add_systems(Update, (
                process_graph_import_requests,
                handle_node_added,
                handle_node_removed,
                handle_edge_added,
                handle_edge_removed,
            ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::domain::conceptual_graph::{
        ConceptGraph, ConceptNode, ConceptType, ConceptualPoint,
        QualityDimension, DimensionType, NodeId,
    };
    use crate::presentation::components::conceptual_visualization::{
        ConceptualNodeVisual, ConceptualSpaceVisual, SpaceId, SpaceBounds, GridSettings,
        DraggableNode, ConceptNodeType,
    };

    // Define test resource for cursor position
    #[derive(Resource, Default)]
    struct TestCursorPosition(Option<Vec2>);

    #[test]
    fn test_conceptual_visualization_system() {
        // Create a test app
        let mut app = App::new();

        // Add minimal plugins
        app.add_plugins(MinimalPlugins);

        // Add our visualization system
        app.add_systems(Update, visualize_conceptual_nodes);

        // Create a test concept graph
        let mut graph = ConceptGraph::new("TestGraph");

        // Add quality dimensions
        graph = graph
            .with_dimension(QualityDimension::new("abstraction", DimensionType::Continuous, 0.0..1.0))
            .with_dimension(QualityDimension::new("complexity", DimensionType::Continuous, 0.0..1.0))
            .with_dimension(QualityDimension::new("stability", DimensionType::Continuous, 0.0..1.0));

        // Add a test node
        let node = ConceptNode::Atom {
            id: NodeId::new(),
            concept_type: ConceptType::Entity,
            quality_position: ConceptualPoint::new(vec![0.5, 0.5, 0.5]),
            properties: Default::default(),
        };

        let node_id = node.id();
        graph.add_node(node);

        // Spawn the graph with visualization component
        app.world_mut().spawn((
            graph,
            ConceptualSpaceVisual {
                space_id: SpaceId::new(),
                dimensions: vec![],
                origin: Vec3::ZERO,
                bounds: SpaceBounds::default(),
                grid_settings: GridSettings::default(),
            },
        ));

        // Run one update cycle
        app.update();

        // Verify node was visualized
        let mut query = app.world_mut().query::<&ConceptualNodeVisual>();
        let visuals: Vec<_> = query.iter(&app.world()).collect();

        assert_eq!(visuals.len(), 1, "Should have created 1 visual node");
        assert_eq!(visuals[0].concept_id, node_id);
        assert_eq!(visuals[0].quality_position.coordinates(), &vec![0.5, 0.5, 0.5]);
    }

    #[test]
    fn test_node_visual_creation() {
        use crate::presentation::components::conceptual_visualization::{DraggableNode, ConceptNodeType};

        // Simple test that node visuals can be created
        let node_visual = ConceptualNodeVisual {
            concept_id: NodeId::new(),
            node_type: ConceptNodeType::Atom {
                category: "Entity".to_string(),
                properties: Default::default(),
            },
            quality_position: ConceptualPoint::new(vec![0.5, 0.5, 0.5]),
            visual_style: Default::default(),
            selected: false,
            hovered: false,
        };

        assert_eq!(node_visual.quality_position.coordinates(), &vec![0.5, 0.5, 0.5]);
        match &node_visual.node_type {
            ConceptNodeType::Atom { category, .. } => assert_eq!(category, "Entity"),
            _ => panic!("Wrong node type"),
        }
    }

    #[test]
    fn test_draggable_node_component() {
        use crate::presentation::components::conceptual_visualization::DraggableNode;

        let draggable = DraggableNode {
            is_dragging: false,
            drag_offset: Vec3::ZERO,
            constraints: Default::default(),
            snap_to_grid: true,
            grid_size: 1.0,
        };

        assert!(!draggable.is_dragging);
        assert!(draggable.snap_to_grid);
        assert_eq!(draggable.grid_size, 1.0);
    }
}
