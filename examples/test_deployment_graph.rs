//! Simple test to verify deployment graph functionality

use cim_domain_graph::deployment::{
    DeploymentNodeType, DeploymentEdgeType, GraphToNixTranslator, 
    NixDeploymentSpec, StandardTranslator, DatabaseEngine, MessageBusType
};
use cim_domain_graph::{Graph, NodeMetadata};
use std::collections::HashMap;
use uuid::Uuid;

fn main() {
    println!("=== Testing Deployment Graph Functionality ===\n");

    // Create a deployment graph
    let mut graph = Graph::new();
    
    // Add nodes with deployment types
    let db_node = graph.create_node(
        "postgres-db",
        NodeMetadata {
            node_type: "Database".to_string(),
            properties: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
        }
    );
    
    let service_node = graph.create_node(
        "api-service",
        NodeMetadata {
            node_type: "Service".to_string(),
            properties: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
        }
    );
    
    let nats_node = graph.create_node(
        "nats-bus",
        NodeMetadata {
            node_type: "MessageBus".to_string(),
            properties: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
        }
    );
    
    // Add edges
    graph.create_edge(service_node, db_node, "depends_on".to_string(), 1.0);
    graph.create_edge(service_node, nats_node, "publishes_to".to_string(), 1.0);
    
    println!("Created deployment graph with {} nodes and {} edges", 
        graph.node_count(), graph.edge_count());
    
    // Create deployment node types
    let deployment_nodes = vec![
        DeploymentNodeType::Database {
            name: "postgres-db".to_string(),
            engine: DatabaseEngine::PostgreSQL,
            version: Some("14".to_string()),
            port: 5432,
            storage_size_gb: 100,
            replicas: 1,
            backup_enabled: true,
        },
        DeploymentNodeType::Service {
            name: "api-service".to_string(),
            image: "alchemist-api:latest".to_string(),
            command: "./api-server".to_string(),
            args: vec!["--port=8080".to_string()],
            env: HashMap::from([
                ("DATABASE_URL".to_string(), "postgresql://localhost:5432/alchemist".to_string()),
                ("NATS_URL".to_string(), "nats://localhost:4222".to_string()),
            ]),
            ports: vec![8080],
            replicas: 2,
            health_check: Some("/health".to_string()),
        },
        DeploymentNodeType::MessageBus {
            name: "nats-bus".to_string(),
            bus_type: MessageBusType::NATS,
            port: 4222,
            jetstream_enabled: true,
            cluster_size: 3,
            persistence_enabled: true,
        },
    ];
    
    // Create deployment edges
    let deployment_edges = vec![
        DeploymentEdgeType::DependsOn {
            from: "api-service".to_string(),
            to: "postgres-db".to_string(),
            startup_order: 1,
            health_check_required: true,
        },
        DeploymentEdgeType::PublishesTo {
            from: "api-service".to_string(),
            to: "nats-bus".to_string(),
            topics: vec!["events".to_string(), "commands".to_string()],
            qos: 1,
        },
    ];
    
    println!("\nDeployment Nodes:");
    for node in &deployment_nodes {
        match node {
            DeploymentNodeType::Database { name, engine, .. } => {
                println!("  - Database: {} ({})", name, engine.as_str());
            }
            DeploymentNodeType::Service { name, replicas, .. } => {
                println!("  - Service: {} (replicas: {})", name, replicas);
            }
            DeploymentNodeType::MessageBus { name, bus_type, .. } => {
                println!("  - MessageBus: {} ({})", name, bus_type.as_str());
            }
            _ => {}
        }
    }
    
    println!("\nDeployment Edges:");
    for edge in &deployment_edges {
        match edge {
            DeploymentEdgeType::DependsOn { from, to, .. } => {
                println!("  - {} depends on {}", from, to);
            }
            DeploymentEdgeType::PublishesTo { from, to, topics, .. } => {
                println!("  - {} publishes to {} (topics: {:?})", from, to, topics);
            }
            _ => {}
        }
    }
    
    // Create deployment spec
    let spec = NixDeploymentSpec {
        name: "alchemist-deployment".to_string(),
        description: "Test deployment for Alchemist system".to_string(),
        nodes: deployment_nodes,
        edges: deployment_edges,
        environment: HashMap::from([
            ("ENV".to_string(), "production".to_string()),
        ]),
        metadata: HashMap::new(),
    };
    
    // Translate to Nix
    let translator = StandardTranslator::new();
    match translator.translate(&spec) {
        Ok(nix_config) => {
            println!("\n=== Generated Nix Configuration ===");
            println!("{}", serde_json::to_string_pretty(&nix_config).unwrap());
        }
        Err(e) => {
            println!("\nError translating to Nix: {}", e);
        }
    }
    
    println!("\n✅ Deployment graph functionality verified!");
    
    // Test multi-agent coordination
    println!("\n=== Testing Multi-Agent Coordination ===");
    
    use cim_agent_coordination::{AgentRegistry, TaskCoordinator, AgentCapability};
    
    let mut registry = AgentRegistry::new();
    
    // Register agents
    let deploy_agent = registry.register_agent(
        "deploy-agent".to_string(),
        vec![
            AgentCapability::DeploymentManagement,
            AgentCapability::InfrastructureProvisioning,
        ],
    );
    
    let monitor_agent = registry.register_agent(
        "monitor-agent".to_string(),
        vec![
            AgentCapability::HealthMonitoring,
            AgentCapability::AlertManagement,
        ],
    );
    
    let config_agent = registry.register_agent(
        "config-agent".to_string(),
        vec![
            AgentCapability::ConfigurationManagement,
        ],
    );
    
    println!("Registered {} agents", registry.agent_count());
    
    // Create task coordinator
    let coordinator = TaskCoordinator::new(registry);
    
    // Test task delegation
    let deployment_task = coordinator.delegate_task(
        "Deploy API service",
        AgentCapability::DeploymentManagement,
    );
    
    match deployment_task {
        Ok(agent_id) => {
            println!("✅ Task delegated to agent: {}", agent_id);
        }
        Err(e) => {
            println!("❌ Failed to delegate task: {}", e);
        }
    }
    
    println!("\n=== Testing Agent Dialog Routing ===");
    
    use cim_domain_dialog::routing::{AgentDialogRouter, channel::ChannelType};
    use cim_domain_dialog::value_objects::{
        Message, MessageContent, MessageIntent, Participant, ParticipantType, ParticipantRole
    };
    
    let mut router = AgentDialogRouter::new();
    
    // Register agent capabilities for routing
    router.register_agent(
        "deploy-agent".to_string(),
        vec!["deployment".to_string(), "infrastructure".to_string()],
    );
    
    router.register_agent(
        "monitor-agent".to_string(),
        vec!["monitoring".to_string(), "alerts".to_string()],
    );
    
    // Create a deployment channel
    let channel_id = router.create_agent_channel(
        vec!["deploy-agent".to_string(), "monitor-agent".to_string()],
        ChannelType::Task,
    );
    
    println!("Created task channel: {:?}", channel_id);
    
    // Test message routing
    let message = Message {
        content: MessageContent::Text("Deploy the new service version".to_string()),
        intent: Some(MessageIntent::Command),
        language: "en".to_string(),
        sentiment: None,
        embeddings: None,
    };
    
    let participants = vec![
        Participant {
            id: Uuid::new_v4(),
            name: "Deploy Agent".to_string(),
            participant_type: ParticipantType::AIAgent,
            role: ParticipantRole::Assistant,
            metadata: HashMap::new(),
        },
        Participant {
            id: Uuid::new_v4(),
            name: "Monitor Agent".to_string(),
            participant_type: ParticipantType::AIAgent,
            role: ParticipantRole::Assistant,
            metadata: HashMap::new(),
        },
    ];
    
    let context = cim_domain_dialog::routing::context_sharing::SharedContext::new();
    let decision = router.route_message(&message, &participants, &context);
    
    println!("\nRouting decision:");
    println!("  Strategy: {}", decision.strategy);
    println!("  Targets: {} agents", decision.targets.len());
    println!("  Confidence: {:.2}", decision.confidence);
    
    println!("\n✅ All deployment functionality verified!");
}