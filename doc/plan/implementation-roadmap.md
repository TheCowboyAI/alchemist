# CIM Leaf Implementation Roadmap

## File Structure and Implementation Order

### Week 1: Foundation

#### 1. Graph Deployment Types
```
cim-domain-graph/src/deployment/
├── mod.rs                      # Module exports
├── node_types.rs               # DeploymentNodeType enum
├── edge_types.rs               # DeploymentEdgeType enum  
├── translator.rs               # GraphToNixTranslator trait
└── validation.rs               # Graph validation rules
```

**Implementation Order**:
1. `node_types.rs` - Define all deployment node types
2. `edge_types.rs` - Define relationship types
3. `validation.rs` - Validation rules and cycle detection
4. `translator.rs` - Core translation trait
5. `mod.rs` - Wire everything together

#### 2. Agent Coordination Crate
```
cim-agent-coordination/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Crate root
│   ├── registry.rs             # AgentRegistry
│   ├── coordinator.rs          # TaskCoordinator
│   ├── discovery.rs            # Agent discovery via NATS
│   ├── presence.rs             # Heartbeat and presence
│   └── task.rs                 # Task definitions
└── tests/
    └── coordination_tests.rs
```

**Implementation Order**:
1. `Cargo.toml` - Dependencies (tokio, async-nats, etc.)
2. `task.rs` - Core task structures
3. `registry.rs` - Agent tracking
4. `discovery.rs` - NATS integration
5. `coordinator.rs` - Task assignment logic

### Week 2: Core Translation

#### 3. Node Translators
```
cim-domain-graph/src/deployment/translators/
├── mod.rs
├── service.rs                  # Service → systemd.services
├── agent.rs                    # Agent → systemd.services
├── database.rs                 # Database → postgres service
├── nats.rs                     # NATS → cluster config
└── storage.rs                  # Storage → filesystem mounts
```

**Files to Create** (in order):
1. `service.rs` - Most common use case
2. `database.rs` - Stateful services
3. `nats.rs` - Message bus configuration
4. `agent.rs` - Background workers
5. `storage.rs` - Persistent volumes

#### 4. Flake Generator
```
cim-domain-graph/src/deployment/generator/
├── mod.rs
├── flake_builder.rs            # Main flake.nix builder
├── nixos_config.rs             # NixOS system configuration
├── service_module.rs           # Service definition generator
└── network_module.rs           # Network topology generator
```

### Week 3: Agent Intelligence

#### 5. Dialog Routing
```
cim-domain-dialog/src/routing/
├── mod.rs
├── agent_router.rs             # Message routing logic
├── channel.rs                  # Agent communication channels
├── context_sharing.rs          # Context propagation
└── strategies.rs               # Routing strategies
```

#### 6. Shared Knowledge Base
```
cim-domain-agent/src/knowledge/
├── mod.rs
├── store.rs                    # Knowledge storage
├── fact.rs                     # Fact representation
├── sync.rs                     # CRDT synchronization
├── query.rs                    # Query interface
└── conflict.rs                 # Conflict resolution
```

### Week 4: Consensus & Integration

#### 7. Consensus Protocols
```
cim-agent-coordination/src/consensus/
├── mod.rs
├── protocol.rs                 # ConsensusProtocol trait
├── voting.rs                   # Voting mechanisms
├── proposals.rs                # Proposal management
├── simple_majority.rs          # Simple majority implementation
├── qualified_majority.rs       # Qualified majority
└── weighted_voting.rs          # Weighted voting
```

#### 8. Integration Layer
```
alchemist/src/deployment/
├── mod.rs
├── pipeline.rs                 # End-to-end pipeline
├── agent_bridge.rs             # Agent-infrastructure bridge
├── ui_integration.rs           # UI deployment features
└── monitoring.rs               # Deployment monitoring
```

## Specific Implementation Tasks

### Task 1: Create DeploymentNodeType (Day 1-2)
```rust
// cim-domain-graph/src/deployment/node_types.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentNodeType {
    Service {
        name: String,
        command: String,
        args: Vec<String>,
        environment: HashMap<String, String>,
        port: Option<u16>,
        health_check: Option<HealthCheck>,
        resources: ResourceRequirements,
    },
    Agent {
        name: String,
        capabilities: Vec<String>,
        subscriptions: Vec<String>,
        rate_limit: Option<RateLimit>,
        resources: ResourceRequirements,
    },
    Database {
        name: String,
        engine: DatabaseEngine,
        version: String,
        persistent: bool,
        backup_schedule: Option<String>,
        resources: ResourceRequirements,
    },
    MessageBus {
        name: String,
        bus_type: MessageBusType,
        cluster_size: usize,
        persistence: bool,
        topics: Vec<TopicConfig>,
    },
    LoadBalancer {
        name: String,
        strategy: LoadBalancingStrategy,
        health_check_interval: Duration,
        backends: Vec<String>, // References to service nodes
    },
    Storage {
        name: String,
        storage_type: StorageType,
        size: String, // "10Gi", "1Ti", etc.
        mount_path: String,
        access_mode: AccessMode,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseEngine {
    PostgreSQL,
    MySQL,
    MongoDB,
    Redis,
    SQLite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageBusType {
    NATS,
    Kafka,
    RabbitMQ,
    Redis,
}

// Additional enums and structs...
```

### Task 2: Implement GraphToNixTranslator (Day 3-4)
```rust
// cim-domain-graph/src/deployment/translator.rs

use super::{DeploymentNodeType, DeploymentEdgeType};
use crate::Graph;
use anyhow::Result;

pub trait GraphToNixTranslator {
    /// Translate a deployment graph to a Nix deployment specification
    fn translate_graph(&self, graph: &Graph) -> Result<NixDeploymentSpec>;
    
    /// Validate that a graph is suitable for deployment
    fn validate_deployment_graph(&self, graph: &Graph) -> Result<()>;
    
    /// Extract service definitions from the graph
    fn extract_services(&self, graph: &Graph) -> Result<Vec<ServiceSpec>>;
    
    /// Extract dependencies from edges
    fn extract_dependencies(&self, graph: &Graph) -> Result<DependencyMap>;
}

pub struct StandardTranslator {
    node_translators: HashMap<String, Box<dyn NodeTranslator>>,
    edge_processors: Vec<Box<dyn EdgeProcessor>>,
}

impl GraphToNixTranslator for StandardTranslator {
    fn translate_graph(&self, graph: &Graph) -> Result<NixDeploymentSpec> {
        // 1. Validate graph
        self.validate_deployment_graph(graph)?;
        
        // 2. Topological sort for dependency order
        let sorted_nodes = graph.topological_sort()?;
        
        // 3. Translate each node
        let mut services = Vec::new();
        let mut databases = Vec::new();
        let mut agents = Vec::new();
        
        for node_id in sorted_nodes {
            let node = graph.get_node(&node_id)?;
            match node.node_type {
                DeploymentNodeType::Service { .. } => {
                    services.push(self.translate_service_node(node)?);
                }
                DeploymentNodeType::Database { .. } => {
                    databases.push(self.translate_database_node(node)?);
                }
                // ... other node types
            }
        }
        
        // 4. Process edges for dependencies
        let dependencies = self.extract_dependencies(graph)?;
        
        // 5. Build deployment spec
        Ok(NixDeploymentSpec {
            services,
            databases,
            agents,
            dependencies,
            network_topology: self.build_network_topology(graph)?,
        })
    }
}
```

### Task 3: Create Agent Coordination Service (Day 5-6)
```rust
// cim-agent-coordination/src/coordinator.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use dashmap::DashMap;

pub struct TaskCoordinator {
    /// Pending tasks waiting for assignment
    pending_tasks: Arc<RwLock<Vec<CoordinationTask>>>,
    /// Active task assignments
    assignments: Arc<DashMap<TaskId, Assignment>>,
    /// Agent registry
    registry: Arc<AgentRegistry>,
    /// Task routing strategies
    strategies: Vec<Box<dyn RoutingStrategy>>,
}

impl TaskCoordinator {
    pub async fn submit_task(&self, task: CoordinationTask) -> Result<TaskId> {
        // 1. Validate task
        self.validate_task(&task)?;
        
        // 2. Find capable agents
        let capable_agents = self.registry
            .find_agents_with_capability(&task.required_capability)
            .await?;
        
        if capable_agents.is_empty() {
            return Err(CoordinationError::NoCapableAgents);
        }
        
        // 3. Apply routing strategy
        let selected_agent = self.select_agent(&capable_agents, &task).await?;
        
        // 4. Create assignment
        let assignment = Assignment {
            task_id: task.id.clone(),
            agent_id: selected_agent.id.clone(),
            assigned_at: Utc::now(),
            status: AssignmentStatus::Pending,
        };
        
        // 5. Notify agent via NATS
        self.notify_agent(&selected_agent, &task).await?;
        
        // 6. Track assignment
        self.assignments.insert(task.id.clone(), assignment);
        
        Ok(task.id)
    }
    
    pub async fn get_task_status(&self, task_id: &TaskId) -> Result<TaskStatus> {
        self.assignments
            .get(task_id)
            .map(|a| a.status.clone())
            .ok_or(CoordinationError::TaskNotFound)
    }
}
```

### Task 4: Implement Consensus Protocol (Day 7-8)
```rust
// cim-agent-coordination/src/consensus/simple_majority.rs

use super::{ConsensusProtocol, Proposal, Vote, ConsensusResult};

pub struct SimpleMajorityProtocol {
    proposal: Proposal,
    votes: Arc<RwLock<HashMap<AgentId, Vote>>>,
    participants: Vec<AgentId>,
    timeout: Duration,
}

impl ConsensusProtocol for SimpleMajorityProtocol {
    async fn start(&mut self) -> Result<()> {
        // Notify all participants
        for agent_id in &self.participants {
            self.notify_agent(agent_id, &self.proposal).await?;
        }
        
        // Start timeout timer
        tokio::spawn(self.timeout_monitor());
        
        Ok(())
    }
    
    async fn cast_vote(&mut self, agent_id: AgentId, vote: Vote) -> Result<()> {
        if !self.participants.contains(&agent_id) {
            return Err(ConsensusError::UnauthorizedVoter);
        }
        
        let mut votes = self.votes.write().await;
        votes.insert(agent_id, vote);
        
        // Check if we have enough votes
        if votes.len() >= self.required_votes() {
            self.try_conclude().await?;
        }
        
        Ok(())
    }
    
    async fn get_result(&self) -> Result<ConsensusResult> {
        let votes = self.votes.read().await;
        
        let approve_count = votes.values()
            .filter(|v| matches!(v, Vote::Approve))
            .count();
            
        let total_votes = votes.len();
        let required = self.participants.len() / 2 + 1;
        
        if approve_count >= required {
            Ok(ConsensusResult::Approved)
        } else if total_votes - approve_count > self.participants.len() - required {
            Ok(ConsensusResult::Rejected)
        } else {
            Ok(ConsensusResult::Pending)
        }
    }
}
```

## Testing Strategy for Each Component

### Unit Test Example for Each Module
```rust
// Graph Translation Tests
#[test]
fn test_service_node_to_nix() {
    let node = create_test_service_node();
    let translator = ServiceNodeTranslator::new();
    let nix = translator.translate(&node).unwrap();
    assert!(nix.contains("systemd.services"));
}

// Agent Coordination Tests
#[tokio::test]
async fn test_task_assignment_with_capability() {
    let coordinator = create_test_coordinator().await;
    let task = create_task_requiring("deploy");
    let assignment = coordinator.submit_task(task).await.unwrap();
    assert!(assignment.is_assigned());
}

// Consensus Tests
#[tokio::test]
async fn test_simple_majority_consensus() {
    let mut protocol = SimpleMajorityProtocol::new(5);
    protocol.cast_vote("agent1", Vote::Approve).await.unwrap();
    protocol.cast_vote("agent2", Vote::Approve).await.unwrap();
    protocol.cast_vote("agent3", Vote::Approve).await.unwrap();
    assert_eq!(protocol.get_result().await.unwrap(), ConsensusResult::Approved);
}
```

## Milestones and Deliverables

### Week 1 Deliverable
- [ ] DeploymentNodeType and DeploymentEdgeType fully defined
- [ ] GraphToNixTranslator trait implemented
- [ ] Basic agent coordination service running
- [ ] 20+ unit tests passing

### Week 2 Deliverable
- [ ] All node translators implemented
- [ ] Complete flake.nix generation working
- [ ] Agent discovery via NATS functional
- [ ] 50+ unit tests passing

### Week 3 Deliverable
- [ ] Dialog routing between agents working
- [ ] Shared knowledge base with CRDT sync
- [ ] Basic consensus protocols implemented
- [ ] Integration tests passing

### Week 4 Deliverable
- [ ] Complete end-to-end pipeline tested
- [ ] UI integration for deployment metadata
- [ ] Performance benchmarks meeting targets
- [ ] Full test suite (100+ tests) passing

## Success Criteria

1. **Functional**: Can deploy a 5-service application with database
2. **Performance**: < 1s translation for 100-node graphs
3. **Reliability**: Handles agent failures gracefully
4. **Usability**: Non-technical users can deploy via UI