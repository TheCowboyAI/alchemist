# CIM Leaf Deployment Test Plan

## Overview

This document outlines comprehensive testing strategies to ensure the CIM Leaf deployment functionality works correctly with multi-agent collaboration.

## Test Categories

### 1. Unit Tests

#### Graph to Nix Translation
```rust
// cim-domain-graph/src/deployment/tests/translation_tests.rs

#[test]
fn test_service_node_translation() {
    let node = DeploymentNode::Service {
        name: "web-api".into(),
        port: 8080,
        command: "cargo run --bin api".into(),
        environment: hashmap!{
            "DATABASE_URL" => "postgresql://localhost/myapp"
        }
    };
    
    let nix_module = node.to_nix_module();
    assert!(nix_module.contains("systemd.services.web-api"));
    assert!(nix_module.contains("ExecStart = \"cargo run --bin api\""));
    assert!(nix_module.contains("Environment = \"DATABASE_URL=postgresql://localhost/myapp\""));
}

#[test]
fn test_dependency_edge_translation() {
    let edge = DeploymentEdge::DependsOn {
        from: "web-api",
        to: "postgres",
        startup_delay: Some(Duration::from_secs(5)),
    };
    
    let nix_deps = edge.to_nix_dependencies();
    assert!(nix_deps.contains("after = [ \"postgres.service\" ]"));
    assert!(nix_deps.contains("wants = [ \"postgres.service\" ]"));
}

#[test]
fn test_cycle_detection() {
    let graph = Graph::new()
        .add_node("a", DeploymentNode::Service { ... })
        .add_node("b", DeploymentNode::Service { ... })
        .add_edge("a", "b", DeploymentEdge::DependsOn)
        .add_edge("b", "a", DeploymentEdge::DependsOn);
    
    let result = validate_deployment_graph(&graph);
    assert!(matches!(result, Err(DeploymentError::CyclicDependency(_))));
}
```

#### Agent Coordination
```rust
// cim-agent-coordination/src/tests/coordination_tests.rs

#[tokio::test]
async fn test_agent_registration() {
    let coordinator = AgentCoordinator::new();
    let agent = TestAgent::new("agent-1", vec!["deploy", "monitor"]);
    
    coordinator.register_agent(agent.clone()).await?;
    
    let registered = coordinator.get_agent("agent-1").await?;
    assert_eq!(registered.capabilities, vec!["deploy", "monitor"]);
}

#[tokio::test]
async fn test_task_assignment() {
    let coordinator = AgentCoordinator::new();
    coordinator.register_agent(agent_with_capability("deploy")).await?;
    
    let task = CoordinationTask {
        id: "deploy-web-api",
        required_capability: "deploy",
        payload: json!({"service": "web-api"}),
    };
    
    let assignment = coordinator.assign_task(task).await?;
    assert!(assignment.agent_id.is_some());
}

#[tokio::test]
async fn test_consensus_voting() {
    let consensus = ConsensusProtocol::SimpleMajority;
    let agents = vec!["agent-1", "agent-2", "agent-3"];
    
    let proposal = Proposal {
        id: "deploy-v2",
        description: "Deploy version 2.0",
        proposed_by: "agent-1",
    };
    
    consensus.cast_vote("agent-1", Vote::Approve).await?;
    consensus.cast_vote("agent-2", Vote::Approve).await?;
    consensus.cast_vote("agent-3", Vote::Reject).await?;
    
    let result = consensus.tally_votes().await?;
    assert_eq!(result, ConsensusResult::Approved);
}
```

### 2. Integration Tests

#### End-to-End Deployment
```rust
// tests/integration/deployment_tests.rs

#[tokio::test]
async fn test_simple_deployment_pipeline() {
    // Setup
    let test_env = TestEnvironment::new().await;
    let graph = create_simple_web_service_graph();
    
    // Step 1: Validate graph
    let validator = GraphValidator::new();
    validator.validate_deployment_graph(&graph)?;
    
    // Step 2: Translate to Nix
    let translator = GraphToNixTranslator::new();
    let deployment_spec = translator.translate_graph(&graph)?;
    
    // Step 3: Generate flake.nix
    let flake_generator = FlakeGenerator::new();
    let flake_content = flake_generator.generate(&deployment_spec)?;
    
    // Verify flake.nix is valid
    assert!(flake_content.contains("nixosConfigurations"));
    assert!(flake_content.contains("systemd.services.web-api"));
    
    // Step 4: Simulate deployment (in test environment)
    let deployment_result = test_env.deploy_flake(&flake_content).await?;
    assert_eq!(deployment_result.status, DeploymentStatus::Success);
}

#[tokio::test]
async fn test_multi_agent_deployment_approval() {
    let test_env = TestEnvironment::new().await;
    
    // Spawn test agents
    let agents = test_env.spawn_agents(vec![
        ("deployer", vec!["deploy", "vote"]),
        ("monitor", vec!["monitor", "vote"]),
        ("security", vec!["audit", "vote"]),
    ]).await;
    
    // Create deployment proposal
    let graph = create_database_cluster_graph();
    let proposal = DeploymentProposal::from_graph(&graph);
    
    // Agents discuss
    let dialog = agents.start_dialog("deployment-discussion").await;
    dialog.add_message("deployer", "Proposing database cluster deployment").await;
    dialog.add_message("security", "Need to verify security settings").await;
    
    // Trigger consensus
    let consensus = agents.start_consensus(proposal).await;
    
    // Agents vote based on their analysis
    consensus.cast_vote("deployer", Vote::Approve).await;
    consensus.cast_vote("monitor", Vote::Approve).await;
    consensus.cast_vote("security", Vote::ApproveWithConditions {
        conditions: vec!["Enable encryption at rest"],
    }).await;
    
    // Check result
    let result = consensus.wait_for_result().await?;
    assert_eq!(result, ConsensusResult::ApprovedWithConditions);
    
    // Verify conditions are applied
    let updated_graph = apply_conditions(&graph, result.conditions);
    let final_flake = updated_graph.to_flake_nix()?;
    assert!(final_flake.contains("encryption = true"));
}
```

#### Knowledge Sharing Tests
```rust
#[tokio::test]
async fn test_knowledge_synchronization() {
    let test_env = TestEnvironment::new().await;
    let agents = test_env.spawn_agents(3).await;
    
    // Agent 1 learns a fact
    agents[0].add_knowledge(Fact {
        id: "service-port",
        subject: "web-api",
        predicate: "listens-on",
        object: "8080",
        confidence: 1.0,
    }).await;
    
    // Wait for sync
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Verify all agents have the fact
    for agent in &agents {
        let knowledge = agent.query_knowledge("web-api", "listens-on").await?;
        assert_eq!(knowledge.len(), 1);
        assert_eq!(knowledge[0].object, "8080");
    }
}

#[tokio::test]
async fn test_knowledge_conflict_resolution() {
    let test_env = TestEnvironment::new().await;
    let agents = test_env.spawn_agents(2).await;
    
    // Conflicting facts
    agents[0].add_knowledge(Fact {
        subject: "database",
        predicate: "version",
        object: "14.5",
        timestamp: Utc::now(),
    }).await;
    
    agents[1].add_knowledge(Fact {
        subject: "database",
        predicate: "version", 
        object: "15.1",
        timestamp: Utc::now() + Duration::from_secs(1),
    }).await;
    
    // Wait for conflict resolution
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Verify latest timestamp wins
    let resolved = agents[0].query_knowledge("database", "version").await?;
    assert_eq!(resolved[0].object, "15.1");
}
```

### 3. Scenario Tests

#### Complete CIM Leaf Deployment
```rust
#[tokio::test]
async fn test_production_like_deployment() {
    let test_env = TestEnvironment::new().await;
    
    // 1. Human creates deployment graph in UI
    let graph = GraphBuilder::new()
        .add_service("api-gateway", ServiceSpec {
            port: 443,
            tls: true,
            replicas: 2,
        })
        .add_service("web-api", ServiceSpec {
            port: 8080,
            command: "cargo run --release",
        })
        .add_database("postgres", DatabaseSpec {
            version: "15",
            persistent: true,
            backup_schedule: "0 2 * * *",
        })
        .add_message_bus("nats", NatsSpec {
            cluster_size: 3,
            jetstream: true,
        })
        .add_dependency("api-gateway", "web-api")
        .add_dependency("web-api", "postgres")
        .add_dependency("web-api", "nats")
        .build();
    
    // 2. Agents analyze the deployment
    let agents = test_env.spawn_production_agents().await;
    let analysis = agents.analyze_deployment(&graph).await?;
    
    assert!(analysis.issues.is_empty());
    assert!(analysis.recommendations.contains("Enable rate limiting on api-gateway"));
    
    // 3. Agents discuss and improve
    let dialog = agents.collaborative_improvement_dialog(&graph).await;
    let improved_graph = dialog.get_improved_graph().await?;
    
    // 4. Consensus on deployment
    let consensus = agents.deployment_consensus(&improved_graph).await?;
    assert_eq!(consensus.result, ConsensusResult::Approved);
    
    // 5. Generate and validate flake.nix
    let flake = improved_graph.to_flake_nix()?;
    let validation = validate_flake(&flake)?;
    assert!(validation.is_valid);
    
    // 6. Deploy with monitoring
    let deployment = test_env.deploy_with_monitoring(&flake, &agents).await?;
    
    // 7. Verify all services are running
    assert!(deployment.wait_for_healthy(Duration::from_secs(60)).await?);
    
    // 8. Agents verify deployment state
    for agent in &agents {
        let state = agent.verify_deployment_state(&deployment).await?;
        assert_eq!(state, ExpectedState::Healthy);
    }
    
    // 9. Test rollback capability
    let rollback_graph = improved_graph.with_failure_injection("web-api");
    let failed_deployment = test_env.deploy(&rollback_graph.to_flake_nix()?).await;
    
    // Agents detect failure and coordinate rollback
    let rollback_consensus = agents.rollback_consensus(&failed_deployment).await?;
    assert_eq!(rollback_consensus.result, ConsensusResult::Approved);
    
    let rollback = test_env.rollback_deployment(&failed_deployment).await?;
    assert!(rollback.is_successful());
}
```

### 4. Performance Tests

```rust
#[tokio::test]
async fn test_graph_translation_performance() {
    let sizes = vec![10, 50, 100, 500, 1000];
    
    for size in sizes {
        let graph = generate_random_deployment_graph(size);
        let translator = GraphToNixTranslator::new();
        
        let start = Instant::now();
        let _ = translator.translate_graph(&graph)?;
        let duration = start.elapsed();
        
        println!("Translation time for {} nodes: {:?}", size, duration);
        assert!(duration < Duration::from_secs(1), "Translation too slow for {} nodes", size);
    }
}

#[tokio::test]
async fn test_consensus_scalability() {
    let agent_counts = vec![5, 10, 20, 50];
    
    for count in agent_counts {
        let test_env = TestEnvironment::new().await;
        let agents = test_env.spawn_agents(count).await;
        
        let proposal = create_test_proposal();
        let start = Instant::now();
        
        let consensus = agents.start_consensus(proposal).await;
        
        // All agents vote
        for agent in &agents {
            consensus.cast_vote(agent.id(), Vote::Approve).await;
        }
        
        let result = consensus.wait_for_result().await?;
        let duration = start.elapsed();
        
        println!("Consensus time for {} agents: {:?}", count, duration);
        assert!(duration < Duration::from_secs(30), "Consensus too slow for {} agents", count);
    }
}
```

### 5. Failure Tests

```rust
#[tokio::test]
async fn test_agent_failure_during_deployment() {
    let test_env = TestEnvironment::new().await;
    let agents = test_env.spawn_agents(5).await;
    
    // Start deployment process
    let graph = create_test_deployment_graph();
    let deployment = agents.start_deployment(&graph).await;
    
    // Kill one agent mid-deployment
    agents[2].kill().await;
    
    // Deployment should still succeed with remaining agents
    let result = deployment.wait_for_completion().await?;
    assert_eq!(result.status, DeploymentStatus::Success);
    
    // Verify remaining agents updated their knowledge
    for agent in agents.iter().filter(|a| a.is_alive()) {
        let knowledge = agent.query_knowledge("deployment", &deployment.id).await?;
        assert!(knowledge.contains_fact("status", "success"));
    }
}

#[tokio::test]
async fn test_network_partition_during_consensus() {
    let test_env = TestEnvironment::new().await;
    let agents = test_env.spawn_agents(6).await;
    
    // Start consensus
    let proposal = create_test_proposal();
    let consensus = agents.start_consensus(proposal).await;
    
    // Create network partition (3 agents on each side)
    test_env.create_partition(vec![0, 1, 2], vec![3, 4, 5]).await;
    
    // Each partition tries to reach consensus
    for i in 0..3 {
        consensus.cast_vote(agents[i].id(), Vote::Approve).await;
        consensus.cast_vote(agents[i+3].id(), Vote::Reject).await;
    }
    
    // Consensus should fail due to partition
    let result = consensus.wait_for_result_timeout(Duration::from_secs(10)).await;
    assert!(matches!(result, Err(ConsensusError::NetworkPartition)));
    
    // Heal partition
    test_env.heal_partition().await;
    
    // Consensus should recover and complete
    let final_result = consensus.wait_for_result().await?;
    assert!(matches!(final_result, ConsensusResult::Failed { reason: _ }));
}
```

## Test Execution Plan

### Phase 1: Unit Tests (Week 1)
- [ ] Graph translation unit tests
- [ ] Agent coordination unit tests
- [ ] Consensus protocol unit tests
- [ ] Knowledge base unit tests

### Phase 2: Integration Tests (Week 2)
- [ ] End-to-end deployment tests
- [ ] Multi-agent collaboration tests
- [ ] Knowledge synchronization tests
- [ ] Dialog routing tests

### Phase 3: Scenario Tests (Week 3)
- [ ] Complete CIM Leaf deployment
- [ ] Complex multi-service scenarios
- [ ] Rollback and recovery scenarios
- [ ] Agent learning scenarios

### Phase 4: Performance & Failure Tests (Week 4)
- [ ] Translation performance benchmarks
- [ ] Consensus scalability tests
- [ ] Agent failure recovery tests
- [ ] Network partition tests

## Success Metrics

1. **Test Coverage**
   - Unit tests: > 95%
   - Integration tests: > 80%
   - Scenario tests: All passing

2. **Performance Benchmarks**
   - Graph translation: < 1s for 100 nodes
   - Consensus: < 30s for 10 agents
   - Knowledge sync: < 100ms propagation

3. **Reliability**
   - All failure tests passing
   - Graceful degradation verified
   - Recovery mechanisms tested

## Continuous Testing

- Run unit tests on every commit
- Run integration tests on every PR
- Run full test suite nightly
- Performance benchmarks weekly
- Chaos testing monthly