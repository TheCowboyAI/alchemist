# Identity Domain ECS Refactoring Progress

## Overview

The identity domain has been refactored to use a pure ECS (Entity Component System) architecture while maintaining DDD (Domain-Driven Design) principles through an aggregate pattern.

## What Was Completed

### 1. Architecture Transformation
- ✅ Removed all legacy code and backward compatibility
- ✅ Implemented full ECS pattern with Bevy
- ✅ Maintained aggregate pattern for business rule enforcement
- ✅ Created clear separation of concerns

### 2. Core Components Created

#### Components (`src/components/`)
- **IdentityEntity**: Core identity component with ID, type, and status
- **IdentityRelationship**: Relationships between identities with rules
- **IdentityWorkflow**: Workflow state and transitions
- **IdentityProjection**: Cross-domain projections
- **IdentityVerification**: Verification levels and methods
- **IdentityClaim**: Claims about identities (email, phone, etc.)

#### Systems (`src/systems/`)
- **Lifecycle Systems**: create, update, merge, archive identities
- **Relationship Systems**: establish, validate, traverse, expire relationships
- **Workflow Systems**: start, process steps, complete, timeout workflows
- **Projection Systems**: create and sync cross-domain projections
- **Verification Systems**: start, process, complete verification

#### Aggregate (`src/aggregate/`)
- **IdentityAggregate**: Enforces business rules and invariants
  - Validates identity creation (no duplicates)
  - Validates status transitions
  - Validates merge operations
  - Validates relationship establishment
  - Validates workflow transitions
  - Validates verification level changes

#### Queries (`src/queries/`)
- Find identity by ID
- Find identities by type
- Find relationships for identity
- Find active workflows
- Get aggregate state
- Find by verification level
- Find by claim
- Traverse relationship graph

#### Projections (`src/projections/`)
- **IdentitySummaryProjection**: Fast lookups by ID, type, claim
- **RelationshipGraphProjection**: Optimized graph traversal
- **WorkflowStatusProjection**: Active workflows and actions required
- **PersonDetailsProjection**: Cross-domain person details
- **OrganizationDetailsProjection**: Cross-domain org details

### 3. Event-Driven Architecture

#### Commands
- Identity lifecycle: Create, Update, Merge, Archive
- Relationships: Establish, Revoke, Traverse
- Workflows: Start, Process Step, Complete, Timeout
- Verification: Start, Process, Complete
- Projections: Create, Sync

#### Events
- Identity: Created, Updated, Merged, Archived
- Relationships: Established, Revoked, Validated, Expired
- Workflows: Started, Step Completed, Completed, Timed Out
- Verification: Started, Completed
- Projections: Created, Synced

## Current Issues to Fix

### 1. Type Issues
- [ ] Add Hash derive to ClaimType and IdentityType enums
- [ ] Fix RelationshipId vs Uuid mismatches
- [ ] Fix WorkflowId type mismatches
- [ ] Add missing IdentityMetadata struct

### 2. Event/Command Mismatches
- [ ] Fix field name mismatches (context vs initial_data)
- [ ] Add missing event types (RelationshipValidated, RelationshipExpired)
- [ ] Fix WorkflowOutcome usage

### 3. Float in Hash/Eq
- [ ] Remove f32 from RelationshipType::Owns or use ordered float

### 4. Deprecated Bevy APIs
- [ ] Change all `send()` to `write()` for EventWriter
- [ ] Update to latest Bevy patterns

### 5. Query Lifetime Issues
- [ ] Fix mutable/immutable borrow conflicts in queries
- [ ] Fix lifetime issues with Deserialize

## Benefits of the Refactoring

1. **Clear Domain Focus**: Identity domain now focuses solely on identity lifecycle, relationships, and workflows
2. **Delegation**: Person/Organization details delegated to respective domains
3. **ECS Performance**: Leverages Bevy's efficient ECS for fast queries and updates
4. **Aggregate Integrity**: Business rules enforced through aggregate pattern
5. **Event Sourcing Ready**: All changes go through events
6. **Projection Support**: Optimized read models for different use cases

## Next Steps

1. Fix compilation errors (see issues above)
2. Add comprehensive tests
3. Integrate with NATS for cross-domain events
4. Add migration guide from old API
5. Performance benchmarks

## Migration Strategy

For existing code using the old identity domain:

1. **Entities**: Convert Person/Organization to IdentityEntity with appropriate type
2. **Commands**: Map old commands to new ECS commands
3. **Queries**: Use new query functions instead of repositories
4. **Events**: Subscribe to new event types
5. **Cross-Domain**: Use projections for person/org details 