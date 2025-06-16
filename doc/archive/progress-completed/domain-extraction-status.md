# Domain Extraction Status

## Overview
Extracting domain modules from `cim-domain` into separate bounded context submodules.

## Completed Tasks

### 1. Planning
- ✅ Created comprehensive extraction plan (`doc/plan/extract-domain-submodules.md`)
- ✅ Identified 6 domains to extract:
  - Person/People
  - Organization
  - Agent
  - Policy
  - Document
  - Workflow

### 2. GitHub Repository Creation
- ✅ Created Nix script for automated repository creation
- ✅ Successfully created all 6 repositories:
  - https://github.com/TheCowboyAI/cim-domain-person
  - https://github.com/TheCowboyAI/cim-domain-organization
  - https://github.com/TheCowboyAI/cim-domain-agent
  - https://github.com/TheCowboyAI/cim-domain-policy
  - https://github.com/TheCowboyAI/cim-domain-document
  - https://github.com/TheCowboyAI/cim-domain-workflow

### 3. Extraction Scripts
- ✅ Created general extraction script (`scripts/extract-domain-submodules.sh`)
- ✅ Created specific person domain extraction script (`scripts/extract-person-domain.sh`)
- ✅ Created batch extraction script (`scripts/batch-extract-domains.sh`)
- ✅ Created GitHub repo creation script (`scripts/create-domain-repos.nix`)

## Next Steps

### 1. Extract Domain Code
For each domain, we need to:

1. **Extract the aggregate module**
   - Copy main domain file (e.g., `person.rs` → `aggregate/mod.rs`)
   - Update module structure and imports

2. **Extract related commands**
   - Find all commands in `commands.rs` related to the domain
   - Move to `commands/mod.rs` in the new module

3. **Extract related events**
   - Find all events in `domain_events.rs` related to the domain
   - Move to `events/mod.rs` in the new module

4. **Extract command handlers**
   - Find handlers in `command_handlers.rs` for the domain
   - Move to `handlers/command_handlers.rs`

5. **Extract query handlers**
   - Find handlers in `query_handlers.rs` for the domain
   - Move to `handlers/query_handlers.rs`

6. **Extract projections**
   - Find projections in `projections/` for the domain
   - Move to `projections/mod.rs`

7. **Create value objects**
   - Identify and extract domain-specific value objects
   - Move to `value_objects/mod.rs`

### 2. Update Dependencies

Each extracted domain will need:
- Proper `Cargo.toml` dependencies
- Import statements updated
- Tests added

### 3. Add as Submodules

After extraction and testing:
```bash
git submodule add https://github.com/TheCowboyAI/cim-domain-person.git cim-domain-person
git submodule add https://github.com/TheCowboyAI/cim-domain-organization.git cim-domain-organization
git submodule add https://github.com/TheCowboyAI/cim-domain-agent.git cim-domain-agent
git submodule add https://github.com/TheCowboyAI/cim-domain-policy.git cim-domain-policy
git submodule add https://github.com/TheCowboyAI/cim-domain-document.git cim-domain-document
git submodule add https://github.com/TheCowboyAI/cim-domain-workflow.git cim-domain-workflow
```

### 4. Clean up cim-domain

After successful extraction:
- Remove extracted domain files
- Keep only core abstractions
- Update `lib.rs` exports
- Update documentation

### 5. Update Main Project

- Update root `Cargo.toml` to reference new submodules
- Update integration tests
- Update CI/CD pipelines
- Update documentation

## Manual Extraction Required

Due to the interconnected nature of the code, manual extraction is needed to:
- Properly separate domain-specific code
- Maintain correct dependencies
- Ensure no breaking changes
- Add appropriate tests

## Benefits Once Complete

1. **Clear Bounded Contexts** - Each domain truly separate
2. **Independent Evolution** - Domains can evolve independently
3. **Better Modularity** - Clear dependencies between domains
4. **Easier Testing** - Test each domain in isolation
5. **Team Ownership** - Different teams can own different domains
