# Identity Domain Developer Guide

## Introduction

This guide provides developers with comprehensive information on working with the Identity Domain in the CIM system. The Identity Domain manages relationships between identities and orchestrates verification workflows using an Entity Component System (ECS) architecture.

## Quick Start

### Setting Up Your Development Environment

1. **Add Dependencies**
```toml
[dependencies]
cim-domain-identity = { path = "../cim-domain-identity" }
bevy_ecs = "0.14"
uuid = { version = "1.0", features = ["v4", "serde"] }
serde = { version = "1.0", features = ["derive"] }
```

2. **Import Required Types**
```rust
use cim_domain_identity::{
    // Components
    IdentityEntity, IdentityRelationship, IdentityWorkflow,
    
    // Commands
    CreateIdentityCommand, UpdateIdentityCommand,
    EstablishRelationshipCommand,
    
    // Events
    IdentityCreated, RelationshipEstablished,
    
    // Queries
    find_identity_by_id, find_relationships_by_identity,
    
    // Value Objects
    IdentityType, VerificationLevel, RelationshipType,
};
```

### Basic Usage Example

```rust
use bevy_ecs::prelude::*;
use uuid::Uuid;

// Create a new identity
fn create_user_identity(
    mut commands: EventWriter<CreateIdentityCommand>,
    person_id: Uuid,
) {
    commands.write(CreateIdentityCommand {
        identity_type: IdentityType::Person,
        external_reference: person_id.to_string(),
        initial_verification_level: VerificationLevel::Unverified,
        claims: vec![],
    });
}

// Query an identity
fn get_user_identity(
    world: &mut World,
    identity_id: Uuid,
) -> Option<IdentityView> {
    find_identity_by_id(world, identity_id)
}
```

## Architecture Overview

### Domain Boundaries

The Identity Domain focuses on:
- **Relationships**: Managing connections between identities
- **Workflows**: Orchestrating multi-step processes
- **Verification**: Managing verification states and processes
- **Projections**: Cross-domain references

It delegates to other domains:
- Person details → `cim-domain-person`
- Organization details → `cim-domain-organization`
- Authentication → `cim-domain-policy`
- Cryptography → `cim-security`

### ECS Architecture

```
┌─────────────┐     ┌──────────┐     ┌────────┐
│  Commands   │────▶│ Systems  │────▶│ Events │
└─────────────┘     └──────────┘     └────────┘
                          │
                          ▼
                    ┌──────────┐
                    │Components│
                    └──────────┘
```

## Working with Identities

### Creating an Identity

```rust
// Step 1: Define the command
let create_cmd = CreateIdentityCommand {
    identity_type: IdentityType::Person,
    external_reference: person_id.to_string(),
    initial_verification_level: VerificationLevel::Unverified,
    claims: vec![
        IdentityClaim {
            claim_id: Uuid::new_v4(),
            identity_id: Uuid::nil(), // Will be set by system
            claim_type: ClaimType::Email,
            value: "user@example.com".to_string(),
            verified: false,
            issued_at: SystemTime::now(),
            verified_at: None,
        }
    ],
};

// Step 2: Send the command
commands.write(create_cmd);

// Step 3: Listen for the creation event
fn handle_identity_created(
    mut events: EventReader<IdentityCreated>,
) {
    for event in events.read() {
        println!("Identity created: {}", event.identity_id);
    }
}
```

### Updating an Identity

```rust
// Update verification level
let update_cmd = UpdateIdentityCommand {
    identity_id,
    verification_level: Some(VerificationLevel::Enhanced),
    add_claims: vec![],
    remove_claim_ids: vec![],
};

commands.write(update_cmd);
```

### Querying Identities

```rust
// Find by ID
if let Some(identity) = find_identity_by_id(&mut world, identity_id) {
    println!("Identity type: {:?}", identity.identity_type);
    println!("Verification: {:?}", identity.verification_level);
}

// Find by type
let all_persons = find_identities_by_type(&mut world, IdentityType::Person);
println!("Found {} person identities", all_persons.len());

// Find verified identities
let verified = find_identities_by_verification_level(
    &mut world,
    VerificationLevel::Enhanced
);
```

## Managing Relationships

### Establishing Relationships

```rust
// Create employment relationship
let establish_cmd = EstablishRelationshipCommand {
    source_identity: employee_identity_id,
    target_identity: company_identity_id,
    relationship_type: RelationshipType::EmployedBy,
    metadata: HashMap::from([
        ("department".to_string(), "Engineering".to_string()),
        ("role".to_string(), "Senior Developer".to_string()),
        ("start_date".to_string(), "2024-01-15".to_string()),
    ]),
    expires_at: None,
};

commands.write(establish_cmd);
```

### Querying Relationships

```rust
// Find all relationships for an identity
let relationships = find_relationships_by_identity(&mut world, identity_id);

for rel in relationships {
    match rel.relationship_type {
        RelationshipType::EmployedBy => {
            println!("Employed by: {}", rel.target_identity);
        }
        RelationshipType::MemberOf => {
            println!("Member of: {}", rel.target_identity);
        }
        _ => {}
    }
}
```

### Relationship Patterns

```rust
// Bidirectional relationships
fn create_partnership(
    mut commands: EventWriter<EstablishRelationshipCommand>,
    partner_a: Uuid,
    partner_b: Uuid,
) {
    // A partners with B
    commands.write(EstablishRelationshipCommand {
        source_identity: partner_a,
        target_identity: partner_b,
        relationship_type: RelationshipType::PartnerOf,
        metadata: HashMap::new(),
        expires_at: None,
    });

    // B partners with A (bidirectional)
    commands.write(EstablishRelationshipCommand {
        source_identity: partner_b,
        target_identity: partner_a,
        relationship_type: RelationshipType::PartnerOf,
        metadata: HashMap::new(),
        expires_at: None,
    });
}
```

## Verification Workflows

### Starting Verification

```rust
// Start email verification
let verify_cmd = StartVerificationCommand {
    identity_id,
    verification_method: VerificationMethod::Email,
    initiated_by: admin_id,
};

commands.write(verify_cmd);

// The system will:
// 1. Create a verification workflow
// 2. Send verification instructions
// 3. Track progress through steps
// 4. Update verification level on completion
```

### Monitoring Verification Progress

```rust
fn monitor_verification(
    workflows: Query<&IdentityWorkflow>,
) {
    for workflow in workflows.iter() {
        if workflow.workflow_type == WorkflowType::Verification {
            match &workflow.status {
                WorkflowStatus::InProgress => {
                    println!("Verification in progress: {:?}", workflow.current_step);
                }
                WorkflowStatus::Completed => {
                    println!("Verification completed!");
                }
                WorkflowStatus::Failed(reason) => {
                    println!("Verification failed: {}", reason);
                }
                _ => {}
            }
        }
    }
}
```

### Multi-factor Verification

```rust
// Require multiple verification methods
async fn setup_multi_factor_verification(
    identity_id: Uuid,
    commands: &mut EventWriter<StartVerificationCommand>,
) {
    // Step 1: Email verification
    commands.write(StartVerificationCommand {
        identity_id,
        verification_method: VerificationMethod::Email,
        initiated_by: system_id,
    });

    // Step 2: Phone verification (after email completes)
    commands.write(StartVerificationCommand {
        identity_id,
        verification_method: VerificationMethod::Phone,
        initiated_by: system_id,
    });

    // Step 3: Document verification (for full verification)
    commands.write(StartVerificationCommand {
        identity_id,
        verification_method: VerificationMethod::Document,
        initiated_by: system_id,
    });
}
```

## Cross-Domain Integration

### Listening to Other Domains

```rust
// React to person creation
fn handle_person_created(
    mut events: EventReader<PersonCreated>,
    mut commands: EventWriter<CreateIdentityCommand>,
) {
    for event in events.read() {
        // Create identity for new person
        commands.write(CreateIdentityCommand {
            identity_type: IdentityType::Person,
            external_reference: event.person_id.to_string(),
            initial_verification_level: VerificationLevel::Unverified,
            claims: vec![],
        });
    }
}
```

### Publishing to Other Domains

```rust
// When identity is verified, notify other domains
fn handle_verification_completed(
    mut events: EventReader<VerificationCompleted>,
    mut person_events: EventWriter<PersonVerifiedEvent>,
) {
    for event in events.read() {
        if event.verification_successful {
            // Notify person domain
            person_events.write(PersonVerifiedEvent {
                person_id: parse_external_ref(&event.identity_id),
                verification_level: event.new_verification_level,
            });
        }
    }
}
```

## Testing

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_create_identity() {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        // Add systems
        schedule.add_systems(create_identity_system);

        // Send command
        world.send_event(CreateIdentityCommand {
            identity_type: IdentityType::Person,
            external_reference: "test-person".to_string(),
            initial_verification_level: VerificationLevel::Unverified,
            claims: vec![],
        });

        // Run systems
        schedule.run(&mut world);

        // Verify entity was created
        let identities = world.query::<&IdentityEntity>();
        assert_eq!(identities.iter(&world).count(), 1);
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_identity_workflow() {
    let mut app = App::new();
    
    // Add identity domain plugin
    app.add_plugins(IdentityDomainPlugin);

    // Create identity
    app.world.send_event(CreateIdentityCommand { /* ... */ });
    app.update();

    // Start verification
    let identity_id = /* get from creation event */;
    app.world.send_event(StartVerificationCommand {
        identity_id,
        verification_method: VerificationMethod::Email,
        initiated_by: Uuid::new_v4(),
    });
    app.update();

    // Check workflow started
    let workflows = app.world.query::<&IdentityWorkflow>();
    assert_eq!(workflows.iter(&app.world).count(), 1);
}
```

## Performance Optimization

### Query Optimization

```rust
// Good: Specific queries
Query<(&IdentityEntity, &IdentityVerification), With<ActiveMarker>>

// Avoid: Broad queries
Query<&IdentityEntity>

// Use filters for better performance
fn find_unverified_persons(
    identities: Query<
        &IdentityEntity,
        (
            With<IdentityType>,
            Without<VerifiedMarker>
        )
    >,
) {
    // Process only unverified person identities
}
```

### Batch Processing

```rust
// Process multiple identities efficiently
fn batch_verify_identities(
    mut commands: EventWriter<UpdateIdentityCommand>,
    identity_ids: Vec<Uuid>,
) {
    // Collect all commands
    let updates: Vec<_> = identity_ids
        .into_iter()
        .map(|id| UpdateIdentityCommand {
            identity_id: id,
            verification_level: Some(VerificationLevel::Basic),
            ..Default::default()
        })
        .collect();

    // Send in batch
    for update in updates {
        commands.write(update);
    }
}
```

## Common Patterns

### Workflow Orchestration

```rust
// Complex workflow with multiple steps
fn orchestrate_onboarding(
    identity_id: Uuid,
    commands: &mut Commands,
) {
    // Step 1: Create workflow
    commands.spawn(IdentityWorkflow {
        workflow_id: Uuid::new_v4(),
        identity_id,
        workflow_type: WorkflowType::Onboarding,
        status: WorkflowStatus::InProgress,
        current_step: Some("verification".to_string()),
        started_at: SystemTime::now(),
        timeout_at: Some(SystemTime::now() + Duration::from_days(7)),
    });

    // Workflow will progress through:
    // 1. Identity verification
    // 2. Document collection
    // 3. Approval process
    // 4. Account activation
}
```

### Event Correlation

```rust
// Track related events
fn correlate_identity_events(
    created: EventReader<IdentityCreated>,
    verified: EventReader<VerificationCompleted>,
) {
    // Match verification events to creation events
    // Using correlation IDs or timestamps
}
```

## Troubleshooting

### Common Issues

1. **Identity Not Found**
   - Ensure identity was created successfully
   - Check for typos in UUID
   - Verify query is using correct World instance

2. **Relationship Validation Failures**
   - Both identities must exist
   - Relationship type must be valid
   - Check for duplicate relationships

3. **Workflow Timeouts**
   - Monitor workflow progress
   - Extend timeouts for long-running processes
   - Implement retry logic

### Debugging Tips

```rust
// Enable detailed logging
use log::{debug, info, warn};

fn debug_identity_state(
    world: &World,
    identity_id: Uuid,
) {
    if let Some(identity) = find_identity_by_id(world, identity_id) {
        debug!("Identity: {:?}", identity);
        
        let relationships = find_relationships_by_identity(world, identity_id);
        debug!("Relationships: {} found", relationships.len());
        
        // Check for active workflows
        let workflows = world.query::<&IdentityWorkflow>()
            .iter(world)
            .filter(|w| w.identity_id == identity_id)
            .count();
        debug!("Active workflows: {}", workflows);
    } else {
        warn!("Identity {} not found", identity_id);
    }
}
```

## Best Practices Summary

1. **Use Commands for All Modifications** - Never modify components directly
2. **Listen to Events for Integration** - React to domain events for loose coupling
3. **Validate Early** - Check business rules before processing commands
4. **Handle Errors Gracefully** - Emit failure events for monitoring
5. **Test Thoroughly** - Unit test systems, integration test workflows
6. **Monitor Performance** - Use specific queries and batch operations
7. **Document Domain Logic** - Keep business rules visible in code

## Conclusion

The Identity Domain provides a powerful, flexible system for managing identities and their relationships. By following ECS patterns and event-driven architecture, you can build scalable, maintainable identity management solutions that integrate seamlessly with other domains in the CIM system. 