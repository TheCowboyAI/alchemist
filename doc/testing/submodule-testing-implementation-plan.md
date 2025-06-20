# Event-Driven Testing Framework Implementation Plan

## Overview

This plan details the systematic implementation of event-driven testing across ALL CIM submodules. The goal is to ensure every component properly publishes events to NATS JetStream in the correct sequence with proper correlation/causation chains.

## Implementation Phases

### Phase 1: Infrastructure Layer (Days 1-2)
Foundation modules that all others depend on.

#### 1.1 cim-infrastructure
- [ ] Add `tests/event_flow_tests.rs`
- [ ] Test NATS client creation and connection
- [ ] Test JetStream stream creation
- [ ] Test event publishing with CID chains
- [ ] Test event consumption and replay
- [ ] Validate correlation/causation ID propagation

#### 1.2 cim-ipld
- [ ] Add `tests/event_flow_tests.rs`
- [ ] Test object storage events
- [ ] Test CID generation events
- [ ] Test chain validation events
- [ ] Ensure all storage operations emit events

#### 1.3 cim-bridge
- [ ] Add `tests/event_flow_tests.rs`
- [ ] Test async/sync bridge event flow
- [ ] Test command → event transformation
- [ ] Test event batching and ordering
- [ ] Validate no events are lost in bridging

#### 1.4 cim-keys
- [ ] Add `tests/event_flow_tests.rs`
- [ ] Test key generation events
- [ ] Test authentication events
- [ ] Test YubiKey interaction events
- [ ] Ensure security events are properly logged

### Phase 2: Domain Fundamentals (Days 3-4)
Core domain infrastructure that all domains use.

#### 2.1 cim-domain
- [ ] Add `tests/event_flow_tests.rs`
- [ ] Test base aggregate event handling
- [ ] Test command handler → event flow
- [ ] Test event store integration
- [ ] Validate event metadata (timestamps, IDs)

#### 2.2 cim-component
- [ ] Add `tests/event_flow_tests.rs`
- [ ] Test component lifecycle events
- [ ] Test component state change events
- [ ] Ensure ECS changes emit domain events

#### 2.3 cim-subject
- [ ] Add `tests/event_flow_tests.rs`
- [ ] Test subject hierarchy events
- [ ] Test permission change events
- [ ] Validate subject-based routing

### Phase 3: Domain Implementation (Days 5-10)
Individual domain modules with their specific event flows.

#### 3.1 cim-domain-graph
**User Story**: "As a developer, I want to create a graph and see all events published"
- [ ] Test: Create graph → `GraphCreated` event
- [ ] Test: Add node → `NodeAdded` event with graph context
- [ ] Test: Connect nodes → `EdgeConnected` event with both node IDs
- [ ] Test: Update node → `NodeRemoved` + `NodeAdded` events
- [ ] Validate event sequence and correlation IDs

#### 3.2 cim-domain-identity
**User Story**: "As a system, I want to track all identity operations"
- [ ] Test: Create person → `PersonCreated` event
- [ ] Test: Create organization → `OrganizationCreated` event
- [ ] Test: Associate identities → `IdentityAssociated` event
- [ ] Test: Authentication → `AuthenticationAttempted` + `AuthenticationSucceeded/Failed`
- [ ] Validate security event chains

#### 3.3 cim-domain-person
**User Story**: "As a user, I want all person data changes tracked"
- [ ] Test: Add contact info → `ContactInfoAdded` event
- [ ] Test: Update profile → `ProfileUpdated` event
- [ ] Test: Add relationship → `RelationshipAdded` event
- [ ] Ensure PII is properly handled in events

#### 3.4 cim-domain-agent
**User Story**: "As an AI agent, I want my interactions logged"
- [ ] Test: Agent created → `AgentCreated` event
- [ ] Test: Agent activated → `AgentActivated` event
- [ ] Test: Agent processes → `AgentProcessingStarted` + `AgentProcessingCompleted`
- [ ] Test: Agent response → `AgentResponded` event with correlation to request

#### 3.5 cim-domain-git
**User Story**: "As a developer, I want Git operations to generate graph events"
- [ ] Test: Import repo → `RepositoryImported` event
- [ ] Test: Process commit → `CommitProcessed` + `NodeAdded` events
- [ ] Test: Process branch → `BranchProcessed` + `EdgeConnected` events
- [ ] Validate cross-domain event generation

#### 3.6 cim-domain-location
**User Story**: "As a user, I want location data properly tracked"
- [ ] Test: Create location → `LocationCreated` event
- [ ] Test: Update coordinates → `LocationUpdated` event
- [ ] Test: Add to region → `LocationAddedToRegion` event
- [ ] Test spatial indexing events

#### 3.7 cim-domain-conceptualspaces
**User Story**: "As a system, I want conceptual space calculations tracked"
- [ ] Test: Create space → `ConceptualSpaceCreated` event
- [ ] Test: Add concept → `ConceptAdded` event with embeddings
- [ ] Test: Calculate similarity → `SimilarityCalculated` event
- [ ] Test: Form region → `RegionFormed` event

#### 3.8 cim-domain-workflow
**User Story**: "As a user, I want to see workflow execution events"
- [ ] Test: Create workflow → `WorkflowCreated` event
- [ ] Test: Start execution → `WorkflowStarted` event
- [ ] Test: Complete step → `WorkflowStepCompleted` event
- [ ] Test: Workflow done → `WorkflowCompleted` event with results

#### 3.9 cim-domain-dialog
**User Story**: "As an AI, I want conversations fully tracked"
- [ ] Test: Start dialog → `DialogStarted` event
- [ ] Test: Add message → `MessageAdded` event with participant
- [ ] Test: Switch topic → `TopicSwitched` event
- [ ] Test: End dialog → `DialogEnded` event with summary

#### 3.10 cim-domain-document
**User Story**: "As a user, I want document operations tracked"
- [ ] Test: Upload document → `DocumentUploaded` event
- [ ] Test: Process content → `DocumentProcessed` event
- [ ] Test: Extract metadata → `MetadataExtracted` event
- [ ] Test: Version document → `DocumentVersioned` event

#### 3.11 cim-domain-policy
**User Story**: "As an admin, I want policy changes tracked"
- [ ] Test: Create policy → `PolicyCreated` event
- [ ] Test: Activate policy → `PolicyActivated` event
- [ ] Test: Policy evaluation → `PolicyEvaluated` event
- [ ] Test: Policy violation → `PolicyViolated` event

#### 3.12 cim-domain-organization
**User Story**: "As a manager, I want org changes tracked"
- [ ] Test: Create org → `OrganizationCreated` event
- [ ] Test: Add member → `MemberAdded` event
- [ ] Test: Change structure → `StructureChanged` event
- [ ] Test: Update roles → `RoleUpdated` event

#### 3.13 cim-domain-nix
**User Story**: "As a DevOps engineer, I want Nix operations tracked"
- [ ] Test: Generate config → `NixConfigGenerated` event
- [ ] Test: Build derivation → `DerivationBuilt` event
- [ ] Test: Deploy config → `ConfigDeployed` event
- [ ] Test: Rollback → `ConfigRolledBack` event

### Phase 4: Cross-Domain Integration (Days 11-12)
Modules that compose multiple domains.

#### 4.1 cim-compose
- [ ] Test cross-domain event choreography
- [ ] Test event correlation across domains
- [ ] Validate no events lost in composition

#### 4.2 cim-contextgraph
- [ ] Test graph projection events
- [ ] Test context switching events
- [ ] Test JSON/DOT export events

#### 4.3 cim-conceptgraph
- [ ] Test concept composition events
- [ ] Test graph merging events
- [ ] Test conceptual analysis events

#### 4.4 cim-workflow-graph
- [ ] Test workflow visualization events
- [ ] Test execution tracking events
- [ ] Test workflow composition events

#### 4.5 cim-ipld-graph
- [ ] Test IPLD graph storage events
- [ ] Test graph retrieval events
- [ ] Test CID-based navigation events

### Phase 5: Full System Validation (Days 13-14)
End-to-end testing from UI to persistence.

#### 5.1 cim-domain-bevy
- [ ] Test UI → NATS event publishing
- [ ] Test Bevy event → Domain event mapping
- [ ] Fix missing event publishing (CRITICAL)
- [ ] Validate correlation IDs from UI actions

#### 5.2 cim-agent-alchemist
- [ ] Test agent question → event flow
- [ ] Test agent response → event flow
- [ ] Validate conversation tracking
- [ ] Test memory system integration

#### 5.3 Main Application
- [ ] Full end-to-end event flow test
- [ ] Test event replay from JetStream
- [ ] Validate system recovery from events
- [ ] Performance testing with high event volume

## Test Template

Each submodule should implement tests following this template:

```rust
// tests/event_flow_tests.rs
use cim_infrastructure::testing::EventStreamValidator;

#[tokio::test]
async fn test_domain_operation_event_flow() {
    // Setup
    let validator = EventStreamValidator::new();
    let nats = test_nats_connection().await;
    
    // Define expected events
    let expected = vec![
        ExpectedEvent::new("CommandReceived"),
        ExpectedEvent::new("AggregateValidated"),
        ExpectedEvent::new("EventGenerated"),
        ExpectedEvent::new("EventPublished"),
    ];
    
    // Execute operation
    let correlation_id = CorrelationId::new();
    execute_domain_operation(&nats, correlation_id).await?;
    
    // Validate
    let report = validator.validate_sequence(
        &nats,
        correlation_id,
        expected,
        Duration::from_secs(5)
    ).await?;
    
    assert!(report.all_events_found());
    assert!(report.correct_sequence());
    assert!(report.valid_causation_chain());
}
```

## Success Criteria

1. **Every submodule** has event flow tests
2. **All domain operations** publish events to NATS
3. **Event sequences** match expected patterns
4. **Correlation/causation chains** are intact
5. **No CRUD operations** - everything through events
6. **UI events** properly flow to NATS JetStream

## Timeline

- **Days 1-2**: Infrastructure Layer
- **Days 3-4**: Domain Fundamentals  
- **Days 5-10**: Domain Implementation (2-3 domains per day)
- **Days 11-12**: Cross-Domain Integration
- **Days 13-14**: Full System Validation
- **Day 15**: Fix any discovered issues

Total: 15 days to complete implementation across all submodules.

## Priority Order

1. **CRITICAL**: Fix UI → NATS event publishing (cim-domain-bevy)
2. **HIGH**: Infrastructure and domain fundamentals
3. **MEDIUM**: Individual domain implementations
4. **LOW**: Cross-domain integration (after domains work)

## Next Steps

1. Start with `cim-infrastructure` tests
2. Create test utilities that other modules can reuse
3. Document any discovered issues in each module
4. Update progress.json as each module is completed 