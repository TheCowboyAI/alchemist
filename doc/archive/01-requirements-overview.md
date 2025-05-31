# Information Alchemist Requirements Overview

## Executive Summary

Information Alchemist is a next-generation 3D-capable graph editor and visualization system built as part of the Composable Information Machine (CIM) ecosystem. It combines advanced graph theory, Entity-Component-System (ECS) architecture, Domain-Driven Design (DDD), and Category Theory to provide a powerful tool for understanding and manipulating complex information relationships.

### Vision Statement
To create an intuitive, performant, and scalable graph visualization system that transforms abstract data relationships into tangible, manipulable 3D/2D spaces, enabling users to discover insights through spatial reasoning and visual exploration.

### Key Differentiators
- **Dual-mode visualization**: Seamless switching between 3D immersive and 2D overview modes
- **Physics-based layouts**: Natural organization through force-directed algorithms
- **Event-sourced architecture**: Complete audit trail and time-travel capabilities
- **Domain-aware**: Pluggable domain models with business-specific semantics
- **Massive scale**: Support for 250,000+ graph elements
- **Graph composition**: Load and compose multiple graphs while maintaining their distinct identities

## System Overview

### Product Description
Information Alchemist is a graph modeling and visualization platform that enables users to:
- Create, visualize, and manipulate complex graph structures
- Apply domain-specific semantics to graph elements
- Explore relationships through interactive 3D/2D visualization
- Integrate with the broader CIM ecosystem for distributed processing
- Compose multiple graphs into unified visualizations while preserving subgraph boundaries

### Target Users
1. **Knowledge Workers**: Business analysts exploring organizational relationships
2. **System Architects**: Designing and documenting system architectures
3. **Data Scientists**: Visualizing complex data relationships and patterns
4. **Domain Experts**: Modeling domain-specific workflows and processes

### Core Concepts

#### Graph Elements
- **GraphNode**: Fundamental vertices in the graph structure
- **GraphEdge**: Directed connections between nodes with semantic relationships
- **GraphProperty**: Key-value attributes attached to nodes and edges
- **GraphLabel**: Semantic tags for categorization and filtering
- **Subgraph**: Self-contained graph components that maintain their own identity and structure

#### Visualization Components
- **SpatialTransform**: 3D position, rotation, and scale data
- **VisualStyle**: Appearance properties (materials, colors, shapes)
- **InteractionState**: Selection, hover, and manipulation states
- **AnimationState**: Transition and motion properties
- **SubgraphBoundary**: Visual representation of subgraph containment

#### Domain Integration
- **DomainContext**: Bounded context for domain-specific rules
- **DomainEvent**: Business events that modify graph state
- **DomainProjection**: Views of the graph filtered by domain concerns

## High-Level Requirements

### 1. Graph Management
- Create, read, update, and delete graph structures
- Support for directed and undirected graphs
- Hierarchical graph organization (subgraphs)
- Import/export in multiple formats (native, Cypher, Mermaid, Arrows.app)
- **Compose multiple graphs into a single workspace**
- **Maintain subgraph identity and boundaries**
- **Extract subgraphs as independent graphs**

### 2. Visualization Capabilities
- Real-time 3D rendering with WebGPU/Vulkan
- Smooth transitions between 3D and 2D modes
- Level-of-detail (LOD) system for performance
- Customizable visual styles per domain
- **Visual distinction for subgraph boundaries**
- **Collapse/expand subgraphs for simplified views**
- **Highlight inter-subgraph connections**

### 3. Interaction Paradigms
- Direct manipulation (drag, rotate, scale)
- Multi-selection with box/lasso tools
- Context-sensitive actions based on selection
- Keyboard shortcuts for power users
- **Drag nodes between subgraphs**
- **Create edges across subgraph boundaries**
- **Manage subgraph membership interactively**

### 4. Layout Algorithms
- Force-directed (physics-based)
- Hierarchical (for directed acyclic graphs)
- Circular, grid, and custom layouts
- Manual positioning with snap-to-grid
- **Subgraph-aware layout algorithms**
- **Preserve subgraph internal structure**
- **Inter-subgraph spacing optimization**

### 5. Integration Points
- NATS JetStream for event distribution
- WebAssembly components for domain logic
- AI agents for intelligent layout suggestions
- External data source connectors

## Success Criteria

### Functional Success
- Users can create and manipulate graphs with 1000+ nodes at 60 FPS
- System scales to 250,000+ total elements
- Sub-second response time for common operations
- Zero data loss through event sourcing
- **Seamless composition of multiple graph files**
- **Clear visual separation of subgraphs**

### User Experience Success
- Intuitive interface requiring < 30 minutes training
- Smooth transitions and animations
- Consistent performance across view modes
- Accessible to users with varying technical backgrounds
- **Natural subgraph manipulation workflows**

### Technical Success
- Clean separation of concerns (ECS architecture)
- Extensible through domain plugins
- Deterministic builds with Nix
- Comprehensive test coverage (>80%)
- **Efficient subgraph boundary calculations**

## Constraints and Dependencies

### Technical Constraints
- Must run on NixOS development environment
- Bevy 0.16.0 as the core engine
- Rust as the primary implementation language
- WebGPU/Vulkan for rendering

### External Dependencies
- petgraph 0.8+ for graph algorithms
- daggy 0.9+ for DAG operations
- bevy_egui 0.34+ for UI overlays
- egui-snarl 0.7.1+ for workflow visualization
- NATS JetStream for messaging

### Architectural Constraints
- Strict adherence to ECS principles
- Event-driven state modifications only
- DDD naming conventions throughout
- Composable "Lego block" architecture

## Document Organization

This requirements documentation is organized into the following sections:

1. **Requirements Overview** (this document) - High-level vision and scope
2. **Domain Model** - Detailed domain entities, events, and boundaries
3. **Technical Architecture** - System design and component architecture
4. **User Stories** - Detailed user scenarios and workflows
5. **Non-Functional Requirements** - Performance, security, and quality attributes
6. **Implementation Phases** - Roadmap for iterative development

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2024-12-18 | System | Initial requirements document |
| 1.1 | 2024-12-18 | System | Added graph composition and subgraph management capabilities |
