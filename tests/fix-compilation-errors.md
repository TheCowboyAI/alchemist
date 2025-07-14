# Test Compilation Error Fixes

## Summary of Common Compilation Errors and Their Fixes

### 1. Bevy Feature Flag Issues

**Error Pattern**:
```
error[E0432]: unresolved imports `alchemist::graph_components`, `alchemist::graph_algorithms`, `alchemist::jetstream_persistence`
```

**Fix**: Add feature gates to tests that require Bevy:
```rust
// At the top of the file for imports
#[cfg(feature = "bevy")]
use alchemist::{
    graph_components::*,
    graph_algorithms::*,
    jetstream_persistence::*,
};

// For individual tests
#[cfg(feature = "bevy")]
#[tokio::test]
async fn test_graph_persistence_events() -> Result<()> {
    // test code
}
```

**Affected Files**:
- `graph_integration_test.rs` - Fixed ✓
- `test_jetstream_persistence.rs` - Fixed ✓
- Any test importing Bevy-dependent modules

### 2. Field Name Changes

**Error Pattern**:
```
error[E0609]: no field `source` on type `EdgeData`
error[E0609]: no field `target` on type `EdgeData`
```

**Fix**: Update field names:
```rust
// Old
assert_eq!(edges[0].source, "n1");
assert_eq!(edges[0].target, "n2");

// New
assert_eq!(edges[0].source_id, "n1");
assert_eq!(edges[0].target_id, "n2");
```

**Affected Files**:
- `graph_integration_test.rs` - Fixed ✓

### 3. AiModelConfig Structure Changes

**Error Pattern**:
```
error[E0560]: struct `AiModelConfig` has no field named `name`
error[E0560]: struct `AiModelConfig` has no field named `api_key`
error[E0560]: struct `AiModelConfig` has no field named `parameters`
```

**Fix**: Update to new structure:
```rust
// Old
config.ai_models.push(AiModelConfig {
    name: "gpt-4".to_string(),
    provider: "openai".to_string(),
    api_key: Some(api_key),
    endpoint: None,
    parameters: None,
});

// New
config.ai_models.insert("gpt-4".to_string(), AiModelConfig {
    provider: "openai".to_string(),
    endpoint: None,
    api_key_env: Some("OPENAI_API_KEY".to_string()),
    model_name: "gpt-4-turbo-preview".to_string(),
    max_tokens: Some(4096),
    temperature: Some(0.7),
    timeout_seconds: Some(30),
    rate_limit: None,
    fallback_model: None,
    params: std::collections::HashMap::new(),
});
```

**Affected Files**:
- `test_ai_real_api.rs` - Fixed ✓

### 4. Unresolved Imports

**Error Pattern**:
```
error[E0432]: unresolved imports `alchemist::shell::create_shell`
```

**Fix**: Remove non-existent imports:
```rust
// Old
use alchemist::shell::{AlchemistShell, create_shell};

// New
use alchemist::shell::AlchemistShell;
```

**Affected Files**:
- `shell_command_tests.rs` - Fixed ✓

### 5. HashMap Method Confusion

**Error Pattern**:
```
error[E0599]: no method named `push` found for struct `HashMap`
```

**Fix**: Use `insert` instead of `push`:
```rust
// Old
config.ai_models.push(model_config);

// New
config.ai_models.insert("model_name".to_string(), model_config);
```

## How to Run Tests

### Without Bevy Features
```bash
cargo test --test basic_integration_test
cargo test --test simple_passing_test
cargo test --test ai_model_tests
```

### With Bevy Features
```bash
cargo test --test graph_integration_test --features bevy
cargo test --test test_jetstream_persistence --features bevy
```

### Run All Tests
```bash
# Run non-Bevy tests
cargo test --tests

# Run Bevy tests
cargo test --tests --features bevy
```

## Next Steps

1. Apply similar fixes to remaining test files with compilation errors
2. Consider adding a test configuration in Cargo.toml to automatically enable features for specific tests
3. Update CI/CD pipeline to run tests with appropriate feature flags
4. Document the test structure and feature requirements in the main README