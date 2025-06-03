#[cfg(test)]
mod tests {
    use crate::contexts::event_store::{
        EventStore, EventPayload, DomainEvent, Cid,
        EventReplayer,
    };
    use crate::contexts::graph_management::domain::*;
    use bevy::prelude::*;
    use uuid::Uuid;
    use std::time::SystemTime;

    #[test]
    fn test_event_audit_trail() {
        // Create event store
        let event_store = EventStore::new();
        let graph_id = Uuid::new_v4();

        // Create a test event payload
        let payload = EventPayload {
            data: serde_json::json!({
                "test": true,
                "message": "Test event"
            }),
            created_at: SystemTime::now(),
        };

        // Store the event
        let event = event_store.append_with_payload(
            graph_id,
            "TestEvent".to_string(),
            payload,
        ).unwrap();

        // Verify event was stored
        assert_eq!(event.sequence, 1);
        assert_eq!(event.aggregate_id, graph_id);
        assert!(event.event_cid.is_some());

        // Verify we can retrieve events for the aggregate
        let events = event_store.get_events_for_aggregate(graph_id).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, event.id);
    }

    #[test]
    fn test_merkle_dag_structure() {
        let event_store = EventStore::new();
        let graph_id = Uuid::new_v4();

        // Create first event (no parents)
        let payload1 = EventPayload {
            data: serde_json::json!({"event": 1}),
            created_at: SystemTime::now(),
        };
        let event1 = event_store.append_with_payload(
            graph_id,
            "Event1".to_string(),
            payload1,
        ).unwrap();

        // Verify first event has no parents
        assert_eq!(event1.parent_cids.len(), 0);

        // Create second event (should have first as parent)
        let payload2 = EventPayload {
            data: serde_json::json!({"event": 2}),
            created_at: SystemTime::now(),
        };
        let event2 = event_store.append_with_payload(
            graph_id,
            "Event2".to_string(),
            payload2,
        ).unwrap();

        // Verify second event has first as parent
        assert_eq!(event2.parent_cids.len(), 1);
        assert_eq!(event2.parent_cids[0], event1.event_cid.unwrap());
    }

    #[test]
    fn test_event_replay() {
        let event_store = EventStore::new();
        let graph_id = Uuid::new_v4();

        // Create a node added event
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

        let event = event_store.append_with_payload(
            graph_id,
            "NodeAdded".to_string(),
            node_payload,
        ).unwrap();

        // Test replay
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut commands = app.world_mut().commands();
        EventReplayer::replay_graph(&event_store, graph_id, &mut commands).unwrap();

        // Apply commands
        app.update();

        // Verify node was created
        let query = app.world().query::<&NodeIdentity>();
        assert_eq!(query.iter(&app.world()).count(), 1);
    }

    #[test]
    fn test_cid_computation() {
        // Test that CID computation is deterministic
        let content = b"test content";
        let cid1 = Cid::from_content(content);
        let cid2 = Cid::from_content(content);
        assert_eq!(cid1, cid2);

        // Different content should produce different CIDs
        let different_content = b"different content";
        let cid3 = Cid::from_content(different_content);
        assert_ne!(cid1, cid3);
    }

    #[test]
    fn test_event_traversal() {
        let event_store = EventStore::new();
        let graph_id = Uuid::new_v4();

        // Create a chain of events
        let mut last_cid = None;
        for i in 0..5 {
            let payload = EventPayload {
                data: serde_json::json!({"event": i}),
                created_at: SystemTime::now(),
            };
            let event = event_store.append_with_payload(
                graph_id,
                format!("Event{}", i),
                payload,
            ).unwrap();
            last_cid = event.event_cid;
        }

        // Traverse from the last event
        let events = event_store.traverse_from(&last_cid.unwrap(), 10).unwrap();

        // Should get all 5 events
        assert_eq!(events.len(), 5);

        // Verify they're in the correct order when sorted by sequence
        let mut sorted = events.clone();
        sorted.sort_by_key(|e| e.sequence);
        for (i, event) in sorted.iter().enumerate() {
            assert_eq!(event.event_type, format!("Event{}", i));
        }
    }

    #[test]
    fn test_object_store() {
        let event_store = EventStore::new();

        // Store a payload
        let payload = EventPayload {
            data: serde_json::json!({"test": "data"}),
            created_at: SystemTime::now(),
        };

        let cid = event_store.object_store.put_payload(&payload).unwrap();

        // Retrieve it
        let retrieved = event_store.object_store.get_payload(&cid).unwrap();
        assert!(retrieved.is_some());

        let retrieved_payload = retrieved.unwrap();
        assert_eq!(retrieved_payload.data, payload.data);
    }
}
