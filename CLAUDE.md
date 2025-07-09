# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

You **MUST** read doc/progress/progress.json
This contains our Project Memory with comprehensive tracking of:
- All 14 domains (100% complete with 468 passing tests)
- Architecture health metrics (100% event-driven, CQRS, domain isolation)
- Recent achievements and milestones
- Technical debt status (162 clippy warnings, down from 739)
- Current focus areas and next priorities
- Business impact metrics (40% time savings demonstrated)

ALL Work must create a node in the progress graph structure and update metrics accordingly.

## Project Overview

**Alchemist** is a revolutionary 3D graph visualization and editing system that serves as the primary interface for the **Composable Information Machine (CIM)**. It's built in Rust using Bevy Engine with a strict event-driven architecture and 14+ fully implemented domains.

## Development Commands

### Environment Setup
```bash
# NixOS with direnv (required)
direnv allow
nix develop
```

### Build & Test
```bash
# Build entire workspace
nix build

# Run all tests (499+ tests)
nix develop -c cargo test

# Run specific test
cargo test <test_name>

# Run performance benchmarks
cargo test --release -- --nocapture bench_

# Run demos
cargo run --bin workflow_demo
cargo run --bin state_machine_demo
cargo run --bin workflow_demo_simple
```

## Architecture Overview

### Core Design Principles
- **Event-Driven Architecture**: Everything is an event, zero CRUD operations
- **Domain-Driven Design**: 14+ domains with clear boundaries
- **CQRS Pattern**: Separate read/write models with eventual consistency
- **ECS Integration**: Bevy Entity Component System for 3D visualization
- **Distributed Systems**: NATS messaging with JetStream persistence

### Technology Stack
- **Language**: Rust (Nightly 2024+)
- **Runtime**: Tokio async runtime
- **Visualization**: Bevy Engine 0.16+ (patched version)
- **Messaging**: NATS with JetStream
- **Build System**: Cargo workspace with 23+ crates

### Domain Structure
The system is organized into 14 production-ready domains:

1. **Graph Domain** (`cim-domain-graph`) - Core graph operations, spatial positioning
2. **Agent Domain** (`cim-domain-agent`) - AI provider integration, semantic search
3. **Workflow Domain** (`cim-domain-workflow`) - Business process execution, state machines
4. **ConceptualSpaces Domain** (`cim-domain-conceptual-spaces`) - Semantic reasoning, quality dimensions
5. **Identity Domain** (`cim-domain-identity`) - Person/organization management
6. **Location Domain** (`cim-domain-location`) - Geospatial data, hierarchical locations
7. **Document Domain** (`cim-domain-document`) - Document lifecycle, version control
8. **Git Domain** (`cim-domain-git`) - Repository integration, commit tracking
9. **Dialog Domain** (`cim-domain-dialog`) - Conversation tracking
10. **Policy Domain** (`cim-domain-policy`) - Business rule enforcement
11. **Nix Domain** (`cim-domain-nix`) - Infrastructure as code
12. **Organization Domain** (`cim-domain-organization`) - Hierarchy management
13. **Person Domain** (`cim-domain-person`) - Contact management, relationships
14. **Bevy Domain** (`cim-domain-bevy`) - ECS visualization layer

### Key Technical Innovations
- **CID Chains**: Cryptographic integrity for events
- **Dual ECS Systems**: Domain logic + visualization
- **Conceptual Spaces**: 5D semantic reasoning engine
- **Self-Referential**: System can visualize itself
- **Event Sourcing**: Perfect audit trail, time-travel debugging

## Development Rules (from .cursorrules)

### **MANDATORY TDD WORKFLOW**
You **MUST** follow this exact sequence for ALL development work:

1. **Write User Stories** - Clear, testable requirements
2. **Expand to Tests** - Convert user stories into failing tests
3. **Fulfill Tests** - Implement code to make tests pass
4. **Update Progress** - Track completion in progress.json

**NO CODE IS WRITTEN WITHOUT TESTS FIRST**

### TDD Process Example
```rust
// 1. User Story: "As a user, I want to create a graph node"
// 2. Test First:
#[test]
fn test_create_graph_node() {
    let mut graph = Graph::new();
    let node_id = graph.create_node("test_node");
    assert!(graph.contains_node(&node_id));
}
// 3. Then implement Graph::create_node() to make test pass
```

### Naming Conventions
- **Filenames**: ALL lowercase with underscores (snake_case)
- **Functions**: snake_case
- **Types**: PascalCase
- **Constants**: SCREAMING_SNAKE_CASE

### Architecture Patterns
- **Event-First Design**: Everything must be an event
- **Domain Isolation**: No shared state between domains
- **Single Responsibility**: Each element has one responsibility
- **Dependency Injection**: Inject dependencies, don't create them
- **Test-Driven Development**: Tests must pass before implementation

### Code Organization
- Each domain is a separate crate with clear boundaries
- All domains follow the same internal structure (events, commands, queries, handlers)
- Use `cim-core` for shared types and utilities
- Bevy systems are in `cim-domain-bevy` for visualization

## Performance Benchmarks

The system has exceeded all performance targets:
- Event Creation: 779,352/sec (7.8x target)
- Event Publishing: 1,013,638/sec (101x target)
- Concurrent Operations: 2,389,116/sec
- Query Response: <10ms (15x faster)
- Memory Usage: 1.3KB/event (7.5x better)

## Testing Infrastructure

- **499+ Tests** (100% passing)
- **Unit Tests**: 460+ tests across all domains
- **Integration Tests**: 25 comprehensive tests
- **Performance Tests**: 6 benchmark tests
- **Error Handling**: 8 comprehensive error tests

## AI Integration

### Supported AI Providers
- OpenAI (GPT-4)
- Anthropic (Claude)
- Ollama (Local models)
- Custom fine-tuned models

### Conceptual Spaces Engine
- 5D semantic space for AI reasoning
- Geometric similarity calculations
- Automatic relationship discovery
- Knowledge graph generation

## Deployment

### NixOS-Based Infrastructure
- Reproducible builds with Nix
- Immutable infrastructure
- Blue-green deployments
- Automatic rollback capabilities

### Leaf Node Architecture
- Local processing with cloud sync
- Data locality for performance
- Privacy-preserving edge computing
- Resilient offline operation

## Working with Demos

Demo files are located in various domain crates and demonstrate:
- Workflow visualization with animated state transitions
- Event flow demonstration
- AI agent integration
- Cross-domain interactions
- Graph composition examples
- Real-time collaboration

When running demos, use the appropriate binary target for each domain.

## Task Orientation & Progress Tracking

Based on progress.json, the project is **100% COMPLETE** with all 14 domains production-ready. Current focus is on:

### Active Priorities (from progress.json)
1. Production deployment preparation
2. Performance optimization
3. Documentation updates
4. Real-time NATS event visualization integration
5. Advanced graph layout algorithms
6. Vector database integration (Qdrant/Weaviate)
7. Real AI provider integration for embeddings
8. Cross-domain semantic search capabilities

### Project Status Overview
- **Status**: COMPLETE - All 14 Domains Production-Ready
- **Completion**: 100% (48% overall completion accounting for future features)
- **Tests**: 468/468 passing (100% pass rate)
- **Architecture Health**: EXCELLENT
- **Business Impact**: 40% time savings demonstrated
- **Technical Debt**: 162 clippy warnings (down from 739)
- **Phase**: COMPLETE - PRODUCTION READY

### When Working on Tasks
1. **Always read progress.json first** to understand current state
2. **Follow TDD workflow religiously**:
   - Write user stories for requirements
   - Convert stories to failing tests
   - Implement code to make tests pass
   - Update test counts in progress.json
3. **Create nodes in the graph structure** for significant work
4. **Update completion metrics and test counts** 
5. **Add achievements to recent_changes array**
6. **Update domain-specific progress** where relevant
7. **Maintain the event-driven, CQRS architecture patterns**

### Test Requirements
- **Current Status**: 468/468 tests passing (100% pass rate)
- **All new features must have tests BEFORE implementation**
- **Tests must be added to appropriate domain test suites**
- **Integration tests required for cross-domain features**
- **Performance tests for critical paths**