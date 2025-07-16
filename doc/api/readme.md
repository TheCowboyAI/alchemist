# CIM API Documentation

Welcome to the Composable Information Machine (CIM) API documentation. This comprehensive guide covers all 14 production-ready domains with their commands, events, queries, and integration patterns.

## 📚 Domain API Documentation

### Core Domains

1. **[Graph Domain](domains/graph-domain-api.md)** - Graph visualization and manipulation
   - 100 tests passing | Full CQRS implementation
   - Node/edge management, spatial mapping, workflow execution

2. **[Identity Domain](domains/identity-domain-api.md)** - Identity and authentication management
   - 66 tests passing | Person and organization management
   - Authentication, authorization, identity lifecycle

3. **[Workflow Domain](domains/workflow-domain-api.md)** - Business process management
   - 67 tests passing | State machine implementation
   - Process automation, approval workflows, parallel execution

4. **[ConceptualSpaces Domain](domains/conceptualspaces-domain-api.md)** - Semantic knowledge representation
   - 36 tests passing | AI reasoning capabilities
   - 5D conceptual dimensions, similarity search, semantic analysis

### Business Domains

5. **[Document Domain](domains/document-domain-api.md)** - Document lifecycle management
   - 65 tests passing | Content-addressed storage
   - Versioning, collaboration, templates, import/export

6. **[Agent Domain](domains/agent-domain-api.md)** - AI agent management
   - 8 tests passing | Provider abstraction
   - Capability management, tool integration, monitoring

7. **[Organization Domain](domains/organization-domain-api.md)** - Organizational structures
   - 63 tests passing | Hierarchical management
   - Roles, permissions, member management

8. **[Person Domain](domains/person-domain-api.md)** - Person profiles and relationships
   - 113 tests passing | Contact management
   - Skills, employment, network analysis

### Infrastructure Domains

9. **[Git Domain](domains/git-domain-api.md)** - Version control integration
   - 61 tests passing | Cross-domain integration
   - Repository management, commit tracking, branch operations

10. **[Location Domain](domains/location-domain-api.md)** - Geographic and spatial data
    - 39 tests passing | Spatial queries
    - Address management, geolocation, proximity search

11. **[Nix Domain](domains/nix-domain-api.md)** - Configuration management
    - 99 tests passing | Package management
    - Flake management, Home Manager integration

12. **[Policy Domain](domains/policy-domain-api.md)** - Business rules and policies
    - 54 tests passing | Rule enforcement
    - Security policies, compliance, validation

### UI/Interaction Domains

13. **[Bevy Domain](domains/bevy-domain-api.md)** - 3D visualization layer
    - 19 tests passing | ECS integration
    - Camera controls, animations, visual effects

14. **[Dialog Domain](domains/dialog-domain-api.md)** - Conversation management
    - 6 tests passing | Interaction tracking
    - Message handling, conversation state

## 🔧 Common Patterns

### Command Structure
All domains follow a consistent command pattern:
```rust
pub struct CreateEntity {
    pub entity_id: EntityId,
    pub metadata: HashMap<String, Value>,
    // Domain-specific fields
}
```

### Event Structure
Events follow the event sourcing pattern:
```rust
pub struct EntityCreated {
    pub entity_id: EntityId,
    pub created_at: SystemTime,
    pub created_by: UserId,
    // Domain-specific fields
}
```

### Query Structure
Queries support both simple lookups and complex searches:
```rust
pub struct GetEntity {
    pub entity_id: EntityId,
    pub include_relations: bool,
}

pub struct SearchEntities {
    pub query: String,
    pub filters: Vec<Filter>,
    pub pagination: Pagination,
}
```

## 🔗 Cross-Domain Integration

CIM domains communicate through:

1. **Event Choreography** - Domains react to events from other domains
2. **Command Orchestration** - Workflows coordinate multi-domain operations
3. **Shared Value Objects** - Common types like `UserId`, `Timestamp`
4. **NATS Messaging** - Distributed event bus for loose coupling

See [Cross-Domain Integration Patterns](../guides/cross-domain-integration.md) for detailed examples.

## 📖 Quick Start

1. Choose a domain from the list above
2. Review its commands, events, and queries
3. Check the integration examples
4. Implement your use case

## 🛠️ Tools and Utilities

- **Event Store Explorer** - Browse event history
- **Command Line Interface** - Execute commands via CLI
- **GraphQL Gateway** - Unified API access
- **WebSocket Subscriptions** - Real-time event streams

## 📊 API Statistics

- **Total Commands**: 150+
- **Total Events**: 180+
- **Total Queries**: 90+
- **Cross-Domain Integration Points**: 25+
- **Average Response Time**: <10ms

## 🚀 Getting Started

```bash
# Example: Create a graph with nodes
curl -X POST http://localhost:8080/api/graph/commands \
  -H "Content-Type: application/json" \
  -d '{
    "type": "CreateGraph",
    "payload": {
      "graph_id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "My Workflow",
      "graph_type": "WorkflowGraph"
    }
  }'
```

## 📝 API Conventions

- **IDs**: UUID v4 format
- **Timestamps**: ISO 8601 format
- **Errors**: RFC 7807 Problem Details
- **Pagination**: Cursor-based with limits
- **Versioning**: Header-based (X-API-Version)

## 🔒 Authentication & Authorization

All API endpoints require authentication:
- Bearer token in Authorization header
- API keys for service-to-service
- OAuth2 for third-party integrations

See [Authentication Guide](../guides/authentication.md) for details.

## 📚 Additional Resources

- [Developer Guide](../guides/developer-guide.md)
- [Event Sourcing Patterns](../guides/event-sourcing.md)
- [CQRS Implementation](../guides/cqrs-patterns.md)
- [Performance Optimization](../deployment/performance-optimization.md) 