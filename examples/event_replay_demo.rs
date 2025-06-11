//! # Event Replay Demo
//!
//! This demo showcases the power of event sourcing by demonstrating:
//! 1. Replaying events to rebuild state
//! 2. Collaboration subjects (multiple contributors)
//! 3. Aggregate subjects (single source of truth)
//! 4. Time travel to historical states
//! 5. CID verification for consistency

use ia::domain::{
    aggregates::content_graph::NodeContent,
    events::{
        DomainEvent,
        content_graph::{ContentGraphCreated, ContentAdded, RelationshipEstablished},
    },
    value_objects::{
        AggregateId, NodeId, EdgeId, GraphId,
        Position3D, RelatedBy,
    },
};
use cim_ipld::types::ContentType;
use chrono::{DateTime, Utc, Duration};
use colored::*;
use serde_json::json;
use std::collections::HashMap;
use std::time::SystemTime;

/// Represents an event with metadata for replay
#[derive(Debug, Clone)]
struct EventRecord {
    pub subject: String,
    pub event: DomainEvent,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub sequence: u64,
}

/// Event store for demonstration
struct DemoEventStore {
    events: Vec<EventRecord>,
}

impl DemoEventStore {
    fn new() -> Self {
        Self { events: Vec::new() }
    }

    fn append(&mut self, subject: String, event: DomainEvent, actor: String) {
        let sequence = self.events.len() as u64 + 1;
        self.events.push(EventRecord {
            subject,
            event,
            timestamp: Utc::now(),
            actor,
            sequence,
        });
    }

    fn get_by_subject_pattern(&self, pattern: &str) -> Vec<&EventRecord> {
        self.events
            .iter()
            .filter(|e| e.subject.starts_with(pattern))
            .collect()
    }

    fn get_up_to_time(&self, time: DateTime<Utc>) -> Vec<&EventRecord> {
        self.events
            .iter()
            .filter(|e| e.timestamp <= time)
            .collect()
    }

    fn get_up_to_sequence(&self, seq: u64) -> Vec<&EventRecord> {
        self.events
            .iter()
            .filter(|e| e.sequence <= seq)
            .collect()
    }
}

/// Simple graph state for demo
#[derive(Clone)]
struct GraphState {
    node_count: usize,
    edge_count: usize,
}

impl GraphState {
    fn new() -> Self {
        Self {
            node_count: 0,
            edge_count: 0,
        }
    }
}

fn main() {
    println!("{}", "=== Event Replay Demo ===".bright_cyan().bold());
    println!();

    // Create event store
    let mut event_store = DemoEventStore::new();

    // Create a content graph with collaboration pattern
    let graph_id = GraphId::new();
    let aggregate_id = AggregateId::from(graph_id.0);

    println!("{}", "1. Collaboration Pattern - Multiple Contributors".bright_yellow());
    println!("   Subject: events.collaboration.document.keco");
    println!();

    // Alice creates the initial graph
    simulate_collaboration_events(&mut event_store, &aggregate_id, &graph_id);

    // Show collaboration events
    println!("{}", "Collaboration Events:".bright_green());
    let collab_events = event_store.get_by_subject_pattern("events.collaboration.document.keco");
    for (i, event) in collab_events.iter().enumerate() {
        println!("  {}. [{}] {} by {}",
            i + 1,
            event.timestamp.format("%H:%M:%S"),
            format_event_type(&event.event).bright_blue(),
            event.actor.bright_magenta()
        );
    }
    println!();

    // Now show aggregate pattern
    println!("{}", "2. Aggregate Pattern - Single Source of Truth".bright_yellow());
    println!("   Subject: events.aggregate.contentgraph.{}", aggregate_id);
    println!();

    simulate_aggregate_events(&mut event_store, &aggregate_id, &graph_id);

    // Show aggregate events
    println!("{}", "Aggregate Events:".bright_green());
    let agg_events = event_store.get_by_subject_pattern(&format!("events.aggregate.contentgraph.{}", aggregate_id));
    for (i, event) in agg_events.iter().enumerate() {
        println!("  {}. [Seq: {}] {}",
            i + 1,
            event.sequence,
            format_event_type(&event.event).bright_blue()
        );
    }
    println!();

    // Demonstrate replay capabilities
    println!("{}", "3. Event Replay Demonstrations".bright_yellow());
    println!();

    // Replay all events
    println!("{}", "a) Full Replay - Rebuild Complete State".bright_green());
    let final_state = replay_all_events(&event_store);
    let final_cid = calculate_state_cid(&final_state);
    println!("   Final CID: {}", final_cid.bright_cyan());
    println!("   Nodes: {}, Edges: {}",
        final_state.node_count,
        final_state.edge_count
    );
    println!();

    // Time travel
    println!("{}", "b) Time Travel - State at Specific Time".bright_green());
    let time_point = Utc::now() - Duration::seconds(5);
    let historical_state = replay_up_to_time(&event_store, time_point);
    let historical_cid = calculate_state_cid(&historical_state);
    println!("   Historical CID: {}", historical_cid.bright_cyan());
    println!("   Nodes: {}, Edges: {}",
        historical_state.node_count,
        historical_state.edge_count
    );
    println!();

    // Selective replay
    println!("{}", "c) Selective Replay - Only Collaboration Events".bright_green());
    let collab_only = replay_by_subject(&event_store, "events.collaboration");
    let collab_cid = calculate_state_cid(&collab_only);
    println!("   Collaboration CID: {}", collab_cid.bright_cyan());
    println!();

    // Verify deterministic replay
    println!("{}", "4. Deterministic Replay Verification".bright_yellow());
    let replay1 = replay_all_events(&event_store);
    let replay2 = replay_all_events(&event_store);
    let cid1 = calculate_state_cid(&replay1);
    let cid2 = calculate_state_cid(&replay2);

    if cid1 == cid2 {
        println!("   ✓ Replay is deterministic: CIDs match!");
        println!("   CID: {}", cid1.bright_cyan());
    } else {
        println!("   ✗ Replay is NOT deterministic!");
    }
    println!();

    // Show event statistics
    println!("{}", "5. Event Statistics".bright_yellow());
    let mut event_counts: HashMap<String, usize> = HashMap::new();
    for event in &event_store.events {
        *event_counts.entry(format_event_type(&event.event)).or_insert(0) += 1;
    }

    for (event_type, count) in event_counts {
        println!("   {}: {}", event_type, count);
    }
    println!();

    // Demonstrate snapshot capability
    println!("{}", "6. Snapshot Demonstration".bright_yellow());
    let snapshot_point = 3;
    let snapshot = replay_up_to_sequence(&event_store, snapshot_point);
    let snapshot_cid = calculate_state_cid(&snapshot);
    println!("   Snapshot at sequence {}: CID = {}",
        snapshot_point,
        snapshot_cid.bright_cyan()
    );

    // Continue from snapshot
    let final_from_snapshot = replay_from_sequence(&event_store, snapshot.clone(), snapshot_point);
    let final_from_snapshot_cid = calculate_state_cid(&final_from_snapshot);
    println!("   Final from snapshot: CID = {}",
        final_from_snapshot_cid.bright_cyan()
    );

    if final_from_snapshot_cid == final_cid {
        println!("   ✓ Snapshot replay matches full replay!");
    }

    println!();
    println!("{}", "Key Insights:".bright_yellow());
    println!("   - Collaboration subjects allow multiple actors to contribute");
    println!("   - Aggregate subjects maintain single source of truth");
    println!("   - Event replay is deterministic when events are applied in order");
    println!("   - Snapshots enable efficient replay from checkpoints");
    println!("   - Subject patterns enable selective event filtering");
}

fn simulate_collaboration_events(store: &mut DemoEventStore, _aggregate_id: &AggregateId, graph_id: &GraphId) {
    // Alice creates the graph
    store.append(
        "events.collaboration.document.keco".to_string(),
        DomainEvent::ContentGraphCreated(ContentGraphCreated {
            graph_id: graph_id.clone(),
            created_at: SystemTime::now(),
        }),
        "Alice".to_string(),
    );

    // Bob adds content
    let node1 = NodeId::new();
    store.append(
        "events.collaboration.document.keco".to_string(),
        DomainEvent::ContentAdded(ContentAdded {
            graph_id: graph_id.clone(),
            node_id: node1.clone(),
            content: NodeContent::Value {
                content_type: ContentType::Json,
                data: json!({
                    "title": "Introduction",
                    "author": "Bob"
                }),
            },
            position: Position3D::new(0.0, 0.0, 0.0).unwrap(),
            metadata: HashMap::new(),
            content_cid: None,
        }),
        "Bob".to_string(),
    );

    // Carol adds more content
    let node2 = NodeId::new();
    store.append(
        "events.collaboration.document.keco".to_string(),
        DomainEvent::ContentAdded(ContentAdded {
            graph_id: graph_id.clone(),
            node_id: node2.clone(),
            content: NodeContent::Value {
                content_type: ContentType::Json,
                data: json!({
                    "title": "Chapter 1",
                    "author": "Carol"
                }),
            },
            position: Position3D::new(100.0, 0.0, 0.0).unwrap(),
            metadata: HashMap::new(),
            content_cid: None,
        }),
        "Carol".to_string(),
    );

    // Alice establishes relationship
    let edge1 = EdgeId::new();
    store.append(
        "events.collaboration.document.keco".to_string(),
        DomainEvent::RelationshipEstablished(RelationshipEstablished {
            graph_id: graph_id.clone(),
            edge_id: edge1,
            source: node1,
            target: node2,
            relationship: RelatedBy::Contains,
            strength: 1.0,
        }),
        "Alice".to_string(),
    );
}

fn simulate_aggregate_events(store: &mut DemoEventStore, aggregate_id: &AggregateId, graph_id: &GraphId) {
    let subject = format!("events.aggregate.contentgraph.{}", aggregate_id);

    // System events for the aggregate
    let node3 = NodeId::new();
    store.append(
        subject.clone(),
        DomainEvent::ContentAdded(ContentAdded {
            graph_id: graph_id.clone(),
            node_id: node3.clone(),
            content: NodeContent::Value {
                content_type: ContentType::Json,
                data: json!({
                    "type": "SystemMetadata",
                    "version": "1.0"
                }),
            },
            position: Position3D::new(0.0, 100.0, 0.0).unwrap(),
            metadata: HashMap::new(),
            content_cid: None,
        }),
        "System".to_string(),
    );

    let node4 = NodeId::new();
    store.append(
        subject.clone(),
        DomainEvent::ContentAdded(ContentAdded {
            graph_id: graph_id.clone(),
            node_id: node4.clone(),
            content: NodeContent::Value {
                content_type: ContentType::Json,
                data: json!({
                    "event": "DocumentPublished",
                    "timestamp": Utc::now().to_rfc3339()
                }),
            },
            position: Position3D::new(100.0, 100.0, 0.0).unwrap(),
            metadata: HashMap::new(),
            content_cid: None,
        }),
        "System".to_string(),
    );
}

fn replay_all_events(store: &DemoEventStore) -> GraphState {
    let mut state = GraphState::new();

    for event_record in &store.events {
        apply_event_to_state(&mut state, &event_record.event);
    }

    state
}

fn replay_up_to_time(store: &DemoEventStore, time: DateTime<Utc>) -> GraphState {
    let mut state = GraphState::new();

    let events = store.get_up_to_time(time);
    for event_record in events {
        apply_event_to_state(&mut state, &event_record.event);
    }

    state
}

fn replay_up_to_sequence(store: &DemoEventStore, seq: u64) -> GraphState {
    let mut state = GraphState::new();

    let events = store.get_up_to_sequence(seq);
    for event_record in events {
        apply_event_to_state(&mut state, &event_record.event);
    }

    state
}

fn replay_from_sequence(store: &DemoEventStore, mut state: GraphState, from_seq: u64) -> GraphState {
    for event_record in &store.events {
        if event_record.sequence > from_seq {
            apply_event_to_state(&mut state, &event_record.event);
        }
    }

    state
}

fn replay_by_subject(store: &DemoEventStore, pattern: &str) -> GraphState {
    let mut state = GraphState::new();

    let events = store.get_by_subject_pattern(pattern);
    for event_record in events {
        apply_event_to_state(&mut state, &event_record.event);
    }

    state
}

fn apply_event_to_state(state: &mut GraphState, event: &DomainEvent) {
    // In a real implementation, this would use the aggregate's apply_event method
    // For demo purposes, we'll just count nodes and edges
    match event {
        DomainEvent::ContentGraphCreated(_) => {
            // Graph already created
        }
        DomainEvent::ContentAdded(_) => {
            state.node_count += 1;
        }
        DomainEvent::RelationshipEstablished(_) => {
            state.edge_count += 1;
        }
        _ => {} // Handle other events as needed
    }
}

fn calculate_state_cid(state: &GraphState) -> String {
    // For demo purposes, create a simple hash based on node/edge count
    format!("bafyrei{}nodes{}edges", state.node_count, state.edge_count)
}

fn format_event_type(event: &DomainEvent) -> String {
    event.event_type().to_string()
}
