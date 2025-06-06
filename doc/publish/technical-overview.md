# Technical Overview

## Implementation Status (January 2025)

Information Alchemist is actively being developed as a CIM leaf node with event-driven architecture. This document provides technical details for developers and contributors.

## 🏗️ Architecture

### Three-Layer Architecture

```
┌─────────────────────────────────────────┐
│   Presentation Layer (Bevy ECS)         │
│   - 3D/2D Visualization                 │
│   - User Interactions                   │
│   - Animation System                    │
├─────────────────────────────────────────┤
│   Application Layer (CQRS)              │
│   - Command Handlers                    │
│   - Query Handlers                      │
│   - Projections                         │
├─────────────────────────────────────────┤
│   Domain Layer (Event Sourcing)         │
│   - Graph Aggregate                     │
│   - Domain Events                       │
│   - Business Rules                      │
├─────────────────────────────────────────┤
│   Infrastructure Layer                  │
│   - NATS JetStream                      │
│   - Event Store                         │
│   - Object Store                        │
└─────────────────────────────────────────┘
```

### Event Flow

1. **Presentation Events** (UI interactions) stay in Bevy
2. **Domain Commands** express business intent
3. **Domain Events** record what happened
4. **Projections** update read models
5. **External Systems** integrate via bidirectional event flow

## 📊 Development Progress

### ✅ Completed Features
- **Phase 0**: NATS Integration - Secure messaging foundation
- **Phase 1**: Event Infrastructure - Distributed event store with CID chains
- **Phase 1.5**: IPLD Integration - Content-addressed storage
- **Graph Aggregate**: Full domain model with business rules
- **Integration Tests**: Comprehensive test suite
- **Projections**: Read model implementation
- **Event Separation**: Clear presentation/domain boundaries
- **K7 Visualization**: Complete graph with force-directed layout
- **Smooth Animations**: Ease-in/out transitions

### 🚧 In Progress
- **Phase 2**: Domain Model (60% complete)
  - Graph aggregate ✅
  - Workflow aggregate (pending)
  - ConceptualSpace aggregate (pending)
- **Phase 3**: CQRS Implementation (40% complete)
  - Command handlers ✅
  - Query handlers (partial)
  - Projections ✅
  - Snapshots (pending)
- **Test Coverage**: Currently ~65%, target 80%

### 📅 Upcoming Features
- **Phase 4**: Conceptual Spaces - Semantic positioning
- **Phase 5**: AI Agent Integration - Intelligent assistance
- **Phase 6**: Dog-fooding - Self-visualization
- **HUD System**: Real-time graph insights

## 🛠️ Technology Stack

### Core Technologies
- **Language**: Rust 1.89.0-nightly
- **Graphics**: Bevy 0.16 (Entity Component System)
- **Messaging**: NATS JetStream 2.10+
- **Storage**: Event Store + Object Store (CID-based)
- **Build**: Nix with flakes

### Key Dependencies
```toml
[dependencies]
bevy = "0.16"
async-nats = "0.41"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.11", features = ["v4", "serde"] }
thiserror = "2.0"
```

### Design Patterns
- **Event Sourcing**: All state changes as events
- **CQRS**: Command Query Responsibility Segregation
- **DDD**: Domain-Driven Design with aggregates
- **ECS**: Entity Component System for performance

## 🚦 Development Setup

### Prerequisites
- Nix package manager (with flakes enabled)
- Git with submodules support
- 8GB+ RAM recommended
- GPU with Vulkan support

### Quick Start
```bash
# Clone with submodules
git clone --recursive https://github.com/TheCowboyAI/alchemist.git
cd alchemist

# Enter development shell
nix develop

# Run with auto-reload
cargo watch -x "run --bin ia"

# Run tests
cargo test --lib

# Build release
nix build
```

### Project Structure
```
alchemist/
├── src/
│   ├── domain/          # Business logic
│   ├── application/     # CQRS handlers
│   ├── infrastructure/  # NATS, storage
│   └── presentation/    # Bevy ECS
├── tests/
│   ├── domain/         # Unit tests
│   └── integration/    # End-to-end tests
├── doc/
│   ├── design/current/ # Active designs
│   ├── plan/current/   # Active plans
│   └── publish/        # User documentation
└── nix/                # Build configuration
```

## 🧪 Testing Strategy

### Test Categories
1. **Domain Tests**: Pure business logic (no I/O)
2. **Integration Tests**: End-to-end with NATS
3. **Projection Tests**: Read model updates
4. **UI Tests**: Component behavior (planned)

### Running Tests
```bash
# All tests
cargo test

# Domain only
cargo test --lib domain

# Integration tests
cargo test --test '*'

# With coverage (requires nightly)
cargo +nightly tarpaulin
```

## 🔧 Configuration

### Environment Variables
```bash
# NATS Configuration
NATS_URL=nats://localhost:4222
NATS_USER=admin
NATS_PASSWORD=secret

# Development
RUST_LOG=info,ia=debug
BEVY_HEADLESS=1  # For tests
```

### Feature Flags
- `dev` - Development features (hot reload, debug UI)
- `production` - Optimized build
- `test` - Test utilities

## 📈 Performance Targets

- **Nodes**: 100K+ supported
- **Frame Rate**: 60 FPS maintained
- **Event Latency**: < 10ms local
- **Memory**: < 2GB for 100K nodes
- **Startup**: < 2 seconds

## 🤝 Contributing

### Key Areas Needing Help
1. **Domain Aggregates**: Workflow and ConceptualSpace
2. **Test Coverage**: Need 15% more coverage
3. **Query Handlers**: Optimize read models
4. **Documentation**: User guides and tutorials
5. **Performance**: Benchmarking and optimization

### Development Workflow
1. Check `/doc/plan/current/` for active plans
2. Review `/doc/design/current/` for designs
3. Follow DDD and event sourcing patterns
4. Write tests first (TDD encouraged)
5. Update progress.json when completing tasks

### Code Style
- Follow Rust standard style
- Use `cargo fmt` before commits
- Run `cargo clippy` for lints
- Document public APIs

## 🐛 Known Issues

1. **Bevy Dynamic Linking**: Must use `nix build` or `nix run`
2. **Test Isolation**: Some integration tests need NATS server
3. **Memory Usage**: Force layout can be expensive for large graphs

## 📚 Resources

### Documentation
- `/doc/design/current/` - Architecture designs
- `/doc/plan/current/` - Implementation plans
- `/doc/progress/progress.json` - Current status
- `.cursor/rules` - Project conventions

### External Resources
- [Bevy Book](https://bevyengine.org/learn/book/)
- [NATS Docs](https://docs.nats.io/)
- [Event Sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)
- [DDD Reference](https://www.domainlanguage.com/ddd/reference/)

---

*For the vision and user experience, see the main [README](README.md)*
