# Immediate Actions Required - CIM Project

## 🚨 Critical Issues to Fix

### 1. Broken Examples (Priority 1)
These examples need immediate fixing as they serve as documentation:

- [ ] Fix `cim-domain/examples/command_handler_example.rs`
  - Add missing imports for LocationType, Address, PolicyCommandHandler
  - Update to current API

- [ ] Fix `cim-domain/examples/demos/cqrs_pattern_demo.rs`
  - Remove references to non-existent domain events
  - Update event enum variants

- [ ] Fix `cim-domain/examples/event_stream_example.rs`
  - Change field access to method calls (.event_type(), .timestamp())

- [ ] Fix `cim-keys/examples/cim_leaf_integration.rs`
  - Remove OathType/OathAlgorithm references or add them back

- [ ] Fix `cim-keys/examples/basic_usage.rs`
  - Update KeyUsage API usage

- [ ] Fix `cim-keys/examples/nats_tls_setup.rs`
  - Add or fix export_key method

### 2. Add Missing Tests (Priority 2)

Modules with 0 tests that need coverage:

- [ ] `cim-domain-conceptualspaces` - Add at least 5 basic tests
- [ ] `cim-domain-person` - Add aggregate and command handler tests
- [ ] `cim-infrastructure` - Add NATS bridge tests
- [ ] `cim-keys` - Add key management tests
- [ ] `cim-component` - Add component registration tests

### 3. Code Cleanup (Priority 3)

- [ ] Run `cargo clippy --all --fix` to auto-fix warnings
- [ ] Remove unused imports in:
  - cim-domain-document
  - cim-domain-workflow
  - cim-keys
  
- [ ] Complete or remove stub implementations
- [ ] Add #[allow(dead_code)] or remove unused functions

### 4. Documentation Updates (Priority 4)

- [ ] Create MIGRATION.md for API changes
- [ ] Update README with current examples
- [ ] Document breaking changes in CHANGELOG.md

## Quick Wins (Can do immediately)

1. **Auto-fix clippy warnings**:
   ```bash
   cargo clippy --all --fix --allow-dirty --allow-staged
   ```

2. **Remove unused imports**:
   ```bash
   cargo fix --allow-dirty
   ```

3. **Format all code**:
   ```bash
   cargo fmt --all
   ```

## Validation Checklist

After fixes, ensure:
- [ ] `cargo test --all` passes with 0 failures
- [ ] `cargo test --all --examples` compiles without errors
- [ ] `cargo clippy --all` shows no warnings
- [ ] All domains have at least 3 tests

## Time Estimate

- Fixing examples: 4-6 hours
- Adding missing tests: 8-12 hours  
- Code cleanup: 2-4 hours
- Documentation: 4-6 hours

**Total: 18-28 hours of work** 