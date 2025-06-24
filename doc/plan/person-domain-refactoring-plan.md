# Person Domain Refactoring Plan

## Executive Summary

The cim-domain-person module currently contains duplicated infrastructure code and an overly complex aggregate with 890 lines. This refactoring will:

1. Remove duplicated event store infrastructure (use shared cim-domain infrastructure)
2. Simplify the Person aggregate by extracting concerns
3. Focus the domain on its single responsibility: managing person information

## Current Issues

### 1. Duplicated Infrastructure
- `event_store.rs` (315 lines) duplicates functionality already in `cim-domain/src/infrastructure/`
- Custom `PersonEventStore` trait when generic `EventStore` exists
- Custom `InMemoryEventStore` implementation

### 2. Overly Complex Aggregate
- `person.rs` has 890 lines - violates single responsibility
- Too many concerns in one aggregate:
  - Contact management (emails, phones, addresses)
  - Employment history
  - Skills and education
  - Relationships
  - Social media profiles
  - Customer segmentation
  - Behavioral data
  - Privacy preferences

### 3. Missing Domain Services
- Complex operations embedded in aggregate
- No clear separation of concerns
- Difficult to test individual aspects

## Refactoring Steps

### Phase 1: Remove Infrastructure Duplication

1. **Delete `event_store.rs`**
   - Remove the entire file
   - Update imports to use `cim_domain::infrastructure::{EventStore, EventRepository}`

2. **Update Cargo.toml**
   ```toml
   [dependencies]
   cim-domain = { path = "../cim-domain" }
   ```

3. **Update tests to use shared infrastructure**

### Phase 2: Extract Sub-Aggregates

Create focused sub-aggregates for different concerns:

1. **ContactInfo Aggregate**
   - Emails, phones, addresses
   - Own events: ContactAdded, ContactRemoved, ContactVerified
   - Reduces Person aggregate by ~200 lines

2. **ProfessionalProfile Aggregate**
   - Employment history
   - Skills and certifications
   - Education
   - Own events: EmploymentAdded, SkillAdded, etc.
   - Reduces Person aggregate by ~250 lines

3. **SocialProfile Aggregate**
   - Social media accounts
   - Customer segmentation
   - Behavioral data
   - Own events: ProfileAdded, SegmentChanged, etc.
   - Reduces Person aggregate by ~150 lines

### Phase 3: Simplify Person Aggregate

After extraction, Person aggregate will focus on:
- Core identity (name, active status)
- Relationships to other aggregates
- Merge operations
- ~300 lines (down from 890)

### Phase 4: Create Domain Services

1. **PersonMergeService**
   - Handle complex merge logic
   - Coordinate between aggregates

2. **PersonQueryService**
   - Complex queries across sub-aggregates
   - Projections for read models

## New Structure

```
cim-domain-person/
├── src/
│   ├── lib.rs
│   ├── aggregate/
│   │   ├── mod.rs
│   │   ├── person.rs (~300 lines)
│   │   ├── contact_info.rs (~200 lines)
│   │   ├── professional_profile.rs (~250 lines)
│   │   └── social_profile.rs (~150 lines)
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── person.rs
│   │   ├── contact.rs
│   │   ├── professional.rs
│   │   └── social.rs
│   ├── events/
│   │   ├── mod.rs
│   │   ├── person.rs
│   │   ├── contact.rs
│   │   ├── professional.rs
│   │   └── social.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── merge_service.rs
│   │   └── query_service.rs
│   ├── handlers/
│   │   └── ... (command handlers for each aggregate)
│   └── value_objects/
│       └── ... (unchanged)
```

## Benefits

1. **Reduced Duplication**: Remove 315 lines of infrastructure code
2. **Better Separation**: Each aggregate has single responsibility
3. **Improved Testability**: Can test each concern independently
4. **Easier Maintenance**: Smaller, focused files
5. **Better Performance**: Can load only needed aggregates

## Migration Strategy

1. Create new structure alongside existing
2. Migrate tests incrementally
3. Update dependent code
4. Remove old structure

## Success Criteria

- [ ] No infrastructure duplication
- [ ] Person aggregate < 400 lines
- [ ] All tests passing
- [ ] Clear separation of concerns
- [ ] Each aggregate follows single responsibility principle

## Timeline

- Phase 1: 2 hours (remove duplication)
- Phase 2: 4 hours (extract sub-aggregates)
- Phase 3: 2 hours (simplify person aggregate)
- Phase 4: 2 hours (create services)
- Testing: 2 hours

Total: ~12 hours 