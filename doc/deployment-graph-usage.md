# Deployment Graph Usage Guide

## Overview

The CIM Deployment Graph functionality allows you to visually design infrastructure deployments that can be automatically translated to NixOS configurations. This enables:

1. **Visual Infrastructure Design**: Create deployment graphs with nodes representing services, databases, agents, etc.
2. **Multi-Agent Collaboration**: Agents can discuss and validate deployment configurations
3. **Automatic Nix Generation**: Convert visual graphs to deployable `flake.nix` configurations

## Running the Demos

### 1. Iced UI Demo (Simplest)

```bash
cargo run --example deployment_graph_ui
```

This shows a simple UI where you can:
- Add deployment nodes (services, databases, agents, load balancers)
- View node details and connections
- See how the graph structure is built

### 2. Bevy 3D Visualization Demo

```bash
cargo run --example deployment_graph_demo --package cim-domain-bevy
```

This shows a 3D visualization with:
- Different shapes for different node types
- Color-coded deployment status
- Interactive node selection
- Dependency visualization with arrows

## Using in Code

### Creating a Deployment Graph

```rust
use cim_domain_graph::{Graph, GraphId, NodeId};
use cim_domain_graph::deployment::{
    DeploymentNodeType, ResourceRequirements,
    graph_adapter::create_deployment_node_metadata,
};

// Create a new graph
let mut graph = Graph::new(
    GraphId::new(),
    "My Deployment".to_string(),
    "Production deployment configuration".to_string(),
);

// Add a service node
let service_id = NodeId::new();
let service = DeploymentNodeType::Service {
    name: "api-server".to_string(),
    command: "cargo run --release".to_string(),
    args: vec!["--port".to_string(), "8080".to_string()],
    environment: HashMap::new(),
    port: Some(8080),
    health_check: None,
    resources: ResourceRequirements {
        cpu_cores: Some(2.0),
        memory_mb: Some(1024),
        disk_gb: Some(10),
    },
};

graph.add_node(
    service_id,
    "Service".to_string(),
    create_deployment_node_metadata(service),
).unwrap();
```

### Translating to Nix

```rust
use cim_domain_graph::deployment::{
    GraphToNixTranslator, StandardTranslator,
};

let translator = StandardTranslator::new();
let nix_spec = translator.translate_graph(&graph)?;

// nix_spec now contains structured data that can be 
// converted to a flake.nix file
```

## Node Types

### Service Node
Represents a service that runs as a systemd unit:
- Command to execute
- Environment variables
- Port configuration
- Resource requirements

### Database Node
Represents a database service:
- Engine (PostgreSQL, MySQL, MongoDB, etc.)
- Version
- Persistence settings
- Backup schedule

### Agent Node
Represents an AI agent:
- Capabilities (what it can do)
- Subscriptions (what events it listens to)
- Resource limits

### MessageBus Node
Represents messaging infrastructure:
- Type (NATS, Kafka, RabbitMQ)
- Cluster configuration
- Persistence settings

### LoadBalancer Node
Represents load balancing:
- Strategy (round-robin, least connections, etc.)
- Backend services
- Health check configuration

### Storage Node
Represents storage volumes:
- Type (local, network, object store)
- Size
- Mount paths

## Edge Types

### DependsOn
Service startup dependency - target must be running before source

### ConnectsTo
Network connection between services

### DataFlow
Represents data flow direction and format

### LoadBalances
Load balancer to backend service relationship

### MountsVolume
Service mounting a storage volume

### PublishesTo / SubscribesTo
Message bus pub/sub relationships

### Manages
Agent managing another service

## Multi-Agent Collaboration

The deployment graph integrates with the agent coordination service:

```rust
use cim_agent_coordination::{
    TaskCoordinator, CoordinationTask,
    ConsensusProtocol, SimpleMajorityProtocol,
};

// Agents can validate deployment
let task = CoordinationTask::new(
    "validate_deployment".to_string(),
    "deployment_validation".to_string(),
    serde_json::to_value(&graph)?,
);

coordinator.submit_task(task).await?;

// Agents can vote on deployment approval
let proposal = Proposal {
    title: "Deploy to production".to_string(),
    description: format!("Deploy graph with {} nodes", graph.nodes().len()),
    proposal_type: ProposalType::Deployment,
    data: serde_json::to_value(&nix_spec)?,
    // ...
};

let consensus = SimpleMajorityProtocol::new(proposal, agent_ids, config);
consensus.start().await?;
```

## Architecture

```
Visual Graph (Bevy/Iced UI)
    ↓
Graph Domain (Rust structures)
    ↓
Deployment Types (DeploymentNodeType, etc.)
    ↓
GraphToNixTranslator
    ↓
NixDeploymentSpec
    ↓
flake.nix generation
    ↓
NixOS deployment
```

## Next Steps

1. The generated NixDeploymentSpec needs to be converted to actual Nix code
2. Agents need integration with real deployment systems
3. Monitoring and rollback capabilities need to be added
4. The UI needs deployment status visualization

This provides the foundation for visual infrastructure-as-code where deployments are:
- Designed visually
- Validated by AI agents
- Automatically translated to reproducible NixOS configurations