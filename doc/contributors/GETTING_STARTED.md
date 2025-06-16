# Getting Started as a Contributor

Welcome to Information Alchemist! This guide will help you understand our project, set up your development environment, and make your first meaningful contribution.

## About Information Alchemist

Information Alchemist is a **Composable Information Machine (CIM)** - a revolutionary system that combines:

- **Event-Driven Architecture**: All state changes flow through immutable events
- **Domain-Driven Design**: Clear business boundaries and ubiquitous language
- **Entity Component System**: High-performance, composable system architecture
- **Graph-Based Visualization**: 3D/2D exploration of complex information relationships
- **AI-Native Design**: Built for seamless integration with intelligent agents

## Our EGALITARIAN Values

Before you begin, please understand our commitment to **egalitarian principles**:

- **Merit-Based Evaluation**: Your contributions are judged on technical quality and alignment with project goals, not your background or status
- **Equal Voice**: Every contributor has equal opportunity to propose ideas and participate in discussions
- **Inclusive Excellence**: We maintain high technical standards while making our processes accessible to contributors of all experience levels
- **Respectful Collaboration**: All interactions are professional, constructive, and focused on the work

Read our complete [EGALITARIAN Code of Conduct](../../.github/CODE_OF_CONDUCT.md) to understand how we work together.

## Architecture Overview

### The Three Models

Information Alchemist is built on three foundational models:

#### 1. Mathematical Model (Applied Category Theory)
- **Categories**: Mathematical objects that model information relationships
- **Functors**: Structure-preserving mappings between categories
- **Natural Transformations**: Systematic ways to transform between functors

#### 2. Observable Model (Entity Component System)
- **Components**: Pure data structures with no behavior
- **Entities**: Unique identifiers that compose components
- **Systems**: Functions that operate on entities with specific components
- **Events**: Messages that trigger system responses

#### 3. Domain Model (Domain-Driven Design)
- **Bounded Contexts**: Clear boundaries between business areas
- **Aggregates**: Consistency boundaries for business rules
- **Domain Events**: Business-meaningful occurrences
- **Commands & Queries**: Separate read and write operations (CQRS)

### Technology Stack

- **Language**: Rust (latest stable)
- **Game Engine**: Bevy 0.16 (Entity Component System)
- **Messaging**: NATS JetStream (event streaming)
- **Build System**: Nix (reproducible builds)
- **Graphics**: 3D visualization with WebGPU
- **Platforms**: Native desktop, with WASM support planned

## Setting Up Your Development Environment

### Prerequisites

1. **Nix** (strongly recommended for reproducible builds)
   ```bash
   # Install Nix with Flakes support
   curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
   ```

2. **Git** for version control

3. **Optional**: Rust toolchain (Nix will provide this, but local install can be faster for IDEs)

### Quick Start

1. **Fork and Clone**
   ```bash
   # Fork the repository on GitHub first
   git clone https://github.com/YOUR_USERNAME/alchemist
   cd alchemist
   ```

2. **Initialize Submodules** (Critical Step!)
   ```bash
   # Initialize all git submodules - required for building
   git submodule update --init --recursive
   
   # This downloads all domain modules and infrastructure
   ```

3. **Enter Development Environment**
   ```bash
   # Using Nix (recommended)
   nix develop
   
   # This provides all tools: Rust, NATS, testing frameworks, etc.
   ```

4. **Build the Project**
   ```bash
   # For development, use cargo (handles submodules correctly)
   cargo build
   
   # For testing
   cargo test
   
   # Note: nix build has submodule limitations - use cargo in the nix shell
   ```

4. **Run Tests**
   ```bash
   # Run all tests in headless mode
   BEVY_HEADLESS=1 cargo test
   
   # Run specific domain tests
   cd cim-domain-graph
   cargo test
   ```

5. **Start the Application**
   ```bash
   # Run the main application
   nix run
   
   # Or with cargo
   cargo run
   ```

## Project Structure

Understanding our codebase organization:

```
alchemist/
├── src/                          # Main application (presentation layer)
├── cim-domain-*/                 # Domain modules (bounded contexts)
│   ├── cim-domain-graph/         # Graph visualization domain
│   ├── cim-domain-agent/         # AI agent management
│   ├── cim-domain-identity/      # Authentication & authorization
│   ├── cim-domain-policy/        # Business rules & governance
│   └── ...                       # Other domain contexts
├── cim-infrastructure/           # Cross-cutting infrastructure
├── cim-ipld/                     # Content-addressed storage
├── doc/                          # Comprehensive documentation
│   ├── design/                   # Architecture and design docs
│   ├── plan/                     # Implementation roadmaps
│   ├── progress/                 # Development tracking
│   └── publish/                  # User-facing documentation
├── .github/                      # GitHub templates and workflows
└── nix/                          # Nix build configuration
```

### Domain Modules

Each `cim-domain-*` directory is a self-contained bounded context:

```
cim-domain-example/
├── src/
│   ├── aggregate/       # Domain entities and aggregates
│   ├── commands/        # Command definitions
│   ├── events/          # Domain events  
│   ├── handlers/        # Command and event handlers
│   ├── queries/         # Query definitions
│   ├── projections/     # Read models
│   └── value_objects/   # Immutable value types
└── tests/               # Domain-specific tests
```

## Making Your First Contribution

### 1. Choose Your Area

Pick an area that interests you:

- **Domain Logic**: Business rules and workflows
- **Visualization**: 3D/2D graph rendering and interaction
- **Performance**: Optimization and benchmarking
- **Documentation**: Guides, examples, and API docs
- **Infrastructure**: Build systems and deployment
- **Testing**: Test coverage and quality assurance

### 2. Find a Good First Issue

Look for issues labeled:
- `good-first-issue`: Perfect for newcomers
- `help-wanted`: Community input needed
- `documentation`: Documentation improvements
- `domain-{name}`: Specific to a domain you're interested in

### 3. Understand the Domain

Before making changes:

1. **Read the Documentation**: Start with `/doc/design/` for your area
2. **Understand the Events**: Look at the domain events to understand what happens
3. **Study the Tests**: Tests show expected behavior and serve as examples
4. **Run the Code**: Experience the current functionality

### 4. Follow the Development Process

1. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Write Tests First** (TDD)
   ```bash
   # Write failing tests that describe your feature
   cargo test -- your_test_name
   ```

3. **Implement the Feature**
   - Follow domain-driven design principles
   - Keep components pure data
   - Make state changes through events
   - Maintain single responsibility principle

4. **Ensure Quality**
   ```bash
   # Format code
   nix fmt
   
   # Run linter
   cargo clippy
   
   # Run all tests
   cargo test
   
   # Check build
   nix flake check
   ```

5. **Submit a Pull Request**
   - Use our [PR template](../../.github/pull_request_template.md)
   - Fill out all sections thoroughly
   - Reference related issues

## Development Best Practices

### Domain-Driven Design

- **Start with Events**: Define what happens before how it happens
- **Use Ubiquitous Language**: Use business terms, not technical jargon
- **Respect Boundaries**: Don't couple domains directly - use events
- **Model Business Rules**: Put business logic in aggregates, not systems

### Event Sourcing

- **Events are Facts**: Use past tense (e.g., `OrderPlaced`, not `PlaceOrder`)
- **Events are Immutable**: Never change an event once created
- **Events have Meaning**: Each event represents a business occurrence
- **Chain Integrity**: Events form cryptographically verified chains

### Entity Component System

- **Components are Data**: No methods, just fields
- **Systems are Behavior**: Pure functions that operate on components
- **Entities are IDs**: Lightweight identifiers that group components
- **Events Trigger Changes**: Don't mutate directly, emit events

### Testing Philosophy

- **Test-Driven Development**: Write tests first
- **Domain Tests are Pure**: No Bevy/NATS dependencies in domain tests
- **Headless Testing**: All tests run in `BEVY_HEADLESS=1` mode
- **High Coverage**: Aim for 95%+ test coverage
- **Document with Mermaid**: Include diagrams in test documentation

## Common Patterns

### Adding a New Domain Event

```rust
// 1. Define the event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct NodeCreated {
    pub node_id: NodeId,
    pub position: Position3D,
    pub created_at: SystemTime,
}

// 2. Update the aggregate
impl GraphAggregate {
    pub fn create_node(&mut self, position: Position3D) -> DomainEvent {
        let node_id = NodeId::new();
        let event = NodeCreated {
            node_id,
            position,
            created_at: SystemTime::now(),
        };
        
        self.apply_event(&event);
        DomainEvent::NodeCreated(event)
    }
    
    fn apply_event(&mut self, event: &NodeCreated) {
        self.nodes.insert(event.node_id, Node::new(event.position));
    }
}

// 3. Handle in ECS system
fn handle_node_created(
    mut commands: Commands,
    mut events: EventReader<NodeCreated>,
) {
    for event in events.read() {
        commands.spawn((
            NodeComponent { id: event.node_id },
            Transform::from_translation(event.position.into()),
        ));
    }
}
```

### Adding a New Command

```rust
// 1. Define the command
#[derive(Debug, Clone)]
pub struct CreateNode {
    pub graph_id: GraphId,
    pub position: Position3D,
}

// 2. Create handler
pub async fn handle_create_node(
    cmd: CreateNode,
    event_store: &EventStore,
) -> Result<()> {
    let mut graph = event_store.load_aggregate(cmd.graph_id).await?;
    let events = graph.create_node(cmd.position);
    event_store.save_events(cmd.graph_id, events).await?;
    Ok(())
}

// 3. Wire up in Bevy
fn process_create_node_commands(
    mut commands: EventReader<CreateNode>,
    bridge: Res<EventBridge>,
) {
    for cmd in commands.read() {
        bridge.send_command(cmd.clone());
    }
}
```

## Getting Help

### Documentation Resources

- **Architecture**: `/doc/design/` - Design decisions and patterns
- **User Guide**: `/doc/publish/` - User-facing documentation
- **Examples**: `/examples/` - Working code examples
- **Progress**: `/doc/progress/` - Current development status

### Community Support

- **GitHub Issues**: Ask questions using our [question template](../../.github/ISSUE_TEMPLATE/question.yml)
- **GitHub Discussions**: Participate in design discussions
- **Code Reviews**: Learn from PR feedback and discussions

### Development Questions

Before asking for help, please:
1. Search existing issues and discussions
2. Read relevant documentation
3. Look at examples in the codebase
4. Try to understand the domain model

When asking for help:
- Be specific about what you're trying to accomplish
- Include relevant code snippets
- Mention what you've already tried
- Specify your experience level with Rust/Bevy/DDD

## Contributing Etiquette

### Communication Style

- **Be Direct and Clear**: State your intent and reasoning clearly
- **Focus on the Work**: Keep discussions centered on technical matters
- **Provide Evidence**: Support your arguments with code, benchmarks, or references
- **Acknowledge Others**: Give credit where it's due
- **Ask Questions**: Don't hesitate to ask for clarification

### Code Review Process

- **Review Thoroughly**: Take time to understand the changes
- **Be Constructive**: Offer specific suggestions for improvement
- **Explain Your Reasoning**: Help others understand your perspective
- **Accept Feedback**: Be open to different approaches
- **Iterate Quickly**: Respond to feedback promptly

### Conflict Resolution

If disagreements arise:
1. Focus on technical merits and project goals
2. Provide concrete examples and evidence
3. Consider multiple perspectives
4. Escalate to maintainers if needed
5. Accept decisions gracefully and move forward

## Advanced Topics

### Performance Considerations

- **ECS Efficiency**: Design components for cache-friendly access patterns
- **Event Processing**: Batch related events when possible
- **Memory Management**: Avoid unnecessary allocations in hot paths
- **Profiling**: Use Rust's profiling tools to identify bottlenecks

### Security Practices

- **Input Validation**: Validate all external inputs at domain boundaries
- **Event Integrity**: Maintain cryptographic chains for event verification
- **Access Control**: Implement proper authorization in command handlers
- **Dependency Management**: Keep dependencies updated and audited

### Integration Testing

- **Cross-Domain Flows**: Test event flows between bounded contexts
- **NATS Integration**: Test message serialization and delivery
- **Performance Testing**: Benchmark critical user journeys
- **Error Scenarios**: Test failure modes and recovery

## What's Next?

After your first contribution:

1. **Explore Other Domains**: Try working in different bounded contexts
2. **Improve Performance**: Profile and optimize critical paths
3. **Enhance Documentation**: Help other contributors understand the system
4. **Design New Features**: Propose enhancements that align with our vision
5. **Mentor Others**: Help new contributors get started

## Resources for Learning

### Domain-Driven Design
- "Domain-Driven Design" by Eric Evans
- "Implementing Domain-Driven Design" by Vaughn Vernon
- Our project-specific DDD patterns in `/doc/design/`

### Event Sourcing
- "Versioning in an Event Sourced System" by Greg Young  
- "Event Sourcing" by Martin Fowler
- Our implementation in `cim-infrastructure/event_store/`

### Entity Component System
- "Game Programming Patterns" by Robert Nystrom (ECS chapter)
- Bevy's official ECS guide
- Our ECS integration patterns in `/doc/design/`

### Rust Development
- "The Rust Programming Language" (official book)
- "Rust for Rustaceans" by Jon Gjengset
- Our Rust patterns and conventions in project code

---

Welcome to the Information Alchemist community! We're excited to see what you'll build with us.

*Questions? Check our [FAQ](../qa/) or open a [discussion](../../discussions/new).*

**Version**: 1.0  
**Last Updated**: January 12, 2025  
**Maintainers**: Cowboy AI, LLC 