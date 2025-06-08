//! Presentation systems

pub mod camera_controller;
pub mod event_consumer_example;
pub mod graph_events;
pub mod graph_import_processor;
pub mod import_system;
pub mod subgraph_spatial_map;
pub mod subgraph_visualization;
pub mod voronoi_tessellation;

pub use camera_controller::*;
pub use event_consumer_example::*;
pub use graph_events::*;
pub use graph_import_processor::*;
pub use import_system::{ImportPlugin, display_import_help, import_file_to_graph, ImportState};
pub use subgraph_spatial_map::*;
pub use subgraph_visualization::*;
pub use voronoi_tessellation::*;

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
