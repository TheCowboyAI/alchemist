# Composable Information Machine (CIM) Documentation

Welcome to the CIM documentation. This guide provides a comprehensive overview of the Information Alchemist project - a revolutionary event-driven system for building composable information machines.

## 🚀 Current Status: v0.3.0

**Release Date**: January 21, 2025

### What's New
- **Event-Driven Testing Framework**: Comprehensive 5-layer testing approach
- **Critical Fix**: UI→NATS event publishing now working properly
- **31 Submodules**: All updated to v0.3.0 with consistent versioning
- **Testing Infrastructure**: EventStreamValidator for sequence validation

## 📚 Documentation Structure

### [Architecture](./architecture/)
Core architectural patterns and design decisions:
- **[CIM Overview](./architecture/cim-overview.md)** - Introduction to Composable Information Machines
- **[Event Sourcing](./architecture/event-sourcing.md)** - Event-driven architecture patterns
- **[Domain Model](./architecture/domain-model.md)** - DDD implementation details
- **[IPLD Integration](./architecture/ipld-overview.md)** - Content-addressed data structures

### [Domains](./domains/)
Detailed documentation for each domain module:
- **[Graph Domain](./domains/graph.md)** - Core graph visualization and manipulation
- **[Identity Domain](./domains/identity.md)** - Person and organization management
- **[Workflow Domain](./domains/workflow.md)** - Business process automation
- **[ConceptualSpaces Domain](./domains/conceptualspaces.md)** - Semantic knowledge representation
- [View all domains →](./domains/)

### [API Reference](./api/)
Technical API documentation:
- **[Event API](./api/events.md)** - Event types and structures
- **[Command API](./api/commands.md)** - Command patterns and handlers
- **[Query API](./api/queries.md)** - Query patterns and projections
- **[NATS Integration](./api/nats.md)** - Messaging patterns and subjects

### [Guides](./guides/)
Practical guides for developers:
- **[Getting Started](./guides/getting-started.md)** - Quick start guide
- **[Event-Driven Testing](./guides/event-driven-testing.md)** - Testing framework guide
- **[Domain Development](./guides/domain-development.md)** - Creating new domains
- **[Bevy Integration](./guides/bevy-integration.md)** - UI and visualization

### [Business Context](./business/)
Business-oriented documentation:
- **[Executive Summary](./business/executive-summary.md)** - High-level overview
- **[Use Cases](./business/use-cases.md)** - Real-world applications
- **[ROI Analysis](./business/roi-analysis.md)** - Business value proposition

### [Technical Details](./technical/)
Deep technical documentation:
- **[NATS Configuration](./technical/nats-setup.md)** - JetStream setup
- **[Performance Tuning](./technical/performance.md)** - Optimization guide
- **[Security Model](./technical/security.md)** - Authentication and authorization

## 🎯 Key Concepts

### Event-Driven Architecture
Every state change in CIM flows through immutable events:
```
Command → Aggregate → Event → Projection → Query
```

### Graph-Based Workflows
All domain models are fundamentally graphs that can be composed:
- Nodes represent entities or concepts
- Edges represent relationships
- Graphs can be nested and composed

### Conceptual Spaces
Semantic knowledge representation through geometric spaces:
- Points represent individual concepts
- Regions represent categories
- Distance represents similarity

## 🔧 Quick Links

- **[Glossary](./glossary.md)** - Domain terminology
- **[Vocabulary](./vocabulary.json)** - Complete term definitions
- **[Testing Plan](../testing/event-driven-testing-plan.md)** - Current testing initiative
- **[Progress Dashboard](../testing/testing-progress-dashboard.md)** - Implementation status

## 📦 Project Structure

```
alchemist/                    # Main application
├── cim-domain-*/            # Domain modules (13 domains)
├── cim-infrastructure/      # Core infrastructure
├── cim-bridge/             # Async/sync bridge
├── cim-ipld/               # Content addressing
├── cim-keys/               # Security and authentication
└── bevy-patched/           # Custom Bevy fork (v0.16.1)
```

## 🚦 Getting Started

1. **Explore the Architecture** - Start with [CIM Overview](./architecture/cim-overview.md)
2. **Understand the Domains** - Browse [domain documentation](./domains/)
3. **Try the Examples** - Check out the [guides](./guides/)
4. **Join Development** - See [contribution guidelines](../../CONTRIBUTING.md)

## 📊 Current Metrics

- **Domains**: 13 complete domain modules
- **Test Coverage**: Implementing across 31 submodules
- **Event Types**: 150+ domain events
- **Performance**: Sub-10ms event processing

## 🔗 External Resources

- [GitHub Repository](https://github.com/TheCowboyAI/alchemist)
- [NATS Documentation](https://docs.nats.io)
- [Bevy Engine](https://bevyengine.org)

---

*This documentation reflects the current state of CIM v0.3.0. For the latest updates, check the [progress tracking](../progress/progress.json).* 