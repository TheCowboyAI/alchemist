# GraphComposition Refinement

## Core Refinements Needed

### 1. Composition Algebra

The heart of GraphComposition is its algebraic structure. We need to define:

#### Composition Laws
```rust
// Associativity: (A ∘ B) ∘ C = A ∘ (B ∘ C)
assert_eq!(
    a.compose(&b)?.compose(&c)?,
    a.compose(&b.compose(&c)?)?
);

// Identity: A ∘ id = id ∘ A = A
assert_eq!(
    a.compose(&GraphComposition::identity())?,
    a
);
assert_eq!(
    GraphComposition::identity().compose(&a)?,
    a
);

// Distributivity (for parallel composition)
// A ∘ (B | C) = (A ∘ B) | (A ∘ C)
assert_eq!(
    a.compose(&b.parallel(&c)?)?,
    a.compose(&b)?.parallel(&a.compose(&c)?)?
);
```

#### Composition Types
1. **Sequential (∘)**: Output of A feeds into input of B
2. **Parallel (|)**: A and B execute independently
3. **Choice (+)**: Either A or B based on condition
4. **Loop (*)**: Repeat A while condition holds
5. **Product (×)**: Cartesian product of graphs
6. **Coproduct (⊕)**: Disjoint union of graphs

### 2. Node and Edge Semantics

#### Node Types
```rust
pub enum CompositionNode {
    /// Atomic value - no internal structure
    Atom {
        id: NodeId,
        value: serde_json::Value,
        node_type: String,
    },

    /// Reference to another graph
    GraphRef {
        id: NodeId,
        graph_id: GraphId,
        interface: GraphInterface,
    },

    /// Computation/transformation
    Function {
        id: NodeId,
        input_type: TypeSignature,
        output_type: TypeSignature,
        computation: Box<dyn Fn(Value) -> Value>,
    },

    /// Control flow
    Control {
        id: NodeId,
        control_type: ControlType,
        branches: Vec<EdgeId>,
    },
}

pub enum ControlType {
    Branch { condition: Predicate },
    Merge,
    Loop { condition: Predicate },
    Parallel,
}
```

#### Edge Types
```rust
pub enum CompositionEdge {
    /// Data flow
    DataFlow {
        id: EdgeId,
        source: NodeId,
        target: NodeId,
        data_type: TypeSignature,
    },

    /// Control flow
    ControlFlow {
        id: EdgeId,
        source: NodeId,
        target: NodeId,
        condition: Option<Predicate>,
    },

    /// Structural relationship
    Structural {
        id: EdgeId,
        source: NodeId,
        target: NodeId,
        relationship: RelatedBy,
    },

    /// Temporal ordering
    Temporal {
        id: EdgeId,
        source: NodeId,
        target: NodeId,
        timing: TimingConstraint,
    },
}
```

### 3. Type System for Graphs

We need a type system that ensures composition safety:

```rust
pub struct GraphType {
    /// Input interface
    pub inputs: Vec<PortType>,

    /// Output interface
    pub outputs: Vec<PortType>,

    /// Internal constraints
    pub constraints: Vec<TypeConstraint>,

    /// Composition compatibility
    pub compatible_with: Vec<GraphType>,
}

pub struct PortType {
    pub name: String,
    pub data_type: DataType,
    pub cardinality: Cardinality,
    pub required: bool,
}

pub enum Cardinality {
    One,
    Optional,
    Many,
    AtLeastOne,
}

/// Type checking for composition
impl GraphComposition {
    pub fn check_composition(&self, other: &GraphComposition) -> Result<(), TypeError> {
        // Verify output types of self match input types of other
        for (output, input) in self.type_sig.outputs.iter().zip(&other.type_sig.inputs) {
            if !output.is_compatible_with(input) {
                return Err(TypeError::IncompatibleTypes {
                    output: output.clone(),
                    input: input.clone(),
                });
            }
        }
        Ok(())
    }
}
```

### 4. Invariant System

Graphs can maintain invariants that must hold after any operation:

```rust
pub struct GraphInvariant {
    pub name: String,
    pub predicate: Box<dyn Fn(&GraphComposition) -> bool>,
    pub error_message: String,
}

impl GraphComposition {
    pub fn add_invariant(&mut self, invariant: GraphInvariant) {
        self.invariants.push(invariant);
    }

    pub fn check_invariants(&self) -> Result<(), InvariantViolation> {
        for invariant in &self.invariants {
            if !(invariant.predicate)(self) {
                return Err(InvariantViolation {
                    invariant_name: invariant.name.clone(),
                    message: invariant.error_message.clone(),
                });
            }
        }
        Ok(())
    }

    /// Any operation that modifies the graph must check invariants
    pub fn safe_compose(&mut self, other: &GraphComposition) -> Result<(), CompositionError> {
        let result = self.compose(other)?;
        result.check_invariants()?;
        *self = result;
        Ok(())
    }
}
```

### 5. Composition Patterns Library

Common patterns that should be built-in:

```rust
pub mod patterns {
    /// Map-Reduce pattern
    pub fn map_reduce<M, R, T>(
        map_fn: M,
        reduce_fn: R,
        initial: T,
    ) -> GraphComposition
    where
        M: Fn(&CompositionNode) -> T,
        R: Fn(T, T) -> T,
    {
        GraphComposition::composite("MapReduce")
            .add_node("mapper", Function::new(map_fn))
            .add_node("reducer", Function::new(reduce_fn))
            .add_edge("mapper", "reducer", DataFlow)
    }

    /// Pipeline pattern
    pub fn pipeline(stages: Vec<GraphComposition>) -> GraphComposition {
        stages.into_iter()
            .reduce(|acc, stage| acc.then(stage))
            .unwrap_or_else(GraphComposition::identity)
    }

    /// Fork-Join pattern
    pub fn fork_join(branches: Vec<GraphComposition>) -> GraphComposition {
        let fork = GraphComposition::control(ControlType::Parallel);
        let join = GraphComposition::control(ControlType::Merge);

        branches.into_iter()
            .fold(fork, |acc, branch| acc.add_branch(branch))
            .then(join)
    }

    /// Event Sourcing pattern
    pub fn event_sourced(
        command_handler: GraphComposition,
        event_store: GraphComposition,
        projections: Vec<GraphComposition>,
    ) -> GraphComposition {
        command_handler
            .then(event_store)
            .then(fork_join(projections))
    }
}
```

### 6. Visualization and Debugging

GraphComposition should be introspectable:

```rust
impl GraphComposition {
    /// Generate DOT format for Graphviz
    pub fn to_dot(&self) -> String {
        // Generate visualization
    }

    /// Trace execution path
    pub fn trace_execution(&self, input: Value) -> ExecutionTrace {
        // Record path through graph
    }

    /// Validate structure
    pub fn validate(&self) -> ValidationReport {
        ValidationReport {
            has_cycles: self.detect_cycles(),
            unreachable_nodes: self.find_unreachable_nodes(),
            type_errors: self.check_types(),
            invariant_violations: self.check_invariants().err(),
        }
    }
}
```

### 7. Persistence and Serialization

GraphComposition must be persistable:

```rust
impl GraphComposition {
    /// Serialize to IPLD
    pub fn to_ipld(&self) -> ipld::Ipld {
        // Convert to IPLD representation
    }

    /// Generate CID
    pub fn cid(&self) -> Cid {
        // Content-addressed identifier
    }

    /// Create from IPLD
    pub fn from_ipld(ipld: ipld::Ipld) -> Result<Self, DeserializationError> {
        // Reconstruct from IPLD
    }
}
```

## Priority Implementation Order

1. **Core Structure** (Week 1)
   - GraphComposition struct
   - CompositionNode and CompositionEdge enums
   - Basic composition operations (then, parallel)

2. **Type System** (Week 2)
   - GraphType definition
   - Type checking for composition
   - Port compatibility

3. **Composition Algebra** (Week 3)
   - Implement all composition operators
   - Verify algebraic laws
   - Add composition patterns

4. **Invariants** (Week 4)
   - Invariant system
   - Safe composition operations
   - Validation framework

5. **Advanced Features** (Week 5+)
   - Visualization
   - Persistence
   - Performance optimization

## Key Design Decisions

1. **Immutability by Default** - Composition operations return new graphs
2. **Type Safety** - Compositions checked at compile time where possible
3. **Lazy Evaluation** - Graphs describe computation, execution is separate
4. **Extensibility** - Easy to add new node/edge types and patterns
5. **Performance** - Use Rust's zero-cost abstractions

This refinement positions GraphComposition as the foundational abstraction for building complex, composable systems where everything is a graph.
