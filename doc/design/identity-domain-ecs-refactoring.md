# Identity Domain ECS Refactoring Plan

## Overview

The Identity domain needs to be refactored to:
1. Use the new ECS-based architecture
2. Eliminate duplication with other domains
3. Focus on identity relationships and workflows
4. Integrate with the new authentication system in cim-domain-policy

## Domain Boundaries

### What Identity Domain OWNS:
- **Identity Aggregates**: Core identity entities and their lifecycle
- **Identity Relationships**: How identities relate to each other
- **Identity Workflows**: Processes for identity verification, onboarding, etc.
- **Identity Projections**: How identities are represented in different contexts

### What Identity Domain DELEGATES:

1. **To cim-domain-person**:
   - Personal information details (name, address, phone)
   - Person-specific attributes
   - Personal preferences

2. **To cim-domain-organization**:
   - Organization structure and hierarchy
   - Organization-specific attributes
   - Membership management

3. **To cim-domain-policy**:
   - Authentication policies and rules
   - Authorization decisions
   - Security enforcement

4. **To cim-security**:
   - Cryptographic operations
   - Key management
   - Token generation/validation

5. **To cim-keys**:
   - SSH keys
   - GPG keys
   - API keys storage

6. **To cim-bridge**:
   - Cross-domain event translation
   - Protocol adaptation
   - External system integration

## New Architecture

### Components (ECS)

```rust
// Core identity components
#[derive(Component)]
pub struct IdentityEntity {
    pub identity_id: IdentityId,
    pub identity_type: IdentityType,
}

#[derive(Component)]
pub struct IdentityRelationship {
    pub from_identity: IdentityId,
    pub to_identity: IdentityId,
    pub relationship_type: RelationshipType,
}

#[derive(Component)]
pub struct IdentityWorkflow {
    pub workflow_id: WorkflowId,
    pub identity_id: IdentityId,
    pub workflow_type: IdentityWorkflowType,
    pub current_state: WorkflowState,
}

#[derive(Component)]
pub struct IdentityProjection {
    pub identity_id: IdentityId,
    pub projection_type: ProjectionType,
    pub context: ProjectionContext,
}
```

### Systems

```rust
// Identity lifecycle systems
pub fn create_identity_system() { }
pub fn update_identity_system() { }
pub fn merge_identities_system() { }
pub fn archive_identity_system() { }

// Relationship systems
pub fn establish_relationship_system() { }
pub fn validate_relationship_system() { }
pub fn traverse_relationships_system() { }

// Workflow systems
pub fn start_verification_workflow_system() { }
pub fn process_workflow_step_system() { }
pub fn complete_workflow_system() { }

// Projection systems
pub fn project_to_person_system() { }
pub fn project_to_organization_system() { }
pub fn project_to_security_context_system() { }
```

### Events

```rust
// Identity lifecycle events
pub struct IdentityCreated { }
pub struct IdentityUpdated { }
pub struct IdentitiesMerged { }
pub struct IdentityArchived { }

// Relationship events
pub struct RelationshipEstablished { }
pub struct RelationshipRevoked { }
pub struct RelationshipTypeChanged { }

// Workflow events
pub struct VerificationWorkflowStarted { }
pub struct VerificationStepCompleted { }
pub struct VerificationWorkflowCompleted { }

// Cross-domain events
pub struct IdentityLinkedToPerson { }
pub struct IdentityLinkedToOrganization { }
pub struct IdentityAuthenticationRequested { }
```

## Migration Strategy

### Phase 1: Core Refactoring
1. Create new ECS components
2. Migrate aggregates to components
3. Convert command handlers to systems
4. Update event structures

### Phase 2: Cross-Domain Integration
1. Remove duplicate value objects
2. Create cross-domain event handlers
3. Implement projection systems
4. Update tests

### Phase 3: Workflow Implementation
1. Design identity workflows
2. Implement workflow systems
3. Create workflow UI components
4. Add workflow tests

## Key Design Decisions

1. **Identity as Relationship Hub**: The identity domain becomes the central point for managing relationships between entities across domains.

2. **Workflow-Centric**: Focus on identity workflows (verification, onboarding, merging) rather than data storage.

3. **Projection-Based Integration**: Use projections to represent identities in different contexts without duplicating data.

4. **Event-Driven Coordination**: All cross-domain interactions happen through events, maintaining loose coupling.

## Benefits

1. **Clear Boundaries**: Each domain has a specific responsibility
2. **No Duplication**: Shared concepts are owned by one domain
3. **Flexibility**: Easy to add new identity types or workflows
4. **Testability**: Each system can be tested independently
5. **Performance**: ECS enables efficient batch processing 