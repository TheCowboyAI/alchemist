# Graph Domain Design Documentation

## Overview

This directory contains the complete DDD-compliant design for the Information Alchemist Graph system. All documents follow strict Domain-Driven Design principles using pure business language.

## Design Documents

### 📋 [graph-domain-design.md](./graph-domain-design.md)
**The Authoritative Domain Model**

Our complete domain model specification:
- **Aggregates**: Graph, GraphView, GraphAnalysis
- **Entities**: Node, Edge
- **Value Objects**: Identities, metadata, relationships
- **Domain Events**: Past-tense facts (GraphCreated, NodeAdded)
- **Domain Services**: Verb phrases (CreateGraph, FindGraphPaths)
- **Storage**: Plural collections (Graphs, Nodes, Edges)
- **Bounded Contexts**: Clear separation of concerns

### 📊 [graph-current-state-analysis.md](./graph-current-state-analysis.md)
**Implementation Status**

Analysis of current vs. target state:
- What's already implemented
- What needs to be built
- Migration requirements
- Risk assessment

### 🗺️ [graph-implementation-roadmap.md](./graph-implementation-roadmap.md)
**Development Plan**

Step-by-step implementation guide:
- 8-week sprint schedule
- Deliverables per phase
- Technical milestones
- Success metrics

## DDD Compliance Rules

### ✅ Naming Conventions

| Element | Rule | Example |
|---------|------|---------|
| **Events** | Past-tense facts, no "Event" suffix | `GraphCreated` ✅ ~~`GraphCreatedEvent`~~ ❌ |
| **Services** | Verb phrases revealing intent | `CreateGraph` ✅ ~~`GraphService`~~ ❌ |
| **Storage** | Plural domain terms | `Graphs` ✅ ~~`GraphRepository`~~ ❌ |
| **Entities** | Singular business nouns | `Node` ✅ ~~`NodeEntity`~~ ❌ |
| **Value Objects** | Descriptive nouns | `GraphIdentity` ✅ ~~`GraphId`~~ ❌ |

### 🎯 Domain Language Examples

```rust
// ✅ CORRECT - Pure domain language
pub struct GraphCreated { ... }      // Event
pub struct CreateGraph;              // Service
pub struct Graphs;                   // Storage
pub struct GraphMotion { ... }       // Component

// ❌ INCORRECT - Technical suffixes
pub struct GraphCreatedEvent { ... } // Avoid "Event"
pub struct GraphManager;             // Avoid "Manager"
pub struct GraphRepository;          // Avoid "Repository"
pub struct GraphAnimation { ... }    // Avoid technical terms
```

### 🔄 Event Flow

```
User Action → Command → Domain Service → Domain Event → Event Store
                              ↓
                         Update Aggregate
                              ↓
                         Publish Event → Other Contexts React
```

## Bounded Contexts

### Core Domain
- **Graph Management**: Graph lifecycle and structure

### Supporting Domains
- **Visualization**: Display and interaction
- **Analysis**: Graph algorithms and metrics
- **Import/Export**: Format transformations
- **Animation**: Motion and transitions

### Generic Subdomain
- **Collaboration**: Multi-user editing

## Quick Reference

### Domain Vocabulary
- **Graph**: Collection of nodes and edges with identity
- **Node**: Vertex with content and position
- **Edge**: Connection between nodes
- **View**: Perspective on a graph
- **Layout**: Arrangement of nodes in space
- **Path**: Route between nodes
- **Motion**: Animation of graph elements

### Service Patterns
```rust
// Create operations
CreateGraph, AddNodeToGraph, ConnectGraphNodes

// Query operations
FindGraphPaths, CalculateGraphMetrics, AnalyzeGraph

// Transform operations
ApplyGraphLayout, ImportGraphData, ExportGraphData

// Interaction operations
TrackNodeSelection, AnimateGraphElements, ControlGraphCamera
```

## Getting Started

1. **Read** [graph-domain-design.md](./graph-domain-design.md) to understand the domain model
2. **Review** [graph-current-state-analysis.md](./graph-current-state-analysis.md) for implementation status
3. **Follow** [graph-implementation-roadmap.md](./graph-implementation-roadmap.md) for development steps

## Compliance Verification

All code must comply with:
- `.cursor/rules/ddd.mdc` - DDD naming conventions
- `.cursor/rules/rust.mdc` - Rust patterns
- `.cursor/rules/bevy_ecs.mdc` - ECS best practices

The design documents in this directory are 100% DDD-compliant and serve as the reference for all implementation.
