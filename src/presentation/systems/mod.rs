//! Presentation systems

pub mod import_system;
pub mod graph_import_processor;
pub mod camera_controller;
pub mod subgraph_visualization;
pub mod voronoi_tessellation;

pub use import_system::{ImportPlugin, display_import_help, import_file_to_graph};
pub use graph_import_processor::process_graph_import_requests;
pub use camera_controller::*;
pub use subgraph_visualization::*;

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
