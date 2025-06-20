# Fix Compilation Errors Plan

## Date: January 12, 2025

## Current Status
- Branch: `feature/bounded-context-refactoring`
- 35 commits ahead of main
- Major blocker: Compilation errors in `cim-contextgraph` submodule

## Critical Compilation Errors

### cim-contextgraph Issues

1. **Missing imports in multiple files:**
   - `invariants.rs`: Cannot find `Node` and `Edge` in `crate::types`
   - `composition.rs`: Same missing imports
   - Need to check where these types are actually defined

2. **Missing serde attribute:**
   - Need to add `serde` derive macros or imports

3. **Module not found:**
   - `context_graph_v2` module is referenced but doesn't exist

4. **Thread safety issues:**
   - Generic parameters `N` and `E` need `Send + Sync` bounds

5. **Missing trait implementations:**
   - Need to implement `clone_box` and `type_name` methods

## Investigation Steps

1. **Check cim-contextgraph structure:**
   ```bash
   cd cim-contextgraph
   find . -name "*.rs" | xargs grep -l "struct Node\|struct Edge"
   ```

2. **Find where Node and Edge are defined:**
   - Check if they're in a different module
   - Check if they need to be imported from another crate

3. **Fix import paths:**
   - Update imports to point to correct modules
   - Add missing type definitions if needed

## Fix Strategy

### Option 1: Fix in cim-contextgraph directly
1. Checkout the cim-contextgraph repository
2. Fix all compilation errors
3. Run tests to ensure nothing breaks
4. Push fixes to GitHub
5. Update submodule reference in main repo

### Option 2: Temporary workaround
1. Pin cim-contextgraph to a working commit
2. Document the issues for later fix
3. Proceed with merge if other functionality works

### Option 3: Remove problematic features
1. Comment out or remove the problematic modules temporarily
2. Create issues to track the needed fixes
3. Merge with reduced functionality

## Recommended Approach

Given that this is blocking the merge to main, I recommend:

1. **Immediate fix in cim-contextgraph:**
   - The errors seem straightforward (missing imports/types)
   - Fix directly in the submodule
   - This maintains full functionality

2. **Steps:**
   ```bash
   # 1. Go to cim-contextgraph
   cd cim-contextgraph

   # 2. Find where Node and Edge are defined
   grep -r "pub struct Node" .
   grep -r "pub struct Edge" .

   # 3. Fix imports in affected files
   # 4. Add missing trait implementations
   # 5. Test compilation
   cargo check

   # 6. Commit and push fixes
   git add -A
   git commit -m "fix: Resolve import errors and add missing trait implementations"
   git push origin main

   # 7. Update submodule in main repo
   cd ..
   git add cim-contextgraph
   git commit -m "chore: Update cim-contextgraph with compilation fixes"
   ```

## Success Criteria

- [ ] `cargo check` passes with no errors
- [ ] `cargo test` runs successfully
- [ ] All submodules are at stable commits
- [ ] Branch can be merged to main without breaking the build

## Timeline

- Estimated time to fix: 1-2 hours
- Most time will be spent on:
  - Finding correct import paths
  - Implementing missing traits
  - Running tests to ensure no regressions
