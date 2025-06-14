# Extract Domain Modules to Separate Submodules

## Overview
Extract specific domain implementations from `cim-domain` into their own submodules as they represent separate bounded contexts.

## Domains to Extract

1. **cim-domain-person** - Person/People domain
   - Source: `cim-domain/src/person.rs`
   - Related events, commands, and handlers

2. **cim-domain-organization** - Organization domain
   - Source: `cim-domain/src/organization.rs`
   - Related events, commands, and handlers

3. **cim-domain-agent** - Agent domain
   - Source: `cim-domain/src/agent.rs`
   - Related events, commands, and handlers

4. **cim-domain-policy** - Policy domain
   - Source: `cim-domain/src/policy.rs`
   - Related events, commands, and handlers

5. **cim-domain-workflow** - Workflow domain
   - Source: `cim-domain/src/workflow/`
   - All workflow-related modules

6. **cim-domain-document** - Document domain
   - Source: `cim-domain/src/document.rs`
   - Related events, commands, and handlers

## What Remains in cim-domain

After extraction, `cim-domain` should only contain:
- Core domain abstractions
- Shared domain types
- Common domain infrastructure
- Domain graph visualization
- Concept graph abstractions
- Base entity/aggregate patterns
- Shared identifiers and errors

## Extraction Process

For each domain:

1. Create new repository: `cim-domain-<name>`
2. Create basic Cargo.toml with dependencies
3. Extract domain module and related code
4. Extract related commands from `commands.rs`
5. Extract related events from `domain_events.rs`
6. Extract related handlers from `command_handlers.rs` and `query_handlers.rs`
7. Update imports and dependencies
8. Add as submodule to main project
9. Update cim-domain to remove extracted code

## Dependencies

Each extracted domain will depend on:
- `cim-domain` (for core abstractions)
- `cim-core-domain` (for base types)
- `cim-infrastructure` (for NATS/event store)
- Standard dependencies (serde, tokio, etc.)

## Benefits

1. **Clear Bounded Contexts** - Each domain is truly separate
2. **Independent Evolution** - Domains can evolve independently
3. **Better Modularity** - Clear dependencies between domains
4. **Easier Testing** - Test each domain in isolation
5. **Team Ownership** - Different teams can own different domains

## Implementation Order

1. Start with `cim-domain-person` as it's relatively self-contained
2. Then `cim-domain-organization`
3. Then `cim-domain-agent`
4. Then `cim-domain-policy`
5. Then `cim-domain-document`
6. Finally `cim-domain-workflow` (most complex)

## Post-Extraction Tasks

1. Update main `ia` Cargo.toml to reference new submodules
2. Update integration tests
3. Update documentation
4. Verify all builds pass
5. Update CI/CD pipelines
