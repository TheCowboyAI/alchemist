use bevy::prelude::*;
use super::store::EventStore;
use super::events::DomainEventOccurred;
use super::replay::{handle_replay_requests, ReplayGraphRequest, ReplayFromCidRequest};
use crate::contexts::graph_management::capture_graph_events;

/// Plugin for the Event Store bounded context
/// Provides local event sourcing with Merkle DAG structure
pub struct EventStorePlugin;

impl Plugin for EventStorePlugin {
    fn build(&self, app: &mut App) {
        // Add the event store resource
        app.insert_resource(EventStore::new());

        // Add events
        app.add_event::<DomainEventOccurred>();
        app.add_event::<ReplayGraphRequest>();
        app.add_event::<ReplayFromCidRequest>();

        // Add systems
        app.add_systems(Update, (
            // Capture graph events and convert to domain events
            capture_graph_events,
            // Handle replay requests
            handle_replay_requests,
            // Handle domain events and log them
            handle_domain_events,
        ).chain());

        info!("Event Store plugin initialized with Merkle DAG support");
    }
}

/// System to debug the event store state
#[allow(dead_code)]
pub fn debug_event_store(
    event_store: Res<EventStore>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::F9) {
        match event_store.get_heads() {
            Ok(heads) => {
                info!("Event Store Heads: {:?}", heads);

                // Show some stats
                if let Ok(events) = event_store.events.read() {
                    info!("Total events in DAG: {}", events.len());
                }

                if let Ok(counter) = event_store.sequence_counter.read() {
                    info!("Current sequence: {}", *counter);
                }
            }
            Err(e) => {
                error!("Failed to get event store heads: {}", e);
            }
        }
    }
}

/// System to handle domain events and log them
fn handle_domain_events(
    mut events: EventReader<DomainEventOccurred>,
) {
    for event in events.read() {
        let domain_event = &event.0;
        info!(
            "📝 Domain Event: {} for aggregate {} (CID: {:?})",
            domain_event.event_type,
            domain_event.aggregate_id,
            domain_event.cid()
        );

        // The event is already stored in the EventStore by the adapter
        // This system is just for logging/monitoring
    }
}
