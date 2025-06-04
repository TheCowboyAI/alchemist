# EventStream Transaction Example: Document Workflow

This example demonstrates how NATS EventStream transactions work in practice with a document approval workflow. Each event includes a CID calculated using IPLD dag-cbor format with the previous CID, creating a verifiable chain.

## Scenario

A document goes through multiple stages:
1. **Draft** - Initial creation
2. **Review** - Multiple reviewers provide feedback
3. **Approval** - Manager approves
4. **Published** - Final state

Each stage generates events that modify the workflow graph.

## Event Stream Structure

### Events Generated

```rust
// Event 1: Document Created (first in chain, no previous CID)
DomainEvent {
    event_id: "evt_001",
    aggregate_id: "doc_123",
    event_type: "DocumentCreated",
    sequence: 1001,
    timestamp: "2024-01-15T10:00:00Z",
    event_cid: Cid::from("bafyreigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"),
    previous_cid: None, // First event in chain
    payload: {
        "document_id": "doc_123",
        "title": "Q1 Budget Report",
        "author": "person_456",
        "department": "org_finance"
    }
}

// Event 2: Review Requested (chains from event 1)
DomainEvent {
    event_id: "evt_002",
    aggregate_id: "doc_123",
    event_type: "ReviewRequested",
    sequence: 1002,
    timestamp: "2024-01-15T10:05:00Z",
    event_cid: Cid::from("bafyreigmhbx5ixaqt5p2oapgmvqw5lxr4uwslmlojvmm2ke2hqj2xpqr2a"),
    previous_cid: Some(Cid::from("bafyreigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi")),
    payload: {
        "document_id": "doc_123",
        "reviewers": ["person_789", "person_012"],
        "due_date": "2024-01-17T17:00:00Z"
    }
}

// Event 3: Review Completed (chains from event 2)
DomainEvent {
    event_id: "evt_003",
    aggregate_id: "doc_123",
    event_type: "ReviewCompleted",
    sequence: 1003,
    timestamp: "2024-01-16T14:30:00Z",
    event_cid: Cid::from("bafyreihpc3vupfos5yqnlakgpjxtyx3smkg26ft7e2jnqf3qkyhheqzx5a"),
    previous_cid: Some(Cid::from("bafyreigmhbx5ixaqt5p2oapgmvqw5lxr4uwslmlojvmm2ke2hqj2xpqr2a")),
    payload: {
        "document_id": "doc_123",
        "reviewer": "person_789",
        "status": "approved_with_comments",
        "comments": ["Consider adding Q4 comparison"]
    }
}

// Event 4: Document Revised (chains from event 3)
DomainEvent {
    event_id: "evt_004",
    aggregate_id: "doc_123",
    event_type: "DocumentRevised",
    sequence: 1004,
    timestamp: "2024-01-16T15:00:00Z",
    event_cid: Cid::from("bafyreifepxmvljnlomosqixqzb3rkxnbwvwkzapr3g2kgj3mzxwjkavnue"),
    previous_cid: Some(Cid::from("bafyreihpc3vupfos5yqnlakgpjxtyx3smkg26ft7e2jnqf3qkyhheqzx5a")),
    payload: {
        "document_id": "doc_123",
        "revision": 2,
        "changes": ["Added Q4 comparison section"]
    }
}

// Event 5: Approval Requested (chains from event 4)
DomainEvent {
    event_id: "evt_005",
    aggregate_id: "doc_123",
    event_type: "ApprovalRequested",
    sequence: 1005,
    timestamp: "2024-01-16T15:30:00Z",
    event_cid: Cid::from("bafyreigvhzij4g3nzh5wfpsjn2x3gxlb6mzqpjlqvg2lqzqhqvmfmxhfza"),
    previous_cid: Some(Cid::from("bafyreifepxmvljnlomosqixqzb3rkxnbwvwkzapr3g2kgj3mzxwjkavnue")),
    payload: {
        "document_id": "doc_123",
        "approver": "person_345",
        "level": "department_head"
    }
}
```

### CID Chain Verification

```rust
// The event service verifies the CID chain when fetching
impl EventStreamService {
    async fn verify_event_chain(&self, events: &[DomainEvent]) -> Result<(), ChainError> {
        let mut expected_previous: Option<Cid> = None;

        for event in events {
            // Verify previous CID matches
            if event.previous_cid != expected_previous {
                return Err(ChainError::BrokenChain {
                    event_cid: event.event_cid.clone(),
                    expected_previous,
                    actual_previous: event.previous_cid.clone(),
                });
            }

            // Recalculate CID to verify integrity
            let calculated_cid = calculate_event_cid(
                &event.payload,
                event.previous_cid.clone(),
                &event.aggregate_id,
                &event.event_type,
            )?;

            if calculated_cid != event.event_cid {
                return Err(ChainError::InvalidCid {
                    event_cid: event.event_cid.clone(),
                    calculated_cid,
                });
            }

            // Update expected previous for next event
            expected_previous = Some(event.event_cid.clone());
        }

        Ok(())
    }
}
```

## Fetching as Transaction

### 1. Fetch Complete History

```rust
// Fetch all events for this document as a transaction
let transaction = event_service.fetch_transaction(
    "doc_123".into(),
    TransactionOptions {
        replay_policy: ReplayPolicy::FromBeginning,
        max_events: None,
        ..Default::default()
    },
).await?;

// The service automatically verifies the CID chain
event_service.verify_event_chain(&transaction.events)?;

// Result:
EventStreamTransaction {
    transaction_id: "txn_abc123",
    sequence_range: SequenceRange {
        start: 1001,
        end: 1005,
        stream_name: "event-store",
    },
    aggregate_id: "doc_123",
    events: vec![evt_001, evt_002, evt_003, evt_004, evt_005],
    metadata: TransactionMetadata {
        fetched_at: "2024-01-17T09:00:00Z",
        consumer_name: "workflow-processor",
        filter_subject: Some("event.store.doc_123.*"),
        replay_policy: ReplayPolicy::FromBeginning,
    },
}
```

### 2. Fetch Recent Changes Only

```rust
// Fetch only events after the last processed sequence
let transaction = event_service.fetch_transaction(
    "doc_123".into(),
    TransactionOptions {
        replay_policy: ReplayPolicy::AfterSequence(1003),
        max_events: Some(10),
        ..Default::default()
    },
).await?;

// Result: Only events 1004 and 1005
EventStreamTransaction {
    transaction_id: "txn_def456",
    sequence_range: SequenceRange {
        start: 1004,
        end: 1005,
        stream_name: "event-store",
    },
    aggregate_id: "doc_123",
    events: vec![evt_004, evt_005],
    // ...
}
```

## Graph Mutations from Events

### Event to Graph Mutation Conversion

```rust
// The system converts each event to graph mutations
match event.event_type.as_str() {
    "DocumentCreated" => GraphMutation::AddNode {
        node_id: "node_doc_123",
        node_type: NodeType::Document,
        properties: hashmap! {
            "title" => "Q1 Budget Report",
            "status" => "draft",
            "author" => "person_456",
        },
    },

    "ReviewRequested" => vec![
        GraphMutation::AddNode {
            node_id: "node_review_789",
            node_type: NodeType::ReviewTask,
            properties: hashmap! {
                "reviewer" => "person_789",
                "due_date" => "2024-01-17T17:00:00Z",
            },
        },
        GraphMutation::AddEdge {
            edge_id: "edge_doc_to_review",
            source: "node_doc_123",
            target: "node_review_789",
            edge_type: EdgeType::RequiresReview,
            properties: hashmap! {},
        },
    ],

    "ReviewCompleted" => GraphMutation::UpdateNode {
        node_id: "node_review_789",
        updates: hashmap! {
            "status" => "completed",
            "result" => "approved_with_comments",
        },
    },

    "DocumentRevised" => GraphMutation::Transform {
        target: TransformTarget::Node("node_doc_123"),
        operation: TransformOperation::UpdateProperties {
            properties: hashmap! {
                "revision" => 2,
                "status" => "revised",
            },
        },
    },

    "ApprovalRequested" => vec![
        GraphMutation::AddNode {
            node_id: "node_approval_345",
            node_type: NodeType::ApprovalTask,
            properties: hashmap! {
                "approver" => "person_345",
                "level" => "department_head",
            },
        },
        GraphMutation::AddEdge {
            edge_id: "edge_doc_to_approval",
            source: "node_doc_123",
            target: "node_approval_345",
            edge_type: EdgeType::RequiresApproval,
            properties: hashmap! {},
        },
    ],
}
```

## Bevy Visualization Updates

### Real-time Processing

```rust
// System that processes transactions and updates visualization
pub fn document_workflow_visualization_system(
    mut graph_events: EventReader<GraphMutationEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &GraphNode)>,
) {
    for event in graph_events.read() {
        match &event.mutation {
            GraphMutation::AddNode { node_id, node_type, properties } => {
                // Spawn visual representation
                let color = match node_type {
                    NodeType::Document => Color::BLUE,
                    NodeType::ReviewTask => Color::YELLOW,
                    NodeType::ApprovalTask => Color::GREEN,
                    _ => Color::GRAY,
                };

                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(Sphere::new(0.5)),
                        material: materials.add(StandardMaterial {
                            base_color: color,
                            ..default()
                        }),
                        transform: Transform::from_xyz(
                            // Position based on workflow stage
                            match node_type {
                                NodeType::Document => 0.0,
                                NodeType::ReviewTask => -2.0,
                                NodeType::ApprovalTask => 2.0,
                                _ => 0.0,
                            },
                            0.0,
                            0.0,
                        ),
                        ..default()
                    },
                    GraphNode {
                        node_id: node_id.clone(),
                        graph_index: NodeIndex::new(0), // Set by graph model
                    },
                    // Add interaction components
                    PickableBundle::default(),
                    RaycastPickTarget::default(),
                ));
            }

            GraphMutation::AddEdge { source, target, edge_type, .. } => {
                // Find source and target entities
                let source_entity = query.iter()
                    .find(|(_, node)| node.node_id == *source)
                    .map(|(e, _)| e);

                let target_entity = query.iter()
                    .find(|(_, node)| node.node_id == *target)
                    .map(|(e, _)| e);

                if let (Some(source_e), Some(target_e)) = (source_entity, target_entity) {
                    // Create edge visualization
                    spawn_edge_visual(
                        &mut commands,
                        source_e,
                        target_e,
                        edge_type,
                    );
                }
            }

            GraphMutation::UpdateNode { node_id, updates } => {
                // Update visual properties
                if let Some((entity, _)) = query.iter()
                    .find(|(_, node)| node.node_id == *node_id) {

                    // Change color based on status
                    if let Some(status) = updates.get("status") {
                        let new_color = match status.as_str() {
                            Some("completed") => Color::GREEN,
                            Some("revised") => Color::ORANGE,
                            _ => Color::GRAY,
                        };

                        commands.entity(entity).insert(
                            materials.add(StandardMaterial {
                                base_color: new_color,
                                ..default()
                            })
                        );
                    }
                }
            }

            _ => {}
        }
    }
}
```

## Transaction Replay

### Animated Replay of Workflow

```rust
// Start replay of document workflow
pub fn start_workflow_replay(
    mut commands: Commands,
    transaction_cache: Res<TransactionCache>,
) {
    if let Some(transaction) = transaction_cache.get("txn_abc123") {
        commands.spawn((
            TransactionReplay {
                transaction_id: transaction.transaction_id.clone(),
                current_event: 0,
                total_events: transaction.events.len(),
                replay_speed: 1.0, // Real-time speed
                timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            },
            // UI to show replay progress
            TextBundle::from_section(
                "Replaying: 0/5 events",
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
        ));
    }
}

// System that animates the replay
pub fn animate_replay_system(
    time: Res<Time>,
    mut replay_query: Query<(&mut TransactionReplay, &mut Text)>,
    transaction_cache: Res<TransactionCache>,
    mut graph_events: EventWriter<GraphMutationEvent>,
) {
    for (mut replay, mut text) in replay_query.iter_mut() {
        replay.timer.tick(time.delta());

        if replay.timer.finished() && replay.current_event < replay.total_events {
            if let Some(transaction) = transaction_cache.get(&replay.transaction_id) {
                if let Some(event) = transaction.events.get(replay.current_event) {
                    // Send event for processing
                    if let Some(mutation) = convert_to_graph_mutation(event) {
                        graph_events.send(GraphMutationEvent {
                            source: EventSource::Replay {
                                original_sequence: event.sequence,
                            },
                            mutation,
                            transaction_id: Some(replay.transaction_id.clone()),
                        });
                    }

                    replay.current_event += 1;
                    text.sections[0].value = format!(
                        "Replaying: {}/{} events - {}",
                        replay.current_event,
                        replay.total_events,
                        event.event_type
                    );

                    replay.timer.reset();
                }
            }
        }
    }
}
```

## Real-time Subscription

### Subscribe to Live Updates

```rust
// Subscribe to document events
pub async fn subscribe_to_document_events(
    subscription_manager: Res<NatsSubscriptionManager>,
    document_id: &str,
) {
    // Subscribe to all events for this document
    subscription_manager.subscribe(
        format!("event.store.{}.>", document_id),
        None,
        SubscriptionHandler::GraphUpdate {
            target_graph: GraphId::from(document_id),
        },
    ).await.unwrap();

    // Also subscribe to related person events
    subscription_manager.subscribe(
        "event.store.person.*.assigned".to_string(),
        Some(EventFilter::Custom(Box::new(move |event| {
            // Filter for events that reference our document
            event.payload.get("document_id")
                .and_then(|v| v.as_str())
                .map(|id| id == document_id)
                .unwrap_or(false)
        }))),
        SubscriptionHandler::Buffered {
            max_size: 100,
            max_age: Duration::from_secs(5),
        },
    ).await.unwrap();
}
```

## Complete Flow

1. **Initial Load**: Fetch historical events as transaction
2. **Build Graph**: Convert events to graph structure
3. **Render**: Create Bevy entities for visualization
4. **Subscribe**: Set up real-time subscriptions
5. **Update**: Process incoming events as graph mutations
6. **Interact**: User can replay, pause, or explore the graph

This example shows how EventStream transactions provide a coherent view of related events that can be efficiently processed, replayed, and visualized in Bevy.
