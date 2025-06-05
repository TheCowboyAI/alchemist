# Information Alchemist

A graph visualization system that implements a **Composable Information Machine (CIM)** leaf node, combining event-driven architecture with conceptual space theory for semantic graph manipulation.

## Why This Changes Everything

Traditional tools force you to choose: either visual graphs OR semantic meaning OR version control OR AI integration. Information Alchemist gives you **all of them simultaneously**:

### 1. **Visual Workflows That Actually Execute**
- Draw a workflow → it becomes executable code
- See state machines running in real-time
- Debug by watching events flow through your graph
- Every execution is recorded and replayable

### 2. **AI That Understands Your Structure**
```rust
// Your graph isn't just nodes and edges - it has meaning
let similar_workflows = graph.find_similar_patterns(my_workflow);
let suggested_improvements = ai_agent.analyze_bottlenecks(my_workflow);
let auto_composed = ai_agent.combine_workflows(workflow_a, workflow_b);
```

### 3. **Time Travel Through Information**
- Replay any graph to any point in history
- See how decisions evolved
- Undo/redo with perfect fidelity
- Fork reality - explore "what if" scenarios

### 4. **Semantic Search That Works**
- "Find workflows similar to customer onboarding"
- "Show me all processes that touch payment data"
- "What changed between these two versions?"
- AI understands your intent, not just keywords

### 5. **Composable Knowledge Blocks**
- Save subgraphs as reusable components
- Snap together complex systems from simple parts
- Share verified patterns with your team
- Build a library of domain expertise

## Real-World Example

```rust
// A financial analyst exploring market strategies
let market_graph = Graph::new("Trading Strategies");

// Add domain knowledge
market_graph.add_concept("Risk Management", ConceptualPoint::from_embedding(risk_embed));
market_graph.add_concept("Profit Centers", ConceptualPoint::from_embedding(profit_embed));

// AI suggests connections based on semantic similarity
let suggested_edges = ai.find_hidden_relationships(&market_graph);

// Execute the strategy as a state machine
let execution = market_graph.as_workflow().execute_with_rollback();

// Time travel to see what worked
let successful_paths = execution.replay_successful_branches();
```

## Safe AI Manipulation

Unlike black-box AI tools, Information Alchemist gives you:

### **Complete Auditability**
- Every AI suggestion is an event you can inspect
- See exactly why the AI made each recommendation
- Roll back any AI-initiated change instantly
- Cryptographic proof of what happened when

### **Bounded Execution**
- AI operates within state machine constraints
- Invalid transitions are impossible by design
- Set permissions on what AI can modify
- Sandbox experiments before committing

### **Explainable Decisions**
```rust
// AI must explain its reasoning
let suggestion = ai.suggest_optimization(workflow);
assert!(suggestion.explanation.is_complete());
assert!(suggestion.evidence.is_traceable());
assert!(suggestion.impact.is_bounded());
```

## What Makes It Different

- **Conceptual Space Mapping**: Every graph element has both visual position and semantic coordinates, enabling similarity search and AI reasoning
- **State Machine Aggregates**: All transactional behavior controlled by Mealy state machines that can be visualized
- **Event-Sourced Everything**: Complete history with time-travel debugging and cryptographic integrity via CID chains
- **CIM Integration**: Operates as a distributed node in the larger Composable Information Machine network

## Architecture

Built on three foundational layers:

1. **Presentation Layer** (Bevy ECS) - Real-time 3D/2D visualization
2. **Domain Layer** (Event Sourcing + CQRS) - Business logic with state machines
3. **Infrastructure Layer** (NATS + Storage) - Distributed messaging and persistence

See [architecture/](architecture/) for details:
- [CIM Integration](architecture/cim-overview.md) - How we fit into the CIM ecosystem
- [Event Sourcing](architecture/event-sourcing.md) - State machines and event patterns
- [Components](architecture/system-components.md) - Detailed component reference

## Key Innovations

1. **Mealy State Machines for Aggregates**
   - If it needs transactions, it MUST be an aggregate with a state machine
   - State transitions generate events automatically
   - Can visualize state machines as graphs within the system

2. **Dual Space Representation**
   ```rust
   pub struct GraphNode {
       pub position: Vec3,              // Visual space
       pub conceptual_point: ConceptualPoint,  // Semantic space
   }
   ```

3. **Event Cascades**
   - Systems process components and emit events
   - Events trigger other systems, creating emergent workflows
   - No orchestration needed - behavior emerges from event patterns

## Quick Start

```bash
# Run with Nix
nix run

# Build
nix build
```

## Documentation

- [Vocabulary](vocabulary.md) - Domain terminology and concepts
- [Design Justification](design-justification.md) - Research-backed design decisions
- [Conceptual Implementation](conceptual-implementation.md) - Theory to practice
- [UI-Backend Integration](ui-backend-integration.md) - CIM leaf node architecture

## Technology Stack

- **Core**: Rust, Bevy 0.16.0 (ECS), NATS JetStream
- **Patterns**: Event Sourcing, CQRS, Domain-Driven Design
- **Theory**: Conceptual Spaces (Gärdenfors), CIM Architecture

## Status

Pre-release. Core architecture stable, implementation evolving.

---

*Information Alchemist: Where Information Becomes Understanding Through Visual Intelligence*
