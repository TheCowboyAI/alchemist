# Plan: Fix CIM Domain vs CIM Compose Separation

## Problem Statement
- `cim-compose` duplicates core types like `Entity` and `EntityId` that should come from `cim-domain`
- Unclear separation of responsibilities between the two crates
- Risk of diverging implementations and confusion

## Solution Overview
1. Make `cim-compose` depend on `cim-domain` for core DDD types
2. Remove duplicate type definitions from `cim-compose`
3. Keep only graph-specific types in `cim-compose`
4. Update all imports and usages

## Implementation Steps

### Step 1: Update cim-compose Dependencies
- Add `cim-domain` as a dependency in `cim-compose/Cargo.toml`
- This establishes the proper dependency direction

### Step 2: Remove Duplicate Types from cim-compose
Remove from `cim-compose/src/base_types.rs`:
- `Entity<T>` struct (use from cim-domain)
- `EntityId<T>` struct (use from cim-domain)
- Marker types that duplicate cim-domain markers

Keep in `cim-compose/src/base_types.rs`:
- `NodeId` (graph-specific, not an entity)
- `EdgeId` (graph-specific, not an entity)
- `BaseNodeType` (graph-specific)
- `BaseRelationshipType` (graph-specific)
- `Relationship<T>` (graph-specific)
- `Metadata` (graph-specific)
- `DomainMapping` trait (for conversions)

### Step 3: Update Imports in cim-compose
- Import `Entity`, `EntityId`, and markers from `cim_domain`
- Update type aliases to use imported types
- Fix any compilation errors

### Step 4: Verify Domain Modules
Check that domain modules:
- Import core types from `cim-domain`
- Use `cim-compose` only for graph composition
- Don't have their own duplicate definitions

### Step 5: Add Integration Tests
Create tests that verify:
- Types from cim-domain can be composed using cim-compose
- No circular dependencies exist
- Clear separation is maintained

### Step 6: Update Documentation
- Update module-level documentation in both crates
- Add examples showing proper usage
- Document the architectural decision

## Expected Outcome
- Clear separation: cim-domain defines "what things are", cim-compose defines "how things combine"
- No duplicate type definitions
- Proper dependency direction (cim-compose depends on cim-domain, not vice versa)
- All tests pass
- Documentation reflects the clean architecture

## Risk Mitigation
- Run all tests after each step
- Check for breaking changes in domain modules
- Ensure backward compatibility where possible
