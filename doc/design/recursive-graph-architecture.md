# Recursive Graph Architecture in CIM

## Core Concept

In CIM, **everything is a graph**, and graphs can contain other graphs. This creates a fractal, recursive structure where:

- A **ContextGraph** is the root abstraction
- Each node in a graph can itself be a graph (via Subgraph component)
- Every DDD component is represented by a specifically shaped graph

## DDD Components as Graphs

### Entity Graph
```
Entity(Invoice)
├── Identity Node (InvoiceId: "INV-001")
├── Attribute Nodes
│   ├── Date: "2024-01-15"
│   └── Status: "Pending"
└── Relationship Nodes
    ├── Buyer → Graph(Party)
    └── Seller → Graph(Party)
```

### Value Object Graph
```
ValueObject(Money)
├── Amount: 100.00
└── Currency: "USD"
(Immutable - no identity node)
```

### Aggregate Graph
```
Aggregate(Order)
├── Root: Entity(Order)
├── LineItems: Collection[Entity(LineItem)]
└── ShippingAddress: ValueObject(Address)
```

### Event Graph
```
Event(InvoiceCreated)
├── AggregateId: "INV-001"
├── Timestamp: "2024-01-15T10:00:00Z"
├── Actor: Reference(User, "user-123")
└── Data
    ├── Buyer: Reference(Party, "party-456")
    └── Amount: ValueObject(Money)
```

## Recursive Example: Invoice

An Invoice is a ContextGraph that contains:

```
ContextGraph(Invoice)
├── Node: Buyer
│   └── Content: Graph(Party)
│       ├── Node: Name (ValueObject)
│       ├── Node: TaxId (ValueObject)
│       └── Node: Address (ValueObject)
├── Node: Seller
│   └── Content: Graph(Party)
├── Node: LineItems
│   └── Content: Graph(Collection)
│       ├── Node: Item1
│       │   └── Content: Graph(LineItem)
│       │       ├── Node: Product (Reference)
│       │       ├── Node: Quantity (ValueObject)
│       │       └── Node: Price (ValueObject)
│       └── Node: Item2
│           └── Content: Graph(LineItem)
└── Node: Totals
    └── Content: Graph(InvoiceTotals)
        ├── Node: Subtotal (ValueObject)
        ├── Node: Tax (ValueObject)
        └── Node: Total (ValueObject)
```

## Implementation

```rust
// In ContextGraph, recursion is achieved through the Subgraph component:
pub struct Subgraph<N, E> {
    pub graph: Box<ContextGraph<N, E>>,
}

// Example: Creating a recursive structure
let mut outer = ContextGraph::<String, String>::new("OuterGraph");
let mut inner = ContextGraph::<String, String>::new("InnerGraph");

// Add the inner graph as a subgraph component to a node
let container_node = outer.add_node("Container".to_string());
outer.get_node_mut(container_node)?
    .components.add(Subgraph { graph: Box::new(inner) })?;
```

## Benefits

1. **Uniform Representation**: Everything uses the same graph structure
2. **Composability**: Graphs compose naturally into larger graphs
3. **Fractal Nature**: Same patterns at every level of abstraction
4. **Cross-Domain References**: Easy to link concepts across domains
5. **Visualization**: Natural mapping to visual graph representations

## Key Insight

This recursive design means that whether you're looking at:
- A single value object (small graph)
- An entity (medium graph with identity)
- An aggregate (large graph with consistency boundary)
- An entire bounded context (huge graph of graphs)

...you're always working with the same fundamental structure: a graph that can contain other graphs.
