# Phase 0: NATS Integration Foundation

## Overview

**Duration**: Week 1
**Status**: 🚧 In Progress
**Started**: 2025-06-04
**Target**: 2025-06-11

This phase establishes the foundation for Information Alchemist as a CIM leaf node by implementing NATS connectivity, security, and event bridging.

## Objectives

1. **Connect to CIM Cluster**: Establish secure NATS connection
2. **Event Bridge**: Create bidirectional event flow between NATS and Bevy
3. **Subject Structure**: Implement CIM-compliant subject naming
4. **Security**: Configure JWT authentication and TLS

## Task Breakdown

### 0.1 NATS Client Setup ⏳

**Goal**: Configure async NATS client with JetStream support

**Subtasks**:
- [ ] Add `async-nats` dependency to Cargo.toml
- [ ] Create `infrastructure/nats/mod.rs` module structure
- [ ] Implement `NatsIntegration` struct with client and JetStream context
- [ ] Add connection configuration with retry logic
- [ ] Create connection health monitoring
- [ ] Write integration tests with test NATS server

**Code Location**: `src/infrastructure/nats/client.rs`

### 0.2 Security Configuration ⏳

**Goal**: Implement secure connection with JWT and TLS

**Subtasks**:
- [ ] Create `SecurityConfig` struct for credentials
- [ ] Implement JWT authentication flow
- [ ] Configure TLS for encrypted connections
- [ ] Add credential file support
- [ ] Implement NKEY seed handling
- [ ] Create security tests

**Code Location**: `src/infrastructure/nats/security.rs`

### 0.3 Event Bridge Architecture ⏳

**Goal**: Bridge between NATS messages and Bevy ECS events

**Subtasks**:
- [ ] Create subject constants following CIM conventions
- [ ] Implement NATS → Bevy event converter
- [ ] Implement Bevy → NATS command publisher
- [ ] Add subscription management
- [ ] Handle backpressure and buffering
- [ ] Create bridge system for Bevy app

**Code Location**: `src/infrastructure/nats/event_bridge.rs`

## Dependencies

```toml
[dependencies]
async-nats = "0.35"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Architecture Decisions

### Subject Naming Convention

Following CIM standards:
```
graph.commands.create
graph.events.created
node.commands.add
node.events.added
edge.commands.connect
edge.events.connected
```

### Event Flow

```
User Action → Bevy Event → Command → NATS Subject → CIM Backend
                                           ↓
                                    JetStream Persistence
                                           ↓
CIM Backend → Domain Event → NATS Subject → Event Bridge → Bevy System
```

### Security Model

- JWT tokens for authentication
- TLS for all connections
- Credential files in `~/.config/cim/`
- Automatic token refresh

## Testing Strategy

1. **Unit Tests**: Mock NATS client for isolated testing
2. **Integration Tests**: Use test NATS server in Docker
3. **Security Tests**: Verify auth flows and encryption
4. **Performance Tests**: Measure latency and throughput

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| NATS server unavailable | High | Implement offline mode with queuing |
| Network latency | Medium | Local caching and batching |
| Security misconfiguration | High | Comprehensive security tests |
| Event ordering | Medium | Use JetStream for guaranteed ordering |

## Success Criteria

- [ ] Successfully connect to NATS server
- [ ] Publish and receive test messages
- [ ] Security authentication working
- [ ] Event bridge converting messages bidirectionally
- [ ] All tests passing
- [ ] Documentation complete

## Daily Progress

### Day 1 (2025-06-04)
- ✅ Architecture design completed
- ✅ Phase 0 added to implementation plan
- ✅ Progress tracking updated
- 🚧 Starting NATS client implementation

### Day 2 (2025-06-05)
- [ ] Complete NATS client setup
- [ ] Begin security configuration

### Day 3 (2025-06-06)
- [ ] Complete security implementation
- [ ] Start event bridge

### Day 4 (2025-06-07)
- [ ] Complete event bridge
- [ ] Integration testing

### Day 5 (2025-06-08)
- [ ] Performance testing
- [ ] Documentation
- [ ] Phase completion

## Code Examples

### NATS Client Setup
```rust
pub struct NatsIntegration {
    client: async_nats::Client,
    jetstream: async_nats::jetstream::Context,
    subscriptions: HashMap<String, Subscription>,
}

impl NatsIntegration {
    pub async fn connect(config: ConnectionConfig) -> Result<Self> {
        let client = async_nats::connect_with_options(
            config.url,
            config.options
        ).await?;

        let jetstream = async_nats::jetstream::new(client.clone());

        Ok(Self {
            client,
            jetstream,
            subscriptions: HashMap::new(),
        })
    }
}
```

### Event Bridge
```rust
pub fn nats_event_bridge(
    nats: Res<NatsIntegration>,
    mut events: EventWriter<DomainEventOccurred>,
) {
    // Poll NATS for new messages
    if let Some(msg) = nats.try_next() {
        if let Ok(event) = serde_json::from_slice(&msg.payload) {
            events.send(DomainEventOccurred(event));
        }
    }
}
```

## Next Phase Preview

**Phase 1: Distributed Event Infrastructure**
- JetStream configuration for event persistence
- Object store client implementation
- Content addressing system
- Distributed caching layer

---

*This document is updated daily during the phase. Upon completion, it will be summarized and archived.*
