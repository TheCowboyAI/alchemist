//! Presentation systems

pub mod import_system;
pub mod graph_import_processor;

pub use import_system::{ImportPlugin, ImportState, display_import_help, create_test_graph_on_startup};
pub use graph_import_processor::process_graph_import_requests;

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
