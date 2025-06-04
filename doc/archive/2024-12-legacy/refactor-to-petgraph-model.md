# Refactor to Petgraph-Based Model Plan

## Problem Statement
Currently synchronizing 3 separate graph representations:
1. **ECS Entities** - Bevy components (Graph, Node, Edge)
2. **Daggy Storage** - Used for MerkleDAG structures (Event Store, Object Store)
3. **General Graphs** - Workflow graphs, relationship graphs, etc.

This causes:
- Performance overhead from constant synchronization
- Memory waste from duplicate representations
- Complexity from keeping models in sync
- Reinventing graph algorithms that petgraph already provides

## Solution: Specialized Graph Types Architecture

### Core Principle
- **Daggy**: For MerkleDAG structures (Event Store, Object Store) - KEEP AS IS
- **Petgraph**: For general graphs (workflows, relationships, authorization)
- **ECS**: Rendering layer with index references (SIMPLIFY)
- **External Domains**: Adapter pattern for People, Organization, Location, etc.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    External Domains                          │
│  ┌──────────┐  ┌──────────────┐  ┌──────────┐              │
│  │  People  │  │ Organization │  │ Location │  ...          │
│  └──────────┘  └──────────────┘  └──────────┘              │
└─────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │ Domain Adapters   │
                    └─────────┬─────────┘
                              │
┌─────────────────────────────┴─────────────────────────────┐
│                   Graph Management Layer                    │
│                                                            │
│  ┌─────────────────┐        ┌─────────────────┐          │
│  │  MerkleDAGs     │        │ General Graphs  │          │
│  │   (Daggy)       │        │  (Petgraph)     │          │
│  ├─────────────────┤        ├─────────────────┤          │
│  │ • Event Store   │        │ • Workflows     │          │
│  │ • Object Store  │        │ • Relationships │          │
│  │ • Audit Logs    │        │ • Authorization │          │
│  │ • Version DAGs  │        │ • Visualization │          │
│  └─────────────────┘        └─────────────────┘          │
└────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │   ECS Rendering   │
                    │  (Bevy Components)│
                    └───────────────────┘
```

## Implementation Plan

### Phase 1: Clarify Graph Type Separation
**Goal**: Establish clear boundaries between graph types

1. **Keep Daggy for MerkleDAGs**:
```rust
// Event Store remains with Daggy
pub struct EventStore {
    dag: Dag<EventNode, EventEdge>,
    cid_index: HashMap<Cid, DagNodeIndex>,
}

// Object Store remains with Daggy
pub struct ObjectStore {
    dag: Dag<ObjectNode, ObjectEdge>,
    cid_index: HashMap<Cid, DagNodeIndex>,
}
```

2. **Use Petgraph for Workflow Graphs**:
```rust
// Workflow graphs use petgraph
pub struct WorkflowGraph {
    graph: Graph<WorkflowNode, WorkflowEdge>,
    event_links: HashMap<NodeIndex, Cid>, // Links to Event Store
}
```

### Phase 2: External Domain Integration
**Goal**: Create adapters for external business domains

1. **Domain Adapter Trait**:
```rust
pub trait DomainAdapter: Send + Sync {
    fn resolve(&self, entity_ref: &ExternalEntityRef) -> Result<DomainEntity, DomainError>;
    fn query(&self, criteria: QueryCriteria) -> Result<Vec<DomainEntity>, DomainError>;
    fn subscribe(&self, entity_ref: &ExternalEntityRef) -> Result<DomainEventStream, DomainError>;
}
```

2. **External Entity References**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalEntityRef {
    pub domain: DomainType,
    pub entity_type: String,
    pub entity_id: String,
    pub cached_cid: Option<Cid>, // If cached in Object Store
}
```

### Phase 3: Unified Graph Service
**Goal**: Coordinate between different graph types

```rust
pub struct UnifiedGraphService {
    // MerkleDAG stores (Daggy)
    event_store: EventStore,
    object_store: ObjectStore,

    // General graphs (Petgraph)
    workflows: HashMap<WorkflowId, WorkflowGraph>,
    relationships: HashMap<RelationshipId, RelationshipGraph>,

    // External domains
    domains: ExternalDomainRefs,
}
```

## Business Document Management Example

### Domain Structure

```
Domain<Graph>: "Document Management System"
│
├── Graph (Aggregate Root): "Document Lifecycle"
│   │
│   ├── Subgraph (Entity): "Document Creation"
│   │   ├── Node: "Draft Document"
│   │   ├── Node: "Add Metadata"
│   │   ├── Node: "Attach Files"
│   │   └── Edge: "CreationFlow" (sequential workflow)
│   │
│   ├── Subgraph (Entity): "Review Process"
│   │   ├── Node: "Legal Review"
│   │   ├── Node: "Technical Review"
│   │   ├── Node: "Management Review"
│   │   └── Edge: "ParallelReview" (concurrent reviews)
│   │
│   ├── Subgraph (Entity): "Authorization Chain"
│   │   ├── Node: "Department Head"
│   │   ├── Node: "Division Manager"
│   │   ├── Node: "Executive Approval"
│   │   └── Edge: "HierarchicalApproval" (sequential authorization)
│   │
│   └── Subgraph (Entity): "Document Distribution"
│       ├── Node: "Internal Publishing"
│       ├── Node: "External Sharing"
│       ├── Node: "Archive Storage"
│       └── Edge: "DistributionRules" (conditional routing)
```

### Implementation with DDD + Petgraph

```rust
// Domain Model
pub struct DocumentManagementDomain {
    aggregate: GraphAggregate,
    petgraph_model: DocumentGraphModel,
    event_store: EventStore,
}

// Petgraph representation
pub struct DocumentGraphModel {
    // Root graph: workflow stages as nodes
    workflow: Graph<WorkflowStage, StageTransition>,
    // Each stage has its own graph
    stages: HashMap<StageId, Graph<DocumentNode, DocumentEdge>>,
}

// Document nodes in petgraph
#[derive(Debug, Clone)]
pub struct DocumentNode {
    pub node_type: DocumentNodeType,
    pub permissions: PermissionSet,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Clone)]
pub enum DocumentNodeType {
    DraftDocument { template: String },
    ReviewPoint { reviewer_role: String, sla_hours: u32 },
    ApprovalGate { authority_level: AuthorityLevel },
    DistributionChannel { channel_type: ChannelType },
}

// Edges represent workflow rules
#[derive(Debug, Clone)]
pub struct DocumentEdge {
    pub flow_type: FlowType,
    pub conditions: Vec<WorkflowCondition>,
    pub authorization_required: AuthorizationRequirement,
}

#[derive(Debug, Clone)]
pub enum FlowType {
    Sequential,              // One after another
    Parallel,               // All at once
    Conditional { rules: Vec<BusinessRule> }, // Based on conditions
    Hierarchical { escalation_path: Vec<AuthorityLevel> }, // Up the chain
}
```

### Document Workflow Implementation

```rust
pub fn create_document_workflow(
    domain: &mut DocumentManagementDomain,
) -> Result<WorkflowId, DomainError> {
    // 1. Create Document Creation subgraph
    let creation_stage = domain.add_workflow_stage("Document Creation")?;

    // Add nodes using petgraph
    let draft_node = domain.petgraph_model.stages
        .get_mut(&creation_stage)
        .unwrap()
        .add_node(DocumentNode {
            node_type: DocumentNodeType::DraftDocument {
                template: "contract_template_v2".to_string(),
            },
            permissions: PermissionSet::new()
                .with_role("author", Permission::Write)
                .with_role("reviewer", Permission::Read),
            metadata: DocumentMetadata::new("Contract Draft"),
        });

    // 2. Create Review Process with parallel reviews
    let review_stage = domain.add_workflow_stage("Review Process")?;

    let legal_review = add_review_node(&mut domain, review_stage, "Legal", 24)?;
    let tech_review = add_review_node(&mut domain, review_stage, "Technical", 48)?;
    let mgmt_review = add_review_node(&mut domain, review_stage, "Management", 72)?;

    // Connect reviews in parallel using petgraph
    let review_graph = domain.petgraph_model.stages.get_mut(&review_stage).unwrap();

    // All reviews happen in parallel
    review_graph.add_edge(
        draft_node,
        legal_review,
        DocumentEdge {
            flow_type: FlowType::Parallel,
            conditions: vec![WorkflowCondition::DocumentComplete],
            authorization_required: AuthorizationRequirement::None,
        },
    );

    // 3. Create Authorization Chain
    let auth_stage = domain.add_workflow_stage("Authorization Chain")?;

    // Use petgraph to find shortest authorization path
    let auth_path = find_authorization_path(
        &domain.petgraph_model,
        DocumentType::Contract,
        ContractValue::High,
    )?;

    // Build authorization chain
    build_authorization_chain(&mut domain, auth_stage, auth_path)?;

    // 4. Record in Event Store
    domain.event_store.append_with_payload(
        domain.aggregate.root.identity.0,
        "WorkflowCreated".to_string(),
        EventPayload {
            data: json!({
                "workflow_type": "document_management",
                "stages": ["creation", "review", "authorization", "distribution"],
                "created_by": "system_admin",
            }),
            created_at: SystemTime::now(),
        },
    )?;

    Ok(WorkflowId::new())
}
```

### Authorization Using Graph Algorithms

```rust
/// Find optimal authorization path using petgraph
pub fn find_authorization_path(
    model: &DocumentGraphModel,
    doc_type: DocumentType,
    value: ContractValue,
) -> Result<Vec<NodeIndex>, DomainError> {
    let auth_graph = &model.workflow;

    // Find starting point based on document type
    let start_node = auth_graph
        .node_indices()
        .find(|&idx| {
            matches!(
                auth_graph[idx],
                WorkflowStage::Authorization { level: AuthorityLevel::DepartmentHead, .. }
            )
        })
        .ok_or(DomainError::NoAuthorizationPath)?;

    // Find required end point based on value
    let end_node = match value {
        ContractValue::Low => start_node, // Department head sufficient
        ContractValue::Medium => find_node_by_authority(auth_graph, AuthorityLevel::DivisionManager)?,
        ContractValue::High => find_node_by_authority(auth_graph, AuthorityLevel::Executive)?,
    };

    // Use Dijkstra to find shortest path
    let path = dijkstra(auth_graph, start_node, Some(end_node), |e| {
        // Weight based on approval time and authority level
        e.weight().estimated_hours as i32
    });

    // Convert to path
    reconstruct_path(path, start_node, end_node)
}

/// Check document access using graph traversal
pub fn check_document_access(
    model: &DocumentGraphModel,
    user: &User,
    document: DocumentId,
) -> Result<AccessLevel, AuthorizationError> {
    // Find document node
    let doc_node = find_document_node(model, document)?;

    // Use BFS to find all accessible nodes from user's position
    let user_position = find_user_position(model, user)?;
    let accessible = bfs_reach(model, user_position);

    if accessible.contains(&doc_node) {
        // Check permission level on the edge
        let edge = model.find_edge(user_position, doc_node)?;
        Ok(edge.weight().access_level)
    } else {
        Err(AuthorizationError::NoAccess)
    }
}
```

### Event Sourcing Integration

```rust
/// Replay document workflow to any point
pub fn replay_document_history(
    event_store: &EventStore,
    document_id: DocumentId,
    target_time: Option<SystemTime>,
) -> Result<DocumentGraphModel, DomainError> {
    let mut model = DocumentGraphModel::new();

    // Get events for this document
    let events = if let Some(time) = target_time {
        event_store.get_events_until(document_id.0, time)?
    } else {
        event_store.get_events_for_aggregate(document_id.0)?
    };

    // Replay each event
    for event in events {
        match event.event_type.as_str() {
            "DocumentCreated" => replay_document_creation(&mut model, &event)?,
            "ReviewRequested" => replay_review_request(&mut model, &event)?,
            "ApprovalGranted" => replay_approval(&mut model, &event)?,
            "DocumentDistributed" => replay_distribution(&mut model, &event)?,
            _ => {}
        }
    }

    Ok(model)
}

/// Animate document flow through workflow
pub fn animate_document_journey(
    model: &DocumentGraphModel,
    document_id: DocumentId,
    event_store: &EventStore,
) -> DocumentAnimation {
    let events = event_store.get_events_for_aggregate(document_id.0).unwrap();

    DocumentAnimation {
        events,
        current_index: 0,
        speed: 1.0,
        timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        // Highlight path in petgraph as animation progresses
        current_path: vec![],
    }
}
```

## Benefits of This Architecture

1. **Right Tool for Right Job**:
   - Daggy optimized for MerkleDAG operations
   - Petgraph optimized for graph algorithms
   - Clear separation of concerns

2. **External Domain Integration**:
   - Clean adapter pattern for external systems
   - Caching in Object Store when needed
   - Validation of external references

3. **Event Sourcing Preserved**:
   - Complete history in Event Store (Daggy)
   - Replay capabilities unchanged
   - CID-based content addressing

4. **Performance**:
   - No unnecessary synchronization
   - Each graph type optimized for its use case
   - Efficient algorithms where needed

## Migration Strategy

1. **Week 1**:
   - Document clear boundaries between graph types
   - Create external domain adapters
   - Test with existing systems

2. **Week 2**:
   - Implement unified graph service
   - Migrate workflow graphs to petgraph
   - Keep Event/Object stores with Daggy

3. **Week 3**:
   - Add caching for external entities
   - Optimize graph algorithms
   - Performance testing

## Success Criteria

- [ ] Event Store and Object Store continue using Daggy
- [ ] Workflow graphs migrated to petgraph
- [ ] External domains integrated via adapters
- [ ] No performance degradation
- [ ] Clear separation of graph types
- [ ] All tests passing

## Next Steps

1. Review current Daggy usage in Event/Object stores
2. Design petgraph schema for workflow graphs
3. Create domain adapter implementations
4. Build unified graph service

This architecture leverages the strengths of each graph library while maintaining clean boundaries and supporting external domain integration.
