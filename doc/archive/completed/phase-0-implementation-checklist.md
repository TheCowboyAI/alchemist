# Phase 0: NATS Integration Foundation - Implementation Checklist

## Overview
Phase 0 establishes the foundational NATS integration for CIM, creating the event-driven backbone for all future phases.

## Prerequisites
- [x] Architecture documentation complete
- [x] Rule system in place
- [x] Basic project structure
- [x] Bevy application framework

## Implementation Tasks

### 1. NATS Client Setup
- [ ] Add NATS dependencies to Cargo.toml
  - `async-nats = "0.35"`
  - `nats = "0.25"` (for sync operations if needed)
- [ ] Create NATS configuration module
  - [ ] Connection settings (host, port, credentials)
  - [ ] JetStream configuration
  - [ ] Security settings (TLS, auth)
- [ ] Implement NATS client wrapper
  - [ ] Connection management
  - [ ] Reconnection logic
  - [ ] Health checks

### 2. Event Store Foundation
- [ ] Define core event types
  - [ ] `DomainEvent` structure with CID support
  - [ ] Event metadata (timestamps, correlation IDs)
  - [ ] Event serialization/deserialization
- [ ] Implement EventStore trait
  ```rust
  #[async_trait]
  pub trait EventStore {
      async fn append_events(&self, events: Vec<DomainEvent>) -> Result<()>;
      async fn get_events(&self, aggregate_id: AggregateId) -> Result<Vec<DomainEvent>>;
      async fn get_events_from(&self, position: EventPosition) -> Result<EventStream>;
  }
  ```
- [ ] Create NATS JetStream implementation
  - [ ] Stream creation and configuration
  - [ ] Event publishing with deduplication
  - [ ] Event consumption with acknowledgment

### 3. Async/Sync Bridge
- [ ] Design bridge architecture
  - [ ] Command channel (Bevy → NATS)
  - [ ] Event channel (NATS → Bevy)
  - [ ] Buffering and batching strategies
- [ ] Implement AsyncSyncBridge
  ```rust
  pub struct AsyncSyncBridge {
      command_sender: Sender<Command>,
      event_receiver: Receiver<Event>,
      runtime: Handle,
  }
  ```
- [ ] Create Bevy systems for bridge
  - [ ] Command sending system
  - [ ] Event polling system
  - [ ] Error handling and logging

### 4. Basic Domain Implementation
- [ ] Create first aggregate (Graph)
  ```rust
  pub struct GraphAggregate {
      id: GraphId,
      version: u64,
      last_event_cid: Option<Cid>,
  }
  ```
- [ ] Implement command handlers
  - [ ] CreateGraph command
  - [ ] AddNode command (basic)
- [ ] Define initial events
  - [ ] GraphCreated
  - [ ] NodeAdded

### 5. Testing Infrastructure
- [ ] Unit tests for event store
  - [ ] Event serialization
  - [ ] CID chain verification
  - [ ] Stream operations
- [ ] Integration tests
  - [ ] NATS connection
  - [ ] Event round-trip
  - [ ] Bridge functionality
- [ ] Test utilities
  - [ ] In-memory event store
  - [ ] Test event builders
  - [ ] Assertion helpers

### 6. Development Environment
- [ ] Update flake.nix
  - [ ] Add NATS server
  - [ ] Configure test environment
- [ ] Create docker-compose.yml
  - [ ] NATS server with JetStream
  - [ ] Development configuration
- [ ] Add development scripts
  - [ ] Start NATS server
  - [ ] Run integration tests
  - [ ] Monitor event streams

## Success Criteria
- [ ] Can connect to NATS server
- [ ] Can publish and consume events
- [ ] Bevy can send commands through bridge
- [ ] Events from NATS update Bevy ECS
- [ ] All tests passing
- [ ] Basic graph creation working end-to-end

## Next Phase Dependencies
This phase enables:
- Phase 1: Distributed Event Infrastructure
- Phase 2: Domain Model with CIM Extensions
- All subsequent phases build on this foundation
