pub mod events;
pub mod persistence;
pub mod plugin;
pub mod replay;
pub mod store;

pub use events::{Cid, DomainEvent, DomainEventOccurred, EventPayload};
pub use persistence::EventPersistence;
pub use plugin::EventStorePlugin;
pub use replay::{EventReplayer, ReplayFromCidRequest, ReplayGraphRequest};
pub use store::{EventStore, ObjectStore};

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::SystemState;
    use bevy::prelude::*;
    use std::time::SystemTime;
    use uuid::Uuid;

    /// Test helper: Create a minimal world with event store
    fn setup_event_world() -> World {
        let mut world = World::new();
        world.init_resource::<EventStore>();
        world.init_resource::<Events<DomainEventOccurred>>();
        world
    }

    #[test]
    fn test_event_store_resource_initialization() {
        // Arrange
        let world = setup_event_world();

        // Act
        let event_store = world.resource::<EventStore>();

        // Assert
        let heads = event_store.get_heads().unwrap();
        assert_eq!(heads.len(), 0, "New event store should have no heads");
    }

    #[test]
    fn test_domain_event_system_integration() {
        // Arrange
        let mut world = setup_event_world();
        let aggregate_id = Uuid::new_v4();

        // Act - Create and send domain event
        {
            let event_store = world.resource::<EventStore>();
            let event = event_store
                .append_with_payload(
                    aggregate_id,
                    "TestEvent".to_string(),
                    EventPayload {
                        data: serde_json::json!({"test": true}),
                        created_at: SystemTime::now(),
                    },
                )
                .unwrap();

            world.send_event(DomainEventOccurred(event));
        }

        // Assert - Verify event can be read
        let mut system_state: SystemState<EventReader<DomainEventOccurred>> =
            SystemState::new(&mut world);
        let mut reader = system_state.get_mut(&mut world);

        let events: Vec<_> = reader.read().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0.event_type, "TestEvent");
    }

    #[test]
    fn test_event_ordering_determinism() {
        // Arrange
        let event_store = EventStore::new();
        let aggregate_id = Uuid::new_v4();

        // Act - Create multiple events
        let mut events = Vec::new();
        for i in 0..10 {
            let event = event_store
                .append_with_payload(
                    aggregate_id,
                    format!("Event{i}"),
                    EventPayload {
                        data: serde_json::json!({"index": i}),
                        created_at: SystemTime::now(),
                    },
                )
                .unwrap();
            events.push(event);
        }

        // Assert - Verify sequence numbers are sequential
        for (i, event) in events.iter().enumerate() {
            assert_eq!(event.sequence as usize, i + 1);
        }

        // Verify retrieval maintains order
        let retrieved = event_store.get_events_for_aggregate(aggregate_id).unwrap();
        for (i, event) in retrieved.iter().enumerate() {
            assert_eq!(event.sequence as usize, i + 1);
            assert_eq!(event.event_type, format!("Event{i}"));
        }
    }
}
