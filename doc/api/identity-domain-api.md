# Identity Domain API Documentation

## Overview

The Identity Domain provides a comprehensive API for managing identities, relationships, workflows, and verifications in the CIM system. It uses an Entity Component System (ECS) architecture with event-driven communication.

## Table of Contents

1. [Core Concepts](#core-concepts)
2. [Components](#components)
3. [Commands](#commands)
4. [Events](#events)
5. [Queries](#queries)
6. [Systems](#systems)
7. [Integration Patterns](#integration-patterns)
8. [Error Handling](#error-handling)

## Core Concepts

### Identity
An identity represents a unique entity in the system that can be a Person, Organization, System, or Service. Each identity has:
- Unique identifier (UUID)
- Type classification
- External reference to domain entity
- Verification status
- Associated claims
- Relationships with other identities

### Relationship
A connection between two identities with:
- Source and target identities
- Relationship type (EmployedBy, MemberOf, etc.)
- Optional metadata
- Establishment timestamp
- Optional expiration

### Workflow
A multi-step process for identity operations like verification:
- Workflow type and current status
- Step progression
- Timeout handling
- State persistence

### Projection
Cross-domain reference linking identities to external entities:
- Target domain and entity ID
- Synchronization status
- Bidirectional navigation

## Components

### IdentityEntity
Core identity information.

```rust
#[derive(Component, Debug, Clone)]
pub struct IdentityEntity {
    pub id: Uuid,
    pub identity_type: IdentityType,
    pub external_reference: String,
    pub created_at: SystemTime,
}
```

### IdentityRelationship
Represents a connection between identities.

```rust
#[derive(Component, Debug, Clone)]
pub struct IdentityRelationship {
    pub id: Uuid,
    pub source_identity: Uuid,
    pub target_identity: Uuid,
    pub relationship_type: RelationshipType,
    pub established_at: SystemTime,
    pub expires_at: Option<SystemTime>,
    pub metadata: HashMap<String, String>,
}
```

### IdentityWorkflow
Workflow state for multi-step processes.

```rust
#[derive(Component, Debug, Clone)]
pub struct IdentityWorkflow {
    pub workflow_id: Uuid,
    pub identity_id: Uuid,
    pub workflow_type: WorkflowType,
    pub status: WorkflowStatus,
    pub current_step: Option<String>,
    pub started_at: SystemTime,
    pub timeout_at: Option<SystemTime>,
}
```

### IdentityVerification
Verification state and history.

```rust
#[derive(Component, Debug, Clone)]
pub struct IdentityVerification {
    pub verification_level: VerificationLevel,
    pub last_verified: Option<SystemTime>,
    pub verification_method: Option<VerificationMethod>,
    pub verified_by: Option<Uuid>,
}
```

### IdentityClaim
Claims associated with an identity.

```rust
#[derive(Component, Debug, Clone)]
pub struct IdentityClaim {
    pub claim_id: Uuid,
    pub identity_id: Uuid,
    pub claim_type: ClaimType,
    pub value: String,
    pub verified: bool,
    pub issued_at: SystemTime,
    pub verified_at: Option<SystemTime>,
}
```

## Commands

### CreateIdentityCommand
Creates a new identity.

```rust
pub struct CreateIdentityCommand {
    pub identity_type: IdentityType,
    pub external_reference: String,
    pub initial_verification_level: VerificationLevel,
    pub claims: Vec<IdentityClaim>,
}

// Usage
let command = CreateIdentityCommand {
    identity_type: IdentityType::Person,
    external_reference: person_id.to_string(),
    initial_verification_level: VerificationLevel::Unverified,
    claims: vec![
        IdentityClaim {
            claim_type: ClaimType::Email,
            value: "user@example.com".to_string(),
            verified: false,
            // ... other fields
        }
    ],
};
```

### UpdateIdentityCommand
Updates identity information.

```rust
pub struct UpdateIdentityCommand {
    pub identity_id: Uuid,
    pub verification_level: Option<VerificationLevel>,
    pub add_claims: Vec<IdentityClaim>,
    pub remove_claim_ids: Vec<Uuid>,
}
```

### EstablishRelationshipCommand
Creates a relationship between identities.

```rust
pub struct EstablishRelationshipCommand {
    pub source_identity: Uuid,
    pub target_identity: Uuid,
    pub relationship_type: RelationshipType,
    pub metadata: HashMap<String, String>,
    pub expires_at: Option<SystemTime>,
}

// Usage
let command = EstablishRelationshipCommand {
    source_identity: employee_id,
    target_identity: company_id,
    relationship_type: RelationshipType::EmployedBy,
    metadata: hashmap!{
        "department" => "Engineering".to_string(),
        "role" => "Senior Developer".to_string(),
    },
    expires_at: None,
};
```

### StartWorkflowCommand
Initiates a workflow process.

```rust
pub struct StartWorkflowCommand {
    pub identity_id: Uuid,
    pub workflow_type: WorkflowType,
    pub initial_context: HashMap<String, String>,
    pub timeout: Option<Duration>,
}
```

### StartVerificationCommand
Begins identity verification.

```rust
pub struct StartVerificationCommand {
    pub identity_id: Uuid,
    pub verification_method: VerificationMethod,
    pub initiated_by: Uuid,
}
```

## Events

### IdentityCreated
Emitted when a new identity is created.

```rust
#[derive(Event, Debug, Clone)]
pub struct IdentityCreated {
    pub identity_id: Uuid,
    pub identity_type: IdentityType,
    pub external_reference: String,
    pub initial_verification_level: VerificationLevel,
    pub created_at: SystemTime,
}
```

### RelationshipEstablished
Emitted when a relationship is created.

```rust
#[derive(Event, Debug, Clone)]
pub struct RelationshipEstablished {
    pub relationship_id: Uuid,
    pub source_identity: Uuid,
    pub target_identity: Uuid,
    pub relationship_type: RelationshipType,
    pub established_at: SystemTime,
}
```

### VerificationCompleted
Emitted when verification is completed.

```rust
#[derive(Event, Debug, Clone)]
pub struct VerificationCompleted {
    pub identity_id: Uuid,
    pub verification_successful: bool,
    pub new_verification_level: VerificationLevel,
    pub verified_by: Uuid,
    pub completed_at: SystemTime,
}
```

### WorkflowCompleted
Emitted when a workflow finishes.

```rust
#[derive(Event, Debug, Clone)]
pub struct WorkflowCompleted {
    pub workflow_id: Uuid,
    pub identity_id: Uuid,
    pub workflow_type: WorkflowType,
    pub final_status: WorkflowStatus,
    pub completed_at: SystemTime,
}
```

## Queries

### find_identity_by_id
Retrieves an identity by ID.

```rust
pub fn find_identity_by_id(
    world: &mut World,
    identity_id: Uuid,
) -> Option<IdentityView>

// Usage
if let Some(identity) = find_identity_by_id(&mut world, identity_id) {
    println!("Found identity: {:?}", identity);
}
```

### find_identities_by_type
Finds all identities of a specific type.

```rust
pub fn find_identities_by_type(
    world: &mut World,
    identity_type: IdentityType,
) -> Vec<IdentityView>

// Usage
let person_identities = find_identities_by_type(&mut world, IdentityType::Person);
```

### find_relationships_by_identity
Gets all relationships for an identity.

```rust
pub fn find_relationships_by_identity(
    world: &mut World,
    identity_id: Uuid,
) -> Vec<RelationshipView>

// Usage
let relationships = find_relationships_by_identity(&mut world, identity_id);
for rel in relationships {
    println!("Related to: {}", rel.target_identity);
}
```

### find_identities_by_verification_level
Finds identities with minimum verification level.

```rust
pub fn find_identities_by_verification_level(
    world: &mut World,
    min_level: VerificationLevel,
) -> Vec<IdentityView>

// Usage
let verified_identities = find_identities_by_verification_level(
    &mut world,
    VerificationLevel::Enhanced,
);
```

### Query Views

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityView {
    pub id: Uuid,
    pub identity_type: IdentityType,
    pub external_reference: String,
    pub verification_level: VerificationLevel,
    pub last_verified: Option<SystemTime>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipView {
    pub id: Uuid,
    pub source_identity: Uuid,
    pub target_identity: Uuid,
    pub relationship_type: RelationshipType,
    pub established_at: SystemTime,
    pub expires_at: Option<SystemTime>,
    pub metadata: HashMap<String, String>,
}
```

## Systems

### Lifecycle Systems

#### create_identity_system
Processes CreateIdentityCommand to create new identities.

```rust
pub fn create_identity_system(
    mut commands: Commands,
    mut events: EventReader<CreateIdentityCommand>,
    mut created_events: EventWriter<IdentityCreated>,
    aggregate: Res<IdentityAggregate>,
)
```

#### update_identity_system
Handles identity updates.

```rust
pub fn update_identity_system(
    mut commands: Commands,
    mut events: EventReader<UpdateIdentityCommand>,
    mut updated_events: EventWriter<IdentityUpdated>,
    identities: Query<&IdentityEntity>,
)
```

### Relationship Systems

#### establish_relationship_system
Creates relationships between identities.

```rust
pub fn establish_relationship_system(
    mut commands: Commands,
    mut events: EventReader<EstablishRelationshipCommand>,
    mut established_events: EventWriter<RelationshipEstablished>,
    identities: Query<&IdentityEntity>,
)
```

#### traverse_relationships_system
Traverses relationship graph.

```rust
pub fn traverse_relationships_system(
    relationships: Query<&IdentityRelationship>,
    identities: Query<&IdentityEntity>,
) -> Vec<(Uuid, Uuid, RelationshipType)>
```

### Workflow Systems

#### start_workflow_system
Initiates workflow processes.

```rust
pub fn start_workflow_system(
    mut commands: Commands,
    mut events: EventReader<StartWorkflowCommand>,
    mut started_events: EventWriter<WorkflowStarted>,
)
```

#### process_workflow_steps_system
Advances workflow through steps.

```rust
pub fn process_workflow_steps_system(
    mut commands: Commands,
    mut workflows: Query<&mut IdentityWorkflow>,
    mut step_events: EventWriter<WorkflowStepCompleted>,
)
```

## Integration Patterns

### Event-Driven Integration

Subscribe to identity events for cross-domain integration:

```rust
// System in another domain
fn handle_identity_created(
    mut events: EventReader<IdentityCreated>,
    mut person_commands: EventWriter<CreatePersonCommand>,
) {
    for event in events.read() {
        if event.identity_type == IdentityType::Person {
            // Create corresponding person record
            person_commands.write(CreatePersonCommand {
                person_id: Uuid::parse_str(&event.external_reference).unwrap(),
                // ... other fields
            });
        }
    }
}
```

### Command Pattern

Send commands to modify identity state:

```rust
// In a Bevy system
fn verify_user_identity(
    mut commands: EventWriter<StartVerificationCommand>,
    user_query: Query<&User>,
) {
    for user in user_query.iter() {
        commands.write(StartVerificationCommand {
            identity_id: user.identity_id,
            verification_method: VerificationMethod::Email,
            initiated_by: admin_id,
        });
    }
}
```

### Query Pattern

Read identity data without modification:

```rust
// In a service
async fn get_user_verification_status(
    world: &mut World,
    user_id: Uuid,
) -> Result<VerificationLevel, Error> {
    let identity = find_identity_by_id(world, user_id)
        .ok_or(Error::IdentityNotFound)?;
    
    Ok(identity.verification_level)
}
```

## Error Handling

### IdentityError

```rust
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("Identity not found: {0}")]
    IdentityNotFound(Uuid),
    
    #[error("Invalid relationship: {0}")]
    InvalidRelationship(String),
    
    #[error("Workflow error: {0}")]
    WorkflowError(String),
    
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    
    #[error("Duplicate identity: {0}")]
    DuplicateIdentity(String),
}
```

### Error Handling Pattern

```rust
// In command handler
match aggregate.validate_relationship(source, target, &rel_type) {
    Ok(_) => {
        // Process relationship
        commands.spawn(IdentityRelationship { ... });
        events.write(RelationshipEstablished { ... });
    }
    Err(e) => {
        // Log error and emit failure event
        error!("Failed to establish relationship: {}", e);
        error_events.write(RelationshipFailed {
            reason: e.to_string(),
            // ... other fields
        });
    }
}
```

## Best Practices

### 1. Command Validation
Always validate commands before processing:

```rust
impl IdentityAggregate {
    pub fn validate_create(&self, command: &CreateIdentityCommand) -> Result<(), IdentityError> {
        // Check for duplicate external reference
        // Validate identity type
        // Ensure claims are valid
        Ok(())
    }
}
```

### 2. Event Ordering
Maintain event order per aggregate:

```rust
// Events for same identity are processed in order
let events = vec![
    IdentityCreated { ... },
    ClaimAdded { ... },
    VerificationStarted { ... },
];
```

### 3. Idempotency
Make operations idempotent:

```rust
// Check if relationship already exists
let exists = relationships.iter().any(|r| {
    r.source_identity == source &&
    r.target_identity == target &&
    r.relationship_type == rel_type
});

if !exists {
    // Create relationship
}
```

### 4. Timeout Handling
Set appropriate timeouts for workflows:

```rust
let timeout = match workflow_type {
    WorkflowType::Verification => Duration::from_secs(3600), // 1 hour
    WorkflowType::Migration => Duration::from_secs(86400),   // 24 hours
    _ => Duration::from_secs(7200), // 2 hours default
};
```

## Performance Considerations

### 1. Query Optimization
Use specific queries instead of broad scans:

```rust
// Good: Specific component query
Query<(&IdentityEntity, &IdentityVerification), With<ActiveIdentity>>

// Avoid: Querying all entities
Query<&IdentityEntity>
```

### 2. Batch Operations
Process multiple commands together:

```rust
fn batch_create_identities(
    commands: Vec<CreateIdentityCommand>,
) -> Vec<Result<Uuid, IdentityError>> {
    // Process all commands in single transaction
}
```

### 3. Event Batching
Batch events for network efficiency:

```rust
// Collect events before sending
let mut event_batch = Vec::new();
for cmd in commands {
    event_batch.extend(process_command(cmd)?);
}
publish_events(event_batch).await?;
```

## Migration Guide

### From Traditional Repository Pattern

```rust
// Old: Repository pattern
let identity = identity_repository.find_by_id(id)?;
identity.update_verification_level(VerificationLevel::Enhanced);
identity_repository.save(identity)?;

// New: Command/Event pattern
commands.write(UpdateIdentityCommand {
    identity_id: id,
    verification_level: Some(VerificationLevel::Enhanced),
    ..Default::default()
});
```

### From Direct Database Access

```rust
// Old: Direct SQL
let identities = sqlx::query_as!(
    Identity,
    "SELECT * FROM identities WHERE type = $1",
    identity_type
).fetch_all(&pool).await?;

// New: ECS Query
let identities = find_identities_by_type(&mut world, identity_type);
```

## Conclusion

The Identity Domain API provides a comprehensive, event-driven approach to identity management with strong typing, clear boundaries, and excellent performance characteristics through the ECS architecture. 