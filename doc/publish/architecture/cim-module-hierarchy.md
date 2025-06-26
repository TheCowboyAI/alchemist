# CIM Module Hierarchy and Architecture

## Overview

The Composable Information Machine (CIM) is a distributed, event-driven system built on content-addressed storage with claims-based security. This document establishes the definitive module hierarchy and explains how these modules assemble into a complete CIM system.

## System Architecture

A complete CIM deployment consists of:

1. **One or more CID-Based Object Stores** - Content-addressed storage for immutable data
2. **One or more Event Stores** - Distributed event logs using NATS JetStream
3. **Claims-Based Security with Yubikey Support** - Hardware-backed authentication
4. **A Leaf Node within a specific Bounded Context** - Edge connectivity
5. **A Cluster within the same Bounded Context** - Core processing
6. **Many Alchemist Clients** - Native, VM, or containerized clients
7. **Network Attached Storage (Minio)** - Local object storage
8. **Wasabi Buckets** - Long-term cloud storage

## Module Hierarchy

### Layer 1: Core Abstractions (Always Available)

These modules form the foundation and are shared between clients and servers:

#### 1.1 cim-ipld
- **Purpose**: Content-addressed storage and IPLD (InterPlanetary Linked Data) support
- **Key Components**:
  - `ChainedContent` - CID-based content chains
  - `CodecRegistry` - Content type codecs
  - `ObjectStore` - Content storage interface
  - Document, Image, Audio, Video content types
- **Dependencies**: None (foundational)

#### 1.2 cim-security
- **Purpose**: Security abstractions for crypto, claims, and secrets
- **Key Components**:
  - Cryptographic operation traits (Sign, Verify, Encrypt, Decrypt)
  - Claims-based authentication abstractions
  - Secrets management interfaces
  - Security policy and context
  - Authentication/Authorization traits
- **Dependencies**: None (foundational)

#### 1.3 cim-infrastructure
- **Purpose**: Cross-cutting infrastructure concerns
- **Key Components**:
  - `NatsClient` - NATS messaging client
  - `NatsConfig` - Connection configuration
  - `MessageHandler` - Message processing interface
  - Error types and results
- **Dependencies**: cim-security (for auth), cim-ipld (for storage)

#### 1.4 cim-component
- **Purpose**: Component trait for attaching data to domain objects
- **Key Components**:
  - `Component` trait - Type-erased components
  - Component storage and retrieval
  - Error handling
- **Dependencies**: None (foundational)

#### 1.5 cim-domain
- **Purpose**: Core Domain-Driven Design abstractions
- **Key Components**:
  - `Entity`, `AggregateRoot` - DDD entities
  - `Command`, `Query`, `DomainEvent` - CQRS patterns
  - `CommandHandler`, `QueryHandler` - Processing interfaces
  - State machines (Moore and Mealy)
  - Event sourcing support
- **Dependencies**: cim-component, cim-subject, cim-security

#### 1.6 cim-bridge
- **Purpose**: AI provider integration bridge
- **Key Components**:
  - Provider interfaces (Ollama, OpenAI, Anthropic)
  - Bridge service for AI communication
  - Message routing and translation
- **Dependencies**: cim-infrastructure

#### 1.7 cim-subject
- **Purpose**: Subject algebra and message routing
- **Key Components**:
  - `Subject`, `Pattern` - Hierarchical message addressing
  - `SubjectAlgebra` - Compositional subject operations
  - `Translator` - Schema translation between contexts
  - `MessageIdentity` - Correlation/causation tracking
  - In-memory and NATS routing implementations
- **Dependencies**: cim-ipld (for CID-based correlation)

### Layer 2: Domain Extensions

These modules extend cim-domain with specific domain concepts:

#### 2.1 Primary Domains
- **cim-domain-graph** - Graph structures and operations
- **cim-domain-identity** - Identity and access management
- **cim-domain-person** - Person entities and relationships
- **cim-domain-agent** - AI agent management
- **cim-domain-organization** - Organizational structures
- **cim-domain-location** - Geographic and spatial data
- **cim-domain-document** - Document management
- **cim-domain-git** - Git repository integration
- **cim-domain-nix** - Nix configuration management
- **cim-domain-workflow** - Business process workflows
- **cim-domain-dialog** - Conversational interactions
- **cim-domain-policy** - Business rules and policies

#### 2.2 Presentation Domain
- **cim-domain-bevy** - Bevy ECS integration for 3D visualization

### Layer 3: Composition and Behavior

These modules compose domains into bounded contexts and manage behavior:

#### 3.1 cim-compose
- **Purpose**: Composition of domains into bounded contexts
- **Key Components**:
  - Context boundaries
  - Domain composition rules
  - Cross-domain integration patterns

#### 3.2 cim-workflow-graph
- **Purpose**: Graph-based workflow representation
- **Key Components**:
  - Workflow as directed graphs
  - State machine integration
  - Visual workflow design

#### 3.3 cim-subject
- **Purpose**: NATS subject pattern management
- **Key Components**:
  - Subject parsing and validation
  - Permission management
  - Message routing patterns

#### 3.4 cim-keys
- **Purpose**: Cryptographic key management
- **Key Components**:
  - GPG, SSH, TLS key support
  - Yubikey integration
  - PKI infrastructure

### Layer 4: Reasoning Structures

These modules provide analysis, construction, and persistence capabilities:

#### 4.1 cim-conceptualspaces
- **Purpose**: Geometric representation of semantic concepts
- **Key Components**:
  - Quality dimensions
  - Conceptual regions
  - Similarity metrics
  - Semantic navigation

#### 4.2 cim-contextgraph
- **Purpose**: Context-aware graph structures
- **Key Components**:
  - Contextual relationships
  - Graph projections
  - Context switching

#### 4.3 cim-conceptgraph
- **Purpose**: Concept-based knowledge graphs
- **Key Components**:
  - Concept nodes and relationships
  - Semantic reasoning
  - Knowledge representation

### Layer 5: Client Applications

#### 5.1 Alchemist Client
- **Purpose**: Primary CIM client application
- **Deployment**: Native, VM, or container
- **Features**:
  - 3D visualization (Bevy)
  - Web interface (Iced)
  - Terminal UI
  - GUI windows
  - NATS messaging interface

## Module Assembly Pattern

### 1. Core Assembly
```
cim-ipld + cim-security + cim-infrastructure + cim-component + cim-domain + cim-bridge + cim-subject
    ↓
Foundation Layer (shared by all modules)
```

### 2. Domain Assembly
```
cim-domain + domain-specific modules
    ↓
Domain Layer (business logic and entities)
```

### 3. Context Assembly
```
Domain modules + cim-compose + cim-workflow + cim-subject
    ↓
Bounded Contexts with defined boundaries
```

### 4. Reasoning Assembly
```
Contexts + conceptual spaces + graphs
    ↓
Knowledge representation and reasoning
```

### 5. Client Assembly
```
All layers + presentation (Bevy/Iced) + NATS leaf node
    ↓
Complete Alchemist client
```

## Read/Write Separation (CQRS)

All modules follow CQRS principles:

- **Commands**: Change state, return only acknowledgments
- **Queries**: Read state, return only acknowledgments
- **Events**: Communicate state changes between contexts
- **Projections**: Optimized read models for queries

## Event Flow

1. **Client → NATS**: Commands/queries sent as messages
2. **NATS → Domain Handler**: Routed to appropriate handler
3. **Handler → Aggregate**: Business logic execution
4. **Aggregate → Events**: State changes as events
5. **Events → Event Store**: Persisted with CID chains
6. **Events → Projections**: Update read models
7. **Events → NATS**: Published for other contexts
8. **NATS → Client**: Acknowledgments and event streams

## Security Model

- **Authentication**: Claims-based with Yubikey support (cim-security abstractions)
- **Authorization**: Subject-based permissions (cim-subject + cim-security)
- **Encryption**: TLS for transport, GPG for data at rest (cim-keys implements cim-security)
- **Integrity**: CID chains ensure tamper detection (cim-ipld)

## Deployment Topology

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ Alchemist Client│     │ Alchemist Client│     │ Alchemist Client│
└────────┬────────┘     └────────┬────────┘     └────────┬────────┘
         │                       │                       │
         └───────────────────────┴───────────────────────┘
                                 │
                        ┌────────▼────────┐
                        │   NATS Leaf     │
                        │   Node (Edge)   │
                        └────────┬────────┘
                                 │
                        ┌────────▼────────┐
                        │  NATS Cluster   │
                        │  (Core + JS)    │
                        └────────┬────────┘
                                 │
                ┌────────────────┴────────────────┐
                │                                 │
       ┌────────▼────────┐              ┌────────▼────────┐
       │   Event Store   │              │  Object Store   │
       │  (JetStream)    │              │    (Minio)      │
       └────────┬────────┘              └────────┬────────┘
                │                                 │
                └──────────┬──────────────────────┘
                           │
                  ┌────────▼────────┐
                  │  Wasabi Buckets │
                  │  (Long-term)    │
                  └─────────────────┘
```

## Summary

The CIM architecture provides a complete platform for building composable information systems through:

1. **Modular Design**: Clear separation of concerns across layers
2. **Event-Driven**: All state changes flow through events
3. **Content-Addressed**: Immutable data with cryptographic integrity
4. **Domain-Aligned**: Business concepts drive the architecture
5. **Distributed**: Scales horizontally with NATS clustering
6. **Secure**: Hardware-backed authentication and encryption
7. **Flexible**: Multiple client deployment options

This hierarchy ensures that each module has a single responsibility while enabling complex systems through composition. 