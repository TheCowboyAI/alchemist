# Test Compilation Errors Summary

## Common Compilation Issues in Alchemist Tests

### 1. Bevy Feature Flag Issues

**Problem**: Several tests import modules that require the `bevy` feature flag but don't check for it.

**Affected Files**:
- `graph_integration_test.rs` - imports `graph_components`, `graph_algorithms`, `jetstream_persistence`
- `test_jetstream_persistence.rs` - imports `jetstream_persistence`
- `comprehensive_user_story_tests.rs` - tests graph components and algorithms

**Solution**: These modules should either:
- Be conditionally compiled with `#[cfg(feature = "bevy")]`
- Have the tests run with `--features bevy`
- Be refactored to not depend on Bevy

### 2. Field Name Changes in Data Structures

**Problem**: Tests are using old field names that have been renamed.

**Examples**:
- `EdgeData` now uses `source_id`/`target_id` instead of `source`/`target`
- `AiModelConfig` structure has changed - no longer has `name`, `api_key`, `parameters` fields

**Affected Files**:
- `graph_integration_test.rs` - lines 89, 90, 203, 204
- Multiple AI-related tests using old `AiModelConfig` structure

### 3. Unresolved Imports

**Problem**: Tests are importing modules that don't exist or have been moved.

**Examples**:
- `ia::simple_agent` module not found
- `alchemist::shell::create_shell` doesn't exist
- `alchemist::ai::AiProvider` and `ModelConfig` not found

### 4. Changed API Methods

**Problem**: Some methods have been renamed or removed.

**Examples**:
- `HashMap` doesn't have `push` method (should use `insert`)
- `GeneralConfig` no longer has `home_dir` field

### 5. Missing Type Imports

**Problem**: Bevy types being used without proper imports or feature flags.

**Examples**:
- `EventReader`, `ResMut` from Bevy
- `MinimalPlugins`, `Update`, `App` from Bevy

## Recommended Fixes

1. **Add feature gates to tests that require Bevy**:
   ```rust
   #[cfg(feature = "bevy")]
   #[test]
   fn test_bevy_functionality() { ... }
   ```

2. **Update field names in tests**:
   - Change `edge.source` to `edge.source_id`
   - Change `edge.target` to `edge.target_id`

3. **Fix HashMap usage**:
   - Change `map.push(value)` to `map.insert(key, value)`

4. **Update imports to match current module structure**

5. **Add a test configuration that enables necessary features**:
   ```toml
   [[test]]
   name = "bevy_tests"
   required-features = ["bevy"]
   ```