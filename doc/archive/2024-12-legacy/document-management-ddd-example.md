# Business Document Management - DDD Graph Example

## Overview

This example demonstrates how to model a complete Business Document Management system using our DDD graph architecture with petgraph for efficient operations and event sourcing for audit trails.

## Domain Model

```
Domain<Graph>: "Business Document Management"
│
├── Aggregate Root: "Document Lifecycle Graph"
│   ├── Identity: GraphId (UUID)
│   ├── Metadata: System configuration, policies
│   └── Subgraph Registry: All workflow stages
│
├── Entities (Subgraphs):
│   ├── "Document Creation" - Draft and prepare documents
│   ├── "Review Process" - Multi-party review workflows
│   ├── "Authorization Chain" - Hierarchical approvals
│   ├── "Distribution" - Publishing and sharing
│   └── "Archival" - Long-term storage and retention
│
├── Value Objects (Nodes):
│   ├── Document states (Draft, Under Review, Approved, etc.)
│   ├── Review checkpoints
│   ├── Approval gates
│   └── Distribution channels
│
└── Systems (Edges):
    ├── Sequential flows (one after another)
    ├── Parallel flows (concurrent activities)
    ├── Conditional routing (based on rules)
    └── Hierarchical escalation (up the chain)
```

## Implementation

### 1. Domain Aggregate Structure

```rust
use petgraph::graph::{Graph, NodeIndex, EdgeIndex};
use std::collections::{HashMap, HashSet};

/// The Document Management Aggregate
pub struct DocumentManagementAggregate {
    /// Aggregate root
    pub root: DocumentLifecycleGraph,
    /// All workflow stages (entities)
    pub stages: HashMap<StageId, WorkflowStage>,
    /// Business policies
    pub policies: DocumentPolicies,
    /// Petgraph representation
    pub graph_model: DocumentGraphModel,
}

/// The Aggregate Root
pub struct DocumentLifecycleGraph {
    pub identity: GraphId,
    pub name: String,
    pub organization: OrganizationId,
    pub stage_registry: HashSet<StageId>,
    pub active_documents: HashMap<DocumentId, DocumentState>,
}

/// Workflow Stage Entity
#[derive(Clone, Debug)]
pub struct WorkflowStage {
    pub identity: StageId,
    pub stage_type: StageType,
    pub parent_graph: GraphId,
    pub nodes: HashSet<NodeId>,
    pub edges: HashSet<EdgeId>,
    pub constraints: StageConstraints,
}

#[derive(Clone, Debug)]
pub enum StageType {
    Creation,
    Review { review_types: Vec<ReviewType> },
    Authorization { levels: Vec<AuthorityLevel> },
    Distribution { channels: Vec<DistributionChannel> },
    Archival { retention_policy: RetentionPolicy },
}
```

### 2. Petgraph Model

```rust
/// Petgraph representation for efficient operations
pub struct DocumentGraphModel {
    /// Main workflow graph
    pub workflow: Graph<StageNode, StageTransition>,
    /// Individual stage graphs
    pub stages: HashMap<StageId, Graph<DocumentNode, WorkflowEdge>>,
    /// Index mappings for fast lookups
    pub node_index: HashMap<NodeId, (StageId, NodeIndex)>,
    pub document_locations: HashMap<DocumentId, NodeIndex>,
}

/// Node in the workflow graph
#[derive(Debug, Clone)]
pub struct DocumentNode {
    pub id: NodeId,
    pub node_type: DocumentNodeType,
    pub permissions: PermissionSet,
    pub metadata: NodeMetadata,
    pub state: NodeState,
}

#[derive(Debug, Clone)]
pub enum DocumentNodeType {
    DraftDocument {
        template: TemplateId,
        required_fields: Vec<FieldRequirement>,
    },
    ReviewCheckpoint {
        reviewer_role: Role,
        review_type: ReviewType,
        sla_hours: u32,
    },
    ApprovalGate {
        authority_level: AuthorityLevel,
        delegation_allowed: bool,
        auto_escalation: Option<Duration>,
    },
    DistributionPoint {
        channel: DistributionChannel,
        access_control: AccessControlList,
    },
    ArchiveNode {
        retention_years: u32,
        compliance_tags: Vec<ComplianceTag>,
    },
}

/// Edge representing workflow behavior
#[derive(Debug, Clone)]
pub struct WorkflowEdge {
    pub edge_type: EdgeType,
    pub behavior: EdgeBehavior,
    pub conditions: Vec<WorkflowCondition>,
    pub authorization: AuthorizationRequirement,
}

#[derive(Debug, Clone)]
pub enum EdgeBehavior {
    /// Sequential progression
    Sequential {
        can_skip: bool,
        timeout: Option<Duration>,
    },
    /// Parallel execution
    Parallel {
        wait_for_all: bool,
        minimum_required: usize,
    },
    /// Conditional routing
    Conditional {
        rules: Vec<BusinessRule>,
        default_path: Option<NodeIndex>,
    },
    /// Hierarchical escalation
    Hierarchical {
        escalation_levels: Vec<AuthorityLevel>,
        escalation_timeout: Duration,
    },
}
```

### 3. Document Workflow Creation

```rust
impl DocumentManagementAggregate {
    /// Create a new document workflow
    pub fn create_contract_workflow(&mut self) -> Result<WorkflowId, DomainError> {
        let workflow_id = WorkflowId::new();

        // 1. Document Creation Stage
        let creation_stage = self.add_stage(WorkflowStage {
            identity: StageId::new(),
            stage_type: StageType::Creation,
            parent_graph: self.root.identity,
            nodes: HashSet::new(),
            edges: HashSet::new(),
            constraints: StageConstraints {
                max_duration: Duration::from_hours(24),
                required_fields: vec!["title", "author", "department"],
                allowed_templates: vec!["contract_v2", "contract_simple"],
            },
        })?;

        // Add creation nodes
        let draft_node = self.add_node_to_stage(
            creation_stage,
            DocumentNode {
                id: NodeId::new(),
                node_type: DocumentNodeType::DraftDocument {
                    template: TemplateId::from("contract_v2"),
                    required_fields: vec![
                        FieldRequirement::Text("title", 1..200),
                        FieldRequirement::Text("parties", 2..10),
                        FieldRequirement::Money("contract_value", 0..),
                        FieldRequirement::Date("effective_date", DateRange::Future),
                    ],
                },
                permissions: PermissionSet::new()
                    .grant(Role::Author, Permission::Write)
                    .grant(Role::LegalTeam, Permission::Read),
                metadata: NodeMetadata::default(),
                state: NodeState::Active,
            },
        )?;

        // 2. Review Stage with Parallel Reviews
        let review_stage = self.add_stage(WorkflowStage {
            identity: StageId::new(),
            stage_type: StageType::Review {
                review_types: vec![
                    ReviewType::Legal,
                    ReviewType::Financial,
                    ReviewType::Technical,
                ],
            },
            parent_graph: self.root.identity,
            nodes: HashSet::new(),
            edges: HashSet::new(),
            constraints: StageConstraints {
                max_duration: Duration::from_hours(72),
                required_fields: vec!["review_comments", "risk_assessment"],
                allowed_templates: vec![],
            },
        })?;

        // Add parallel review nodes
        let legal_review = self.add_review_node(review_stage, ReviewType::Legal, 24)?;
        let financial_review = self.add_review_node(review_stage, ReviewType::Financial, 48)?;
        let technical_review = self.add_review_node(review_stage, ReviewType::Technical, 48)?;

        // Connect with parallel edges
        self.connect_parallel_reviews(
            draft_node,
            vec![legal_review, financial_review, technical_review],
        )?;

        // 3. Authorization Chain
        let auth_stage = self.create_authorization_chain(
            ContractValue::from_amount(1_000_000), // High value contract
        )?;

        // 4. Distribution Stage
        let distribution_stage = self.create_distribution_stage()?;

        // Connect stages in workflow
        self.connect_workflow_stages(vec![
            creation_stage,
            review_stage,
            auth_stage,
            distribution_stage,
        ])?;

        Ok(workflow_id)
    }

    /// Create authorization chain based on document value
    fn create_authorization_chain(
        &mut self,
        contract_value: ContractValue,
    ) -> Result<StageId, DomainError> {
        let auth_stage = self.add_stage(WorkflowStage {
            identity: StageId::new(),
            stage_type: StageType::Authorization {
                levels: self.determine_required_levels(contract_value),
            },
            parent_graph: self.root.identity,
            nodes: HashSet::new(),
            edges: HashSet::new(),
            constraints: StageConstraints {
                max_duration: Duration::from_hours(120),
                required_fields: vec!["approval_notes"],
                allowed_templates: vec![],
            },
        })?;

        // Build hierarchical approval chain
        let mut previous_node = None;
        let levels = self.determine_required_levels(contract_value);

        for level in levels {
            let approval_node = self.add_node_to_stage(
                auth_stage,
                DocumentNode {
                    id: NodeId::new(),
                    node_type: DocumentNodeType::ApprovalGate {
                        authority_level: level,
                        delegation_allowed: level != AuthorityLevel::Executive,
                        auto_escalation: Some(Duration::from_hours(48)),
                    },
                    permissions: self.get_permissions_for_level(level),
                    metadata: NodeMetadata::default(),
                    state: NodeState::Active,
                },
            )?;

            if let Some(prev) = previous_node {
                // Connect hierarchically
                self.add_edge_to_stage(
                    auth_stage,
                    prev,
                    approval_node,
                    WorkflowEdge {
                        edge_type: EdgeType::Authorization,
                        behavior: EdgeBehavior::Hierarchical {
                            escalation_levels: vec![level],
                            escalation_timeout: Duration::from_hours(48),
                        },
                        conditions: vec![
                            WorkflowCondition::PreviousApproved,
                            WorkflowCondition::WithinSLA,
                        ],
                        authorization: AuthorizationRequirement::Required(level),
                    },
                )?;
            }

            previous_node = Some(approval_node);
        }

        Ok(auth_stage)
    }
}
```

### 4. Authorization and Access Control

```rust
/// Authorization service using graph algorithms
pub struct DocumentAuthorizationService {
    graph_model: Arc<RwLock<DocumentGraphModel>>,
}

impl DocumentAuthorizationService {
    /// Check if user can access document
    pub fn check_access(
        &self,
        user: &User,
        document_id: DocumentId,
    ) -> Result<AccessDecision, AuthError> {
        let graph = self.graph_model.read().unwrap();

        // Find document location in graph
        let doc_node = graph.document_locations.get(&document_id)
            .ok_or(AuthError::DocumentNotFound)?;

        // Find user's accessible nodes based on roles
        let accessible_nodes = self.find_accessible_nodes(&graph, user);

        if accessible_nodes.contains(doc_node) {
            // Check specific permissions on the node
            let (stage_id, node_idx) = graph.node_index.get(&document_id).unwrap();
            let stage_graph = &graph.stages[stage_id];
            let node = &stage_graph[*node_idx];

            let permission = node.permissions.get_for_role(&user.role);

            Ok(AccessDecision::Allowed {
                permission_level: permission,
                restrictions: self.get_restrictions(&node, user),
            })
        } else {
            Ok(AccessDecision::Denied {
                reason: "User does not have access to this workflow stage".to_string(),
            })
        }
    }

    /// Find shortest authorization path for document
    pub fn find_authorization_path(
        &self,
        document: &Document,
        current_user: &User,
    ) -> Result<AuthorizationPath, AuthError> {
        let graph = self.graph_model.read().unwrap();

        // Determine required approval level based on document
        let required_level = self.determine_required_level(document);

        // Find current position
        let current_pos = self.find_user_position(&graph, current_user)?;

        // Find target approval node
        let target_node = self.find_approval_node(&graph, required_level)?;

        // Use Dijkstra to find shortest path
        let (distances, predecessors) = dijkstra(
            &graph.workflow,
            current_pos,
            Some(target_node),
            |edge| {
                // Weight based on estimated approval time
                match &graph.workflow[edge.target()].node_type {
                    DocumentNodeType::ApprovalGate { authority_level, .. } => {
                        self.estimate_approval_time(*authority_level)
                    }
                    _ => 1, // Default weight
                }
            },
        );

        // Reconstruct path
        let path = self.reconstruct_path(predecessors, current_pos, target_node)?;

        Ok(AuthorizationPath {
            steps: path,
            estimated_time: distances[&target_node],
            required_approvals: self.extract_approvals(&graph, &path),
        })
    }

    /// Check if user can approve at current stage
    pub fn can_approve(
        &self,
        user: &User,
        document_id: DocumentId,
    ) -> Result<bool, AuthError> {
        let graph = self.graph_model.read().unwrap();

        // Find document's current node
        let doc_node = graph.document_locations.get(&document_id)
            .ok_or(AuthError::DocumentNotFound)?;

        let (stage_id, node_idx) = graph.node_index.get(&document_id).unwrap();
        let node = &graph.stages[stage_id][*node_idx];

        match &node.node_type {
            DocumentNodeType::ApprovalGate { authority_level, delegation_allowed, .. } => {
                // Check if user has required authority
                if user.authority_level >= *authority_level {
                    return Ok(true);
                }

                // Check delegation
                if *delegation_allowed {
                    return Ok(self.has_delegation(user, *authority_level)?);
                }

                Ok(false)
            }
            _ => Ok(false), // Not an approval node
        }
    }
}
```

### 5. Event Sourcing Integration

```rust
/// Events for document lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentLifecycleEvent {
    // Document events
    DocumentCreated {
        document_id: DocumentId,
        template: TemplateId,
        author: UserId,
        metadata: DocumentMetadata,
    },
    DocumentUpdated {
        document_id: DocumentId,
        changes: Vec<FieldChange>,
        updated_by: UserId,
    },

    // Review events
    ReviewRequested {
        document_id: DocumentId,
        review_type: ReviewType,
        reviewer: UserId,
        due_date: DateTime<Utc>,
    },
    ReviewCompleted {
        document_id: DocumentId,
        review_type: ReviewType,
        reviewer: UserId,
        outcome: ReviewOutcome,
        comments: String,
    },

    // Authorization events
    ApprovalRequested {
        document_id: DocumentId,
        approver: UserId,
        authority_level: AuthorityLevel,
    },
    ApprovalGranted {
        document_id: DocumentId,
        approver: UserId,
        authority_level: AuthorityLevel,
        conditions: Vec<ApprovalCondition>,
    },
    ApprovalDenied {
        document_id: DocumentId,
        approver: UserId,
        reason: String,
    },

    // Distribution events
    DocumentPublished {
        document_id: DocumentId,
        channels: Vec<DistributionChannel>,
        access_list: Vec<UserId>,
    },
    DocumentArchived {
        document_id: DocumentId,
        retention_period: Duration,
        archive_location: ArchiveLocation,
    },
}

/// Event handler for replaying document history
pub struct DocumentEventHandler;

impl DocumentEventHandler {
    pub fn replay_to_graph(
        &self,
        events: Vec<DocumentLifecycleEvent>,
        graph: &mut DocumentGraphModel,
    ) -> Result<(), ReplayError> {
        for event in events {
            match event {
                DocumentLifecycleEvent::DocumentCreated { document_id, .. } => {
                    // Add document to creation stage
                    let creation_stage = self.find_creation_stage(graph)?;
                    let node_idx = self.find_draft_node(graph, creation_stage)?;
                    graph.document_locations.insert(document_id, node_idx);
                }

                DocumentLifecycleEvent::ReviewCompleted { document_id, review_type, outcome, .. } => {
                    if outcome == ReviewOutcome::Approved {
                        // Move document to next stage
                        self.advance_document(graph, document_id)?;
                    }
                }

                DocumentLifecycleEvent::ApprovalGranted { document_id, authority_level, .. } => {
                    // Check if all required approvals obtained
                    if self.all_approvals_complete(graph, document_id, authority_level)? {
                        self.advance_to_distribution(graph, document_id)?;
                    }
                }

                // ... handle other events
            }
        }

        Ok(())
    }
}
```

### 6. Practical Usage Example

```rust
/// Example: Processing a high-value contract
pub async fn process_contract_document(
    domain: &mut DocumentManagementAggregate,
    auth_service: &DocumentAuthorizationService,
    event_store: &mut EventStore,
) -> Result<DocumentId, ProcessError> {
    // 1. Create document
    let document_id = DocumentId::new();
    let author = UserId::from("john.doe");

    let event = DocumentLifecycleEvent::DocumentCreated {
        document_id,
        template: TemplateId::from("contract_v2"),
        author: author.clone(),
        metadata: DocumentMetadata {
            title: "Enterprise Software License Agreement".to_string(),
            department: "Legal".to_string(),
            contract_value: 1_500_000,
            tags: vec!["high-value", "enterprise", "software"],
        },
    };

    event_store.append_domain_event(domain.root.identity, event)?;

    // 2. Submit for review
    let reviews = vec![
        (ReviewType::Legal, UserId::from("legal.team")),
        (ReviewType::Financial, UserId::from("cfo.office")),
        (ReviewType::Technical, UserId::from("cto.office")),
    ];

    for (review_type, reviewer) in reviews {
        let event = DocumentLifecycleEvent::ReviewRequested {
            document_id,
            review_type,
            reviewer,
            due_date: Utc::now() + Duration::hours(48),
        };
        event_store.append_domain_event(domain.root.identity, event)?;
    }

    // 3. Find authorization path
    let auth_path = auth_service.find_authorization_path(
        &document,
        &current_user,
    )?;

    println!("Authorization required from: {:?}", auth_path.required_approvals);
    println!("Estimated time: {} hours", auth_path.estimated_time);

    // 4. Check access permissions
    let access = auth_service.check_access(&current_user, document_id)?;

    match access {
        AccessDecision::Allowed { permission_level, .. } => {
            println!("User has {} access", permission_level);
        }
        AccessDecision::Denied { reason } => {
            return Err(ProcessError::AccessDenied(reason));
        }
    }

    Ok(document_id)
}
```

## Benefits of This Architecture

1. **Clear Workflow Visualization**: Graph structure maps directly to business workflows
2. **Efficient Authorization**: Petgraph algorithms compute optimal approval paths
3. **Complete Audit Trail**: Every action recorded in event store
4. **Flexible Workflows**: Easy to modify without changing core structure
5. **Role-Based Access**: Graph traversal determines document access
6. **Parallel Processing**: Support for concurrent reviews and approvals
7. **Time-Based Analysis**: Can replay to any point in document history

## Key Patterns Demonstrated

1. **DDD Aggregate**: Document lifecycle as consistency boundary
2. **Entity Pattern**: Workflow stages with identity and lifecycle
3. **Value Objects**: Document states and metadata
4. **Domain Events**: Complete document history
5. **Repository Pattern**: Event-sourced persistence
6. **Domain Services**: Authorization and workflow services
7. **Graph Algorithms**: Shortest path for approvals, BFS for access control

This architecture provides a robust foundation for complex document management workflows while maintaining clean domain boundaries and leveraging graph algorithms for efficiency.
