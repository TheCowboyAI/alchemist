# Graph Models and Structure-Preserving Morphisms

## Core Philosophy

In CIM, we work with **known graph models** that have well-defined mathematical properties. Each graph model represents a specific structure with its own invariants, and we can apply **structure-preserving morphisms** to transform between compatible models.

## Graph Model Taxonomy

### 1. Complete Graphs (Kn)

A complete graph Kn has n vertices where every pair of distinct vertices is connected by a unique edge.

```rust
pub struct CompleteGraph {
    pub order: usize, // n in Kn
}

impl CompleteGraph {
    pub fn edge_count(&self) -> usize {
        self.order * (self.order - 1) / 2
    }

    pub fn is_valid(&self, graph: &GraphAggregate) -> bool {
        graph.node_count() == self.order &&
        graph.edge_count() == self.edge_count()
    }
}
```

**Properties:**
- K3: Triangle (3 nodes, 3 edges)
- K4: Tetrahedron (4 nodes, 6 edges)
- K5: Complete pentagon (5 nodes, 10 edges)
- K7: Complete heptagon (7 nodes, 21 edges)

### 2. Cycle Graphs (Cn)

A cycle graph Cn consists of n vertices connected in a closed chain.

```rust
pub struct CycleGraph {
    pub order: usize, // n in Cn
}

impl CycleGraph {
    pub fn is_valid(&self, graph: &GraphAggregate) -> bool {
        graph.node_count() == self.order &&
        graph.edge_count() == self.order &&
        graph.all_nodes_have_degree(2)
    }
}
```

**Properties:**
- C3: Triangle cycle
- C4: Square cycle
- C5: Pentagon cycle
- C6: Hexagon cycle

### 3. State Machines

#### Mealy Machine
Output depends on current state and input.

```rust
pub struct MealyMachine {
    pub states: HashSet<StateId>,
    pub inputs: HashSet<Input>,
    pub outputs: HashSet<Output>,
    pub transitions: HashMap<(StateId, Input), (StateId, Output)>,
    pub initial_state: StateId,
}
```

#### Moore Machine
Output depends only on current state.

```rust
pub struct MooreMachine {
    pub states: HashSet<StateId>,
    pub inputs: HashSet<Input>,
    pub outputs: HashMap<StateId, Output>,
    pub transitions: HashMap<(StateId, Input), StateId>,
    pub initial_state: StateId,
}
```

### 4. Domain-Specific Models

#### Address Graph
Represents the structure of an address value object.

```rust
pub struct AddressGraph {
    pub street_node: NodeId,
    pub city_node: NodeId,
    pub state_node: NodeId,
    pub postal_node: NodeId,
    pub country_node: NodeId,
    pub relationships: Vec<AddressRelation>,
}
```

#### Workflow Graph
Represents business processes with clear start/end and decision points.

```rust
pub struct WorkflowGraph {
    pub start_nodes: Vec<NodeId>,
    pub end_nodes: Vec<NodeId>,
    pub decision_nodes: Vec<NodeId>,
    pub action_nodes: Vec<NodeId>,
    pub flow_edges: Vec<FlowEdge>,
}
```

## Graph Recognition

### Pattern Matching

```rust
impl GraphAggregate {
    pub fn recognize_model(&self) -> Option<GraphModel> {
        // Try each model type in order of specificity

        // Check for state machines first (most specific)
        if let Some(mealy) = self.try_parse_mealy_machine() {
            return Some(GraphModel::MealyMachine(mealy));
        }

        if let Some(moore) = self.try_parse_moore_machine() {
            return Some(GraphModel::MooreMachine(moore));
        }

        // Check for domain models
        if let Some(address) = self.try_parse_address_graph() {
            return Some(GraphModel::AddressGraph(address));
        }

        // Check for mathematical graphs
        if self.is_complete_graph() {
            return Some(GraphModel::Complete(self.node_count()));
        }

        if self.is_cycle_graph() {
            return Some(GraphModel::Cycle(self.node_count()));
        }

        // Unknown structure
        None
    }
}
```

### Structural Invariants

Each model maintains specific invariants:

```rust
pub trait GraphModelInvariant {
    fn check_invariants(&self, graph: &GraphAggregate) -> Result<(), InvariantViolation>;
}

impl GraphModelInvariant for CompleteGraph {
    fn check_invariants(&self, graph: &GraphAggregate) -> Result<(), InvariantViolation> {
        // Every node must be connected to every other node
        for node_a in graph.nodes() {
            for node_b in graph.nodes() {
                if node_a != node_b && !graph.has_edge(node_a, node_b) {
                    return Err(InvariantViolation::MissingEdge(node_a, node_b));
                }
            }
        }
        Ok(())
    }
}
```

## Structure-Preserving Morphisms

### Morphism Types

```rust
pub enum GraphMorphism {
    // Completions
    ToComplete,              // Add missing edges to make complete

    // Cycles
    ToCycle,                // Remove edges to form cycle
    BreakCycle,             // Add chord to break cycle

    // Subdivisions
    SubdivideEdges(usize),  // Replace each edge with path of n edges
    ContractEdges(Vec<EdgeId>), // Contract specified edges

    // State Machine Transformations
    MealyToMoore,           // Convert Mealy to Moore machine
    MooreToMealy,           // Convert Moore to Mealy machine
    MinimizeStates,         // Reduce to minimal equivalent machine

    // Domain Transformations
    NormalizeAddress,       // Standardize address format
    SimplifyWorkflow,       // Remove redundant paths
}
```

### Morphism Application

```rust
impl GraphAggregate {
    pub fn apply_morphism(&mut self, morphism: GraphMorphism) -> Result<Vec<DomainEvent>> {
        let mut events = Vec::new();

        match morphism {
            GraphMorphism::ToComplete => {
                let missing_edges = self.find_missing_edges_for_complete();
                for (source, target) in missing_edges {
                    let edge_id = EdgeId::new();
                    self.add_edge(edge_id, source, target)?;
                    events.push(DomainEvent::EdgeAdded {
                        graph_id: self.id,
                        edge_id,
                        source,
                        target,
                        relationship: EdgeRelationship::Completion,
                    });
                }
            }

            GraphMorphism::SubdivideEdges(n) => {
                let edges_to_subdivide: Vec<_> = self.edges.keys().cloned().collect();
                for edge_id in edges_to_subdivide {
                    let (source, target) = self.get_edge_endpoints(edge_id)?;

                    // Remove original edge
                    self.remove_edge(edge_id)?;
                    events.push(DomainEvent::EdgeRemoved {
                        graph_id: self.id,
                        edge_id,
                    });

                    // Add subdivision nodes and edges
                    let mut current = source;
                    for i in 0..n-1 {
                        let intermediate = NodeId::new();
                        self.add_node(intermediate, NodeContent::subdivision(i))?;
                        events.push(DomainEvent::NodeAdded {
                            graph_id: self.id,
                            node_id: intermediate,
                            content: NodeContent::subdivision(i),
                        });

                        let edge = EdgeId::new();
                        self.add_edge(edge, current, intermediate)?;
                        events.push(DomainEvent::EdgeAdded {
                            graph_id: self.id,
                            edge_id: edge,
                            source: current,
                            target: intermediate,
                            relationship: EdgeRelationship::Subdivision,
                        });

                        current = intermediate;
                    }

                    // Final edge to target
                    let final_edge = EdgeId::new();
                    self.add_edge(final_edge, current, target)?;
                    events.push(DomainEvent::EdgeAdded {
                        graph_id: self.id,
                        edge_id: final_edge,
                        source: current,
                        target,
                        relationship: EdgeRelationship::Subdivision,
                    });
                }
            }

            // ... other morphisms
        }

        // Verify invariants after morphism
        if let Some(model) = self.recognize_model() {
            model.check_invariants(self)?;
        }

        Ok(events)
    }
}
```

## Model Templates

### Creating from Templates

```rust
impl GraphAggregate {
    pub fn from_model(model: GraphModel) -> Self {
        match model {
            GraphModel::Complete(n) => Self::create_complete_graph(n),
            GraphModel::Cycle(n) => Self::create_cycle_graph(n),
            GraphModel::MealyMachine(spec) => Self::create_mealy_machine(spec),
            GraphModel::AddressGraph(template) => Self::create_address_graph(template),
            // ... other models
        }
    }

    fn create_complete_graph(n: usize) -> Self {
        let mut graph = Self::new(GraphId::new(), GraphMetadata {
            name: format!("K{}", n),
            model_type: Some("Complete".to_string()),
            ..Default::default()
        });

        // Create nodes
        let nodes: Vec<_> = (0..n)
            .map(|i| {
                let id = NodeId::new();
                graph.add_node(id, NodeContent::vertex(i)).unwrap();
                id
            })
            .collect();

        // Create all edges
        for i in 0..n {
            for j in i+1..n {
                let edge_id = EdgeId::new();
                graph.add_edge(edge_id, nodes[i], nodes[j]).unwrap();
            }
        }

        graph
    }
}
```

## Benefits of Model-Based Approach

### 1. **Semantic Clarity**
Each graph has a known meaning and purpose, not just arbitrary nodes and edges.

### 2. **Validation**
We can verify that graphs maintain their model's invariants.

### 3. **Optimization**
Known models enable specific optimizations (e.g., complete graphs have predictable structure).

### 4. **Transformation**
Structure-preserving morphisms allow safe, meaningful transformations.

### 5. **Recognition**
We can identify patterns and suggest appropriate models.

### 6. **Generation**
Templates enable quick creation of standard structures.

## Implementation Guidelines

### DO:
- ✅ Recognize and name models explicitly
- ✅ Maintain model invariants
- ✅ Use morphisms for transformations
- ✅ Create from templates when possible
- ✅ Document domain-specific models

### DON'T:
- ❌ Use inheritance for models (composition over inheritance)
- ❌ Allow arbitrary modifications that break invariants
- ❌ Treat all graphs as generic structures
- ❌ Ignore model semantics in operations

## Example: Working with Models

```rust
// 1. Create a K5 complete graph
let k5 = GraphAggregate::from_model(GraphModel::Complete(5));

// 2. Recognize an existing graph
let model = existing_graph.recognize_model();
match model {
    Some(GraphModel::Complete(n)) => {
        println!("This is a complete graph K{}", n);
    }
    Some(GraphModel::MealyMachine(m)) => {
        println!("This is a Mealy state machine with {} states", m.states.len());
    }
    None => {
        println!("Unknown graph structure");
    }
}

// 3. Apply morphism
let events = k5.apply_morphism(GraphMorphism::SubdivideEdges(2))?;
// K5 now has each edge replaced by a path of length 2

// 4. Create domain model
let address_template = AddressGraphTemplate {
    country: "USA",
    include_apartment: true,
};
let address_graph = GraphAggregate::from_model(
    GraphModel::AddressGraph(address_template)
);
```

This model-based approach ensures our graphs are not just collections of nodes and edges, but meaningful structures with well-defined properties and transformations.
