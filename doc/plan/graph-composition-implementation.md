# GraphComposition Implementation Plan

## Overview

This plan outlines the concrete steps to implement GraphComposition as our foundational abstraction, replacing ContentGraph with a more powerful compositional algebra.

## Phase 1: Core Structure (Days 1-3)

### Day 1: Basic Types and Traits

1. **Create core types**
   ```rust
   // src/domain/value_objects/graph_composition.rs
   pub struct GraphComposition {
       pub id: GraphId,
       pub composition_root: NodeId,
       pub composition_type: CompositionType,
       pub nodes: HashMap<NodeId, CompositionNode>,
       pub edges: HashMap<EdgeId, CompositionEdge>,
       pub metadata: CompositionMetadata,
       pub lazy_cid: LazyCid<GraphComposition>,
   }
   ```

2. **Define composition types**
   ```rust
   // src/domain/value_objects/composition_type.rs
   pub enum CompositionType {
       Atomic { value_type: String },
       Composite { structure_type: String },
       Domain(DomainCompositionType),
   }
   ```

3. **Create node and edge types**
   ```rust
   // src/domain/value_objects/composition_node.rs
   pub enum CompositionNode {
       Atom { id: NodeId, value: Value, node_type: String },
       GraphRef { id: NodeId, graph_id: GraphId },
   }
   ```

### Day 2: Basic Operations

1. **Implement Composable trait**
   ```rust
   pub trait Composable {
       type Output;
       fn compose(&self, other: &Self) -> Result<Self::Output, CompositionError>;
       fn can_compose_with(&self, other: &Self) -> bool;
   }
   ```

2. **Add sequential composition**
   ```rust
   impl GraphComposition {
       pub fn then(&self, other: &GraphComposition) -> Result<GraphComposition, CompositionError>
   }
   ```

3. **Add parallel composition**
   ```rust
   impl GraphComposition {
       pub fn parallel(&self, other: &GraphComposition) -> Result<GraphComposition, CompositionError>
   }
   ```

### Day 3: Events and Commands

1. **Create GraphComposition events**
   ```rust
   // src/domain/events/graph_composition.rs
   pub enum GraphCompositionEvent {
       GraphCompositionCreated { ... },
       NodesComposed { ... },
       CompositionTypeChanged { ... },
   }
   ```

2. **Create commands**
   ```rust
   // src/domain/commands/graph_composition.rs
   pub enum GraphCompositionCommand {
       CreateGraphComposition { ... },
       ComposeGraphs { ... },
       SetCompositionRoot { ... },
   }
   ```

## Phase 2: Migration (Days 4-5)

### Day 4: Rename and Refactor

1. **Rename ContentGraph to GraphComposition**
   - Update all file names
   - Update all type references
   - Update all imports

2. **Add composition_type field**
   - Migrate existing graphs to Composite type
   - Update constructors

3. **Update events and commands**
   - Rename ContentGraphEvent to GraphCompositionEvent
   - Update event handlers

### Day 5: Update Dependencies

1. **Update aggregates**
   - Graph aggregate to use GraphComposition
   - Update all aggregate methods

2. **Update projections**
   - Modify read models for GraphComposition
   - Update query handlers

3. **Update tests**
   - Rename test files
   - Update test cases

## Phase 3: Type System (Days 6-8)

### Day 6: Graph Types

1. **Define GraphType**
   ```rust
   pub struct GraphType {
       pub inputs: Vec<PortType>,
       pub outputs: Vec<PortType>,
       pub constraints: Vec<TypeConstraint>,
   }
   ```

2. **Implement type checking**
   ```rust
   impl GraphComposition {
       pub fn check_composition(&self, other: &GraphComposition) -> Result<(), TypeError>
   }
   ```

### Day 7: Port System

1. **Create PortType**
   ```rust
   pub struct PortType {
       pub name: String,
       pub data_type: DataType,
       pub cardinality: Cardinality,
   }
   ```

2. **Add port matching**
   - Input/output compatibility
   - Cardinality checking

### Day 8: Type Inference

1. **Implement type inference**
   - Infer types from composed graphs
   - Propagate type constraints

## Phase 4: Advanced Composition (Days 9-12)

### Day 9: Choice and Loop

1. **Add choice composition**
   ```rust
   pub fn choice(&self, other: &GraphComposition) -> Result<GraphComposition, CompositionError>
   ```

2. **Add loop composition**
   ```rust
   pub fn loop_while<F>(&self, condition: F) -> Result<GraphComposition, CompositionError>
   ```

### Day 10: Functors and Monads

1. **Implement GraphFunctor**
   ```rust
   pub trait GraphFunctor {
       fn fmap<F>(&self, f: F) -> Result<GraphComposition, FunctorError>
   }
   ```

2. **Implement GraphMonad**
   ```rust
   pub trait GraphMonad {
       fn pure(value: CompositionNode) -> GraphComposition;
       fn bind<F>(&self, f: F) -> Result<GraphComposition, MonadError>
   }
   ```

### Day 11: Composition Patterns

1. **Create pattern library**
   - Map-Reduce pattern
   - Pipeline pattern
   - Fork-Join pattern
   - Event Sourcing pattern

### Day 12: Invariants

1. **Implement invariant system**
   ```rust
   pub struct GraphInvariant {
       pub name: String,
       pub predicate: Box<dyn Fn(&GraphComposition) -> bool>,
   }
   ```

2. **Add safe composition**
   - Check invariants after composition
   - Rollback on violation

## Phase 5: Integration (Days 13-15)

### Day 13: Bevy Integration

1. **Update Bevy components**
   - GraphCompositionEntity component
   - Update visualization systems

2. **Create composition UI**
   - Drag-and-drop composition
   - Visual feedback

### Day 14: NATS Integration

1. **Update event subjects**
   - graph.composition.* subjects
   - Update event routing

2. **Add composition events**
   - Publish composition operations
   - Subscribe to remote compositions

### Day 15: Testing and Documentation

1. **Create comprehensive tests**
   - Unit tests for each operation
   - Integration tests for composition
   - Property-based tests for laws

2. **Update documentation**
   - API documentation
   - Usage examples
   - Migration guide

## Success Criteria

1. **All existing functionality works** with GraphComposition
2. **Composition operations** are type-safe and verified
3. **Algebraic laws** are tested and enforced
4. **Performance** is equal or better than ContentGraph
5. **Documentation** is complete and clear

## Risk Mitigation

1. **Backward Compatibility**
   - Keep ContentGraph as alias initially
   - Gradual migration path

2. **Performance**
   - Benchmark each operation
   - Optimize hot paths

3. **Complexity**
   - Start with simple compositions
   - Add advanced features incrementally

## Next Steps After Implementation

1. **Demo Applications**
   - Show composition in action
   - Real-world use cases

2. **Performance Optimization**
   - Profile and optimize
   - Add caching where needed

3. **Advanced Features**
   - Visual composition editor
   - Composition debugging tools
   - Composition versioning
