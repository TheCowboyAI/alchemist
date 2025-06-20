# CIM Domain Modules

The Composable Information Machine is organized into 13 domain modules, each representing a bounded context with specific responsibilities.

## Core Domains

### [Graph Domain](./graph.md)
The foundation of CIM - everything is a graph.
- **Purpose**: Graph visualization and manipulation
- **Key Concepts**: Nodes, edges, layouts, traversal
- **Status**: ✅ Complete (41/41 tests)

### [Identity Domain](./identity.md)
Managing people and organizations.
- **Purpose**: Identity and relationship management
- **Key Concepts**: Person, organization, roles
- **Status**: ✅ Complete (54/54 tests)

### [Workflow Domain](./workflow.md)
Business process automation through visual workflows.
- **Purpose**: Define and execute business processes
- **Key Concepts**: Process, task, transition, execution
- **Status**: 🚧 In Progress

### [ConceptualSpaces Domain](./conceptualspaces.md)
Semantic knowledge representation.
- **Purpose**: AI-ready knowledge modeling
- **Key Concepts**: Concepts, dimensions, similarity
- **Status**: 🚧 In Progress

## Supporting Domains

### [Agent Domain](./agent.md)
AI agent integration and management.
- **Purpose**: Autonomous agent capabilities
- **Key Concepts**: Agent, capability, conversation
- **Status**: ✅ Complete (7/7 tests)

### [Person Domain](./person.md)
Extended person information management.
- **Purpose**: Contact and profile management
- **Key Concepts**: Contact info, preferences
- **Status**: ✅ Complete (2/2 tests)

### [Location Domain](./location.md)
Geographic and spatial information.
- **Purpose**: Location-based services
- **Key Concepts**: Address, coordinates, regions
- **Status**: 🚧 In Progress

### [Document Domain](./document.md)
Document processing and management.
- **Purpose**: Handle various document types
- **Key Concepts**: Document, content, metadata
- **Status**: 📋 Planned

## Technical Domains

### [Git Domain](./git.md)
Git repository integration.
- **Purpose**: Version control integration
- **Key Concepts**: Repository, commit, branch
- **Status**: ✅ Working

### [Nix Domain](./nix.md)
NixOS package and configuration management.
- **Purpose**: Reproducible system configuration
- **Key Concepts**: Package, derivation, flake
- **Status**: 📋 Planned

### [Policy Domain](./policy.md)
Business rules and policies.
- **Purpose**: Configurable business logic
- **Key Concepts**: Rule, condition, action
- **Status**: 📋 Planned

### [Dialog Domain](./dialog.md)
Conversational interfaces.
- **Purpose**: Natural language interaction
- **Key Concepts**: Conversation, intent, response
- **Status**: 📋 Planned

### [Organization Domain](./organization.md)
Organizational structure management.
- **Purpose**: Hierarchy and team management
- **Key Concepts**: Department, team, hierarchy
- **Status**: 📋 Planned

## Domain Communication

All domains communicate exclusively through events published on NATS:

```
┌─────────┐     events.person.*      ┌─────────┐
│ Person  │ -----------------------> │  Graph  │
│ Domain  │                          │ Domain  │
└─────────┘                          └─────────┘
     │                                    │
     │ events.agent.*                     │ events.graph.*
     ↓                                    ↓
┌─────────┐                          ┌─────────┐
│  Agent  │ <----------------------- │Workflow │
│ Domain  │   events.workflow.*      │ Domain  │
└─────────┘                          └─────────┘
```

## Event Patterns

Each domain follows consistent event patterns:

### Commands
- `cmd.{domain}.{action}` - e.g., `cmd.graph.create_node`

### Events
- `event.{domain}.{noun}_{verb}` - e.g., `event.graph.node_created`

### Queries
- `query.{domain}.{query_type}` - e.g., `query.graph.find_nodes`

## Development Status

| Domain           | Module                      | Tests   | Status        |
| ---------------- | --------------------------- | ------- | ------------- |
| Graph            | cim-domain-graph            | 41/41   | ✅ Complete    |
| Identity         | cim-domain-identity         | 54/54   | ✅ Complete    |
| Person           | cim-domain-person           | 2/2     | ✅ Complete    |
| Agent            | cim-domain-agent            | 7/7     | ✅ Complete    |
| Git              | cim-domain-git              | Working | ✅ Integration |
| Workflow         | cim-domain-workflow         | -       | 🚧 In Progress |
| ConceptualSpaces | cim-domain-conceptualspaces | -       | 🚧 In Progress |
| Location         | cim-domain-location         | -       | 🚧 In Progress |
| Document         | cim-domain-document         | -       | 📋 Planned     |
| Nix              | cim-domain-nix              | -       | 📋 Planned     |
| Policy           | cim-domain-policy           | -       | 📋 Planned     |
| Dialog           | cim-domain-dialog           | -       | 📋 Planned     |
| Organization     | cim-domain-organization     | -       | 📋 Planned     |

## Getting Started with Domains

1. **Choose a domain** relevant to your use case
2. **Review the domain documentation** for concepts and APIs
3. **Subscribe to domain events** for integration
4. **Send commands** to trigger domain behavior

For detailed implementation examples, see the [Domain Development Guide](../guides/domain-development.md). 