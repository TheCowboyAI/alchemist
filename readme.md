# Information Alchemist

![The Alchemist](./alchemist.webp)
> "A person who transforms or creates something through a seemingly magical process."

## Visual Intelligence for the Composable Information Machine

Information Alchemist is a powerful 3D-capable graph visualization and editing system that serves as the primary user interface for the Composable Information Machine (CIM). It transforms complex data relationships into intuitive, interactive visual spaces where information comes alive.

### 🚀 Key Features

- **3D/2D Visualization**: Seamlessly switch between immersive 3D exploration and efficient 2D overview modes
- **Subgraph Composition**: Load and compose multiple graphs while maintaining their structure as distinct subgraphs
- **Real-time Collaboration**: Multiple users can work on the same graph simultaneously
- **AI-Powered Insights**: Integrated AI agents provide pattern recognition and optimization suggestions
- **Event-Driven Architecture**: Every change is captured as an event, enabling perfect audit trails
- **High Performance**: Handles 250k+ elements at 60 FPS through advanced rendering optimizations
- **Extensible**: WASM-based plugin system for custom algorithms and visualizations

## 📚 Documentation

### [Business Documentation](doc/publish/business/)
For business leaders and decision makers - understand how Information Alchemist transforms your business through visual intelligence.

### [Technical Documentation](doc/publish/technical/)
For developers and technical implementers - comprehensive guides on architecture, integration, and extension.

### [Requirements & Planning](doc/plan/)
Detailed requirements documentation and implementation roadmap.

## 🏗️ Architecture Overview

Information Alchemist is built on three foundational models:

### Mathematical Model
Using Applied Category Theory to define and model information relationships:
- **Applied Categories**: Mathematical objects that model our information worlds
- **Category Theory**: Formal specifications for composable systems
- **Graph Theory**: Foundation for visualizing relationships and flows

### Observable Model (ECS)
Built on Bevy's Entity Component System for maximum performance and flexibility:

#### Components
- **Values**: Collections of data structures
- **Properties**: Attributes and metadata
- **No functionality**: Pure data representation

#### Entities
- **Identifiable Objects**: Unique identifiers for every element
- **Component Composition**: Entities are collections of components
- **Dynamic**: Components can be added/removed at runtime

#### Systems
- **Behaviors**: Functions that operate on entities with specific components
- **Parallel**: Systems run concurrently for performance
- **Event-Driven**: Systems respond to and emit events

### Domain Model (DDD)
Domains provide boundaries and meaning to our ECS worlds:
- **Bounded Contexts**: Clear separation of concerns
- **Ubiquitous Language**: Consistent terminology across the system
- **Event Sourcing**: All changes captured as domain events

## 🛠️ Development

### Prerequisites

- Rust (latest stable)
- Nix (for reproducible builds)
- WASM toolchain (for plugin development)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/thecowboyai/information-alchemist
cd ia

# Using Nix (recommended)
nix develop
nix build
nix run

# Or using Cargo directly
cargo build --release
cargo run
```

### Development with Nix

This project uses Nix for reproducible builds and development environments.

**Note**: Nix caching is sensitive to Git workspace status. For best performance, commit changes before building.

## 🎯 Use Cases

### Business Intelligence
- Customer journey visualization
- Supply chain optimization
- Risk relationship mapping
- Process flow analysis

### Software Architecture
- System dependency graphs
- Microservice relationships
- Data flow visualization
- API interaction mapping

### Knowledge Management
- Concept mapping
- Research relationships
- Documentation structure
- Learning pathways

## 🔌 Integration

Information Alchemist integrates seamlessly with the CIM backend through:

- **NATS JetStream**: Real-time event streaming
- **GraphQL**: Flexible data queries
- **REST API**: File import/export
- **WebSocket**: Live collaboration

## 🚦 Project Status

Information Alchemist is under active development. Current focus areas:

- [ ] Core graph engine implementation
- [ ] 3D rendering pipeline
- [ ] Event system integration
- [ ] WASM plugin architecture
- [ ] AI agent integration
- [ ] Performance optimization


## 📄 License

Information Alchemist is part of the Composable Information Machine ecosystem. See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

Built with:
- [Bevy Engine](https://bevyengine.org/) - Game engine and ECS framework
- [egui](https://github.com/emilk/egui) - Immediate mode GUI
- [petgraph](https://github.com/petgraph/petgraph) - Graph data structures
- [NATS](https://nats.io/) - Messaging and streaming

---

**Information Alchemist**: Where data becomes understanding through the magic of visualization.

*Part of the [Composable Information Machine](https://github.com/thecowboyai/CIM) project*



