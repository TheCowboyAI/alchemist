# Graph DDD Example: Order Processing Workflow

## Domain Structure

Let's model an order processing workflow using our DDD graph architecture:

```
Domain<Graph>: "Order Processing System"
│
├── Graph (Aggregate Root): "Main Order Flow"
│   │
│   ├── Subgraph (Entity): "Order Validation"
│   │   ├── Node (Component): "Validate Customer"
│   │   ├── Node (Component): "Check Inventory"
│   │   ├── Node (Component): "Verify Pricing"
│   │   └── Edge (System): "ValidationFlow" (connects nodes)
│   │
│   ├── Subgraph (Entity): "Payment Processing"
│   │   ├── Node (Component): "Calculate Total"
│   │   ├── Node (Component): "Process Payment"
│   │   ├── Node (Component): "Send Receipt"
│   │   └── Edge (System): "PaymentFlow" (connects nodes)
│   │
│   └── Subgraph (Entity): "Order Fulfillment"
│       ├── Node (Component): "Reserve Inventory"
│       ├── Node (Component): "Generate Shipping"
│       ├── Node (Component): "Notify Customer"
│       └── Edge (System): "FulfillmentFlow" (connects nodes)
```

## Implementation

### 1. Create the Domain (Graph Aggregate)

```rust
// Create the aggregate root
let mut order_system = GraphAggregate::new("Order Processing System");

// The Graph maintains consistency across all subgraphs
let graph_id = order_system.root.identity;
```

### 2. Add Subgraph Entities

```rust
// Each subgraph is an entity with its own identity and lifecycle
let validation_subgraph = Subgraph {
    identity: SubgraphId::new(),
    parent_graph: graph_id,
    nodes: HashSet::new(),
    edges: HashSet::new(),
    constraints: SubgraphConstraints {
        max_nodes: 10,
        allowed_node_types: vec!["Validator", "Checker"],
        required_connections: vec!["Input", "Output"],
    },
};

order_system.add_subgraph(validation_subgraph);
```

### 3. Add Nodes (Components) to Subgraphs

```rust
// Nodes are value objects - they don't have identity
let validate_customer = Node {
    content: NodeContent {
        label: "Validate Customer".to_string(),
        node_type: "Validator".to_string(),
        properties: json!({
            "timeout": "30s",
            "retry_count": 3,
            "required_fields": ["customer_id", "email"]
        }),
    },
    position: SpatialPosition::at_3d(0.0, 0.0, 0.0),
    properties: NodeProperties::default(),
};

// Add to subgraph
validation_subgraph.add_node(validate_customer);
```

### 4. Connect with Edges (Systems)

```rust
// Edges represent behavior - HOW nodes communicate
let validation_flow = Edge {
    source: validate_customer_id,
    target: check_inventory_id,
    behavior: EdgeBehavior::EventEmission {
        event_type: "CustomerValidated".to_string(),
    },
    communication_type: CommunicationType::Asynchronous,
};

// This edge means: "When Validate Customer completes,
// it emits a CustomerValidated event that Check Inventory listens for"
validation_subgraph.add_edge(validation_flow);
```

## Entity Communication Through Events

### Inter-Subgraph Communication

```rust
// Subgraphs (entities) communicate through domain events
impl OrderValidationSubgraph {
    fn complete_validation(&mut self, order: Order) -> Result<(), DomainError> {
        // Validate the order...

        // Emit event for other entities
        self.emit_event(GraphDomainEvent::InterSubgraphMessage {
            from_subgraph: self.identity,
            to_subgraph: payment_processing_id,
            message: DomainMessage::ValidationComplete {
                order_id: order.id,
                validation_result: ValidationResult::Approved,
            },
        });

        Ok(())
    }
}

impl PaymentProcessingSubgraph {
    fn handle_message(&mut self, message: DomainMessage) -> Result<(), DomainError> {
        match message {
            DomainMessage::ValidationComplete { order_id, validation_result } => {
                if validation_result == ValidationResult::Approved {
                    self.start_payment_processing(order_id)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

## Petgraph Representation

```rust
// The aggregate is represented in petgraph for efficient operations
pub struct OrderSystemModel {
    // Root graph: subgraphs as nodes
    root: Graph<SubgraphNode, SubgraphRelationship>,

    // Each subgraph has its own graph
    subgraphs: HashMap<SubgraphId, Graph<Node, Edge>>,
}

// Example: Finding all paths through the workflow
pub fn find_order_paths(model: &OrderSystemModel) -> Vec<WorkflowPath> {
    // Use petgraph to find all paths from validation to fulfillment
    let validation_idx = /* ... */;
    let fulfillment_idx = /* ... */;

    all_simple_paths(&model.root, validation_idx, fulfillment_idx, 0, None)
        .map(|path| WorkflowPath::from_indices(path))
        .collect()
}
```

## Event Store Integration

```rust
// Every change is recorded as an event
let event = GraphDomainEvent::SystemConnected {
    subgraph_id: validation_subgraph.identity,
    edge: Edge {
        source: validate_customer_id,
        target: check_inventory_id,
        behavior: EdgeBehavior::EventEmission {
            event_type: "CustomerValidated".to_string(),
        },
    },
};

event_store.append(graph_id, event)?;

// Can replay to any point
let past_state = event_store.replay_until(graph_id, timestamp)?;
```

## Key Concepts Illustrated

1. **Domain = Graph**: The entire order processing system is our domain
2. **Aggregate Root = Graph**: Maintains consistency across all subgraphs
3. **Entities = Subgraphs**: Each has identity and manages its own lifecycle
4. **Components = Nodes**: Value objects representing capabilities
5. **Systems = Edges**: Define behavior and communication patterns
6. **Events**: How entities communicate across boundaries

## Benefits of This Model

1. **Clear Boundaries**: Each subgraph is a consistency boundary
2. **Loose Coupling**: Subgraphs communicate only through events
3. **Flexibility**: Can add/remove subgraphs without affecting others
4. **Traceability**: Event store records all interactions
5. **Testability**: Each subgraph can be tested in isolation

This model allows us to build complex workflows while maintaining DDD principles and leveraging the power of graphs for visualization and analysis.
