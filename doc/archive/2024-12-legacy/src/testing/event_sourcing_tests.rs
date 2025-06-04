#[cfg(test)]
mod tests {
    use crate::contexts::event_store::{
        EventStore,
        events::{Cid, DomainEvent, DomainEventOccurred, EventPayload},
        plugin::EventStorePlugin,
        replay::{EventReplayer, ReplayGraphRequest, handle_replay_requests},
    };
    use crate::contexts::graph_management::domain::*;
    use crate::contexts::graph_management::events::*;
    use bevy::ecs::system::{RunSystemOnce, SystemState};
    use bevy::prelude::*;
    use std::time::SystemTime;
    use uuid::Uuid;

    /// Helper to create a test world with event store resources
    fn setup_test_world() -> World {
        let mut world = World::new();

        // Initialize resources
        world.init_resource::<EventStore>();
        world.init_resource::<Events<DomainEventOccurred>>();
        world.init_resource::<Events<GraphCreated>>();
        world.init_resource::<Events<NodeAdded>>();
        world.init_resource::<Events<EdgeConnected>>();
        world.init_resource::<Events<ReplayGraphRequest>>();

        world
    }

    #[test]
    fn test_event_store_basic_operations() {
        // Arrange
        let event_store = EventStore::new();
        let aggregate_id = Uuid::new_v4();

        // Act - Create a test payload
        let payload = EventPayload {
            data: serde_json::json!({
                "test": true,
                "value": 42
            }),
            created_at: SystemTime::now(),
        };

        let event = event_store
            .append_with_payload(aggregate_id, "TestEvent".to_string(), payload)
            .unwrap();

        // Assert
        assert_eq!(event.aggregate_id, aggregate_id);
        assert_eq!(event.event_type, "TestEvent");
        assert_eq!(event.sequence, 1);
        assert!(event.event_cid.is_some());

        // Verify retrieval
        let events = event_store.get_events_for_aggregate(aggregate_id).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, event.id);
    }

    #[test]
    #[ignore] // TODO: Fix event adapter integration
    fn test_event_capture_system() {
        // Arrange - Set up world with proper resources
        let mut world = setup_test_world();

        // Create test graph event
        let graph_id = GraphIdentity::new();
        let metadata = GraphMetadata {
            name: "Test Graph".to_string(),
            description: "Test Description".to_string(),
            domain: "test".to_string(),
            created: SystemTime::now(),
            modified: SystemTime::now(),
            tags: vec!["test".to_string()],
        };

        // Act - Send GraphCreated event
        world.send_event(GraphCreated {
            graph: graph_id,
            metadata,
            timestamp: SystemTime::now(),
        });

        // Run the capture system
        let mut system_state: SystemState<(
            Res<EventStore>,
            EventWriter<DomainEventOccurred>,
            EventReader<GraphCreated>,
            EventReader<NodeAdded>,
            EventReader<EdgeConnected>,
            EventReader<NodeRemoved>,
            EventReader<NodeMoved>,
        )> = SystemState::new(&mut world);

        let (
            event_store,
            mut domain_events,
            mut graph_created,
            _node_added,
            _edge_connected,
            _node_removed,
            _node_moved,
        ) = system_state.get_mut(&mut world);

        // Manually call the capture logic
        for event in graph_created.read() {
            match crate::contexts::graph_management::event_adapter::GraphEventAdapter::graph_created_to_domain_event(event, &*event_store) {
                Ok(domain_event) => {
                    domain_events.write(DomainEventOccurred(domain_event));
                }
                Err(e) => {
                    panic!("Failed to convert event: {e}");
                }
            }
        }

        system_state.apply(&mut world);

        // Assert - Check that domain event was created
        let mut event_reader_state: SystemState<EventReader<DomainEventOccurred>> =
            SystemState::new(&mut world);
        let mut domain_event_reader = event_reader_state.get_mut(&mut world);

        let domain_events: Vec<_> = domain_event_reader.read().collect();
        assert_eq!(domain_events.len(), 1);

        let domain_event = &domain_events[0].0;
        assert_eq!(domain_event.event_type, "GraphCreated");
        assert_eq!(domain_event.aggregate_id, graph_id.0);
    }

    #[test]
    fn test_merkle_dag_parent_linking() {
        // Arrange
        let event_store = EventStore::new();
        let graph_id = Uuid::new_v4();

        // Act - Create chain of events
        let event1 = event_store
            .append_with_payload(
                graph_id,
                "Event1".to_string(),
                EventPayload {
                    data: serde_json::json!({"event": 1}),
                    created_at: SystemTime::now(),
                },
            )
            .unwrap();

        let event2 = event_store
            .append_with_payload(
                graph_id,
                "Event2".to_string(),
                EventPayload {
                    data: serde_json::json!({"event": 2}),
                    created_at: SystemTime::now(),
                },
            )
            .unwrap();

        // Assert - Verify parent linking
        assert_eq!(
            event1.parent_cids.len(),
            0,
            "First event should have no parents"
        );
        assert_eq!(
            event2.parent_cids.len(),
            1,
            "Second event should have one parent"
        );
        assert_eq!(
            event2.parent_cids[0],
            event1.event_cid.unwrap(),
            "Second event should link to first"
        );
    }

    #[test]
    #[ignore] // TODO: Fix replay system integration
    fn test_event_replay_system() {
        // Arrange - Create world with event store
        let mut world = setup_test_world();
        let event_store = world.resource::<EventStore>();
        let graph_id = Uuid::new_v4();

        // Add test event to store
        let node_payload = EventPayload {
            data: serde_json::json!({
                "node_id": Uuid::new_v4().to_string(),
                "content": {
                    "label": "Test Node",
                    "category": "test",
                    "properties": {}
                },
                "position": {
                    "x": 1.0,
                    "y": 2.0,
                    "z": 3.0
                }
            }),
            created_at: SystemTime::now(),
        };

        event_store
            .append_with_payload(graph_id, "NodeAdded".to_string(), node_payload)
            .unwrap();

        // Act - Send replay request
        world.send_event(ReplayGraphRequest { graph_id });

        // Run replay system
        world.run_system_once(handle_replay_requests);

        // Assert - Check that node entity was created
        let mut query = world.query::<&NodeIdentity>();
        assert_eq!(
            query.iter(&world).count(),
            1,
            "One node should be created from replay"
        );
    }

    #[test]
    fn test_event_traversal() {
        // Arrange
        let event_store = EventStore::new();
        let graph_id = Uuid::new_v4();

        // Act - Create event chain
        let mut last_cid = None;
        for i in 0..5 {
            let event = event_store
                .append_with_payload(
                    graph_id,
                    format!("Event{i}"),
                    EventPayload {
                        data: serde_json::json!({"event": i}),
                        created_at: SystemTime::now(),
                    },
                )
                .unwrap();
            last_cid = event.event_cid;
        }

        // Assert - Traverse from last event
        let events = event_store.traverse_from(&last_cid.unwrap(), 10).unwrap();
        assert_eq!(events.len(), 5, "Should traverse all 5 events");

        // Verify correct ordering when sorted
        let mut sorted = events.clone();
        sorted.sort_by_key(|e| e.sequence);
        for (i, event) in sorted.iter().enumerate() {
            assert_eq!(event.event_type, format!("Event{i}"));
        }
    }

    #[test]
    fn test_cid_determinism() {
        // Arrange
        let content = b"test content";

        // Act
        let cid1 = Cid::from_content(content);
        let cid2 = Cid::from_content(content);

        // Assert - Same content produces same CID
        assert_eq!(cid1, cid2, "CID computation should be deterministic");

        // Different content produces different CID
        let different_content = b"different content";
        let cid3 = Cid::from_content(different_content);
        assert_ne!(
            cid1, cid3,
            "Different content should produce different CIDs"
        );
    }

    #[test]
    #[ignore] // TODO: Fix plugin initialization in tests
    fn test_event_store_plugin_initialization() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Act
        app.add_plugins(EventStorePlugin);
        app.update();

        // Assert - Check resources are initialized
        assert!(
            app.world().get_resource::<EventStore>().is_some(),
            "EventStore resource should be initialized"
        );

        // Check event is registered
        let events = app.world().resource::<Events<DomainEventOccurred>>();
        assert_eq!(
            events.len(),
            0,
            "DomainEventOccurred events should be registered"
        );
    }

    #[test]
    #[should_panic(expected = "Payload not found")]
    fn test_replay_missing_payload_error() {
        // Arrange
        let mut world = setup_test_world();
        let event_store = world.resource::<EventStore>().clone(); // Clone to avoid borrow issues

        // Create event with invalid payload reference
        let mut event = DomainEvent::new(
            Uuid::new_v4(),
            "TestEvent".to_string(),
            Cid::from_content(b"nonexistent"),
            vec![],
        );
        event.sequence = 1;
        event.compute_cid();

        // Act & Assert - Should panic when payload not found
        let mut commands = world.commands();
        EventReplayer::apply_event(&event, &event_store, &mut commands).expect("Payload not found");
    }
}

#[cfg(test)]
mod performance_tests {
    use crate::contexts::event_store::{EventStore, events::EventPayload};
    use std::time::SystemTime;
    use uuid::Uuid;

    // Note: Actual benchmarks would use #[bench] with nightly or criterion
    #[test]
    fn test_event_store_scaling() {
        // Arrange
        let event_store = EventStore::new();
        let aggregate_id = Uuid::new_v4();
        let event_count = 1000;

        // Act - Add many events
        let start = std::time::Instant::now();
        for i in 0..event_count {
            event_store
                .append_with_payload(
                    aggregate_id,
                    format!("Event{i}"),
                    EventPayload {
                        data: serde_json::json!({"index": i}),
                        created_at: SystemTime::now(),
                    },
                )
                .unwrap();
        }
        let duration = start.elapsed();

        // Assert - Performance check
        assert!(
            duration.as_millis() < 1000,
            "Adding 1000 events should take less than 1 second"
        );

        // Verify retrieval performance
        let start = std::time::Instant::now();
        let events = event_store.get_events_for_aggregate(aggregate_id).unwrap();
        let retrieval_duration = start.elapsed();

        assert_eq!(events.len(), event_count);
        assert!(
            retrieval_duration.as_millis() < 100,
            "Retrieving 1000 events should take less than 100ms"
        );
    }
}
