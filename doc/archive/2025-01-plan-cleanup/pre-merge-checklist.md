# Pre-Merge Checklist for feature/bounded-context-refactoring

## Date: January 12, 2025

## Critical Issues to Resolve

### 🔴 Compilation Errors

#### cim-contextgraph
- [ ] Fix missing imports: `crate::types::Node` and `crate::types::Edge`
- [ ] Add missing `serde` derive macro or import
- [ ] Fix module path: `context_graph_v2` not found in crate root
- [ ] Add thread safety bounds (`Send + Sync`) to generic parameters N and E
- [ ] Implement missing trait items: `clone_box` and `type_name`

#### cim-domain
- [ ] Remove unused import `GraphId` from command_handlers.rs

### 🟡 Build Verification
- [ ] Run `nix develop -c cargo check` - must pass with no errors
- [ ] Run `nix develop -c cargo clippy` - address any warnings
- [ ] Run `nix develop -c cargo test` - all tests must pass
- [ ] Run `nix build` - ensure the final build succeeds

### 🟢 Submodule Verification
- [ ] Verify all submodules are at correct commits
- [ ] Ensure `.gitmodules` is properly configured
- [ ] Test fresh clone with submodule initialization
- [ ] Verify all submodule dependencies are resolved

### 📋 Code Quality Checks
- [ ] Review all 35 commits for consistency
- [ ] Ensure documentation is up to date
- [ ] Verify no sensitive information in commits
- [ ] Check for proper error handling
- [ ] Ensure consistent naming conventions

### 🔄 Integration Testing
- [ ] Test main application functionality
- [ ] Verify NATS integration works
- [ ] Check Bevy visualization components
- [ ] Test event sourcing functionality

### 📝 Documentation Updates
- [ ] Update README with submodule instructions
- [ ] Document any breaking changes
- [ ] Update development setup guide
- [ ] Add migration notes if needed

## Merge Strategy

### Option 1: Fix on Current Branch
1. Fix all compilation errors
2. Run full test suite
3. Squash commits if needed
4. Create PR for review

### Option 2: Create Fix Branch
1. Create new branch from current: `fix/pre-merge-issues`
2. Fix issues incrementally with clear commits
3. Merge back to feature branch
4. Then merge to main

### Option 3: Interactive Rebase
1. Use `git rebase -i` to clean up commit history
2. Squash submodule conversion commits
3. Fix issues in appropriate commits
4. Force push cleaned branch

## Commands to Run

```bash
# Check current issues
nix develop -c cargo check

# Fix warnings automatically
nix develop -c cargo fix --lib -p cim-domain

# Run full test suite
nix develop -c cargo test

# Check for clippy warnings
nix develop -c cargo clippy -- -D warnings

# Build the project
nix build
```

## Next Steps
1. Decide on merge strategy
2. Fix compilation errors first
3. Run full test suite
4. Clean up commit history if needed
5. Create PR or merge directly based on team workflow
