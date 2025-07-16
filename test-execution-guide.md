# Test Execution Guide

## Quick Start

To run tests for the working domains:

```bash
./run_working_tests.sh
```

## Domain-Specific Tests

### Core Domains (Fast, No Bevy)
```bash
# Conceptual Spaces (27 tests)
cargo test --package cim-domain-conceptualspaces --lib

# Workflow (38 tests)
cargo test --package cim-domain-workflow --lib

# Document (5 tests)
cargo test --package cim-domain-document --lib

# Location (29 tests)
cargo test --package cim-domain-location --lib

# Dialog
cargo test --package cim-domain-dialog --lib

# Nix
cargo test --package cim-domain-nix --lib
```

### Infrastructure Tests
```bash
# Core domain functionality
cargo test --package cim-domain --lib

# IPLD and content addressing
cargo test --package cim-ipld --lib
```

## Performance Tests

Run specific performance benchmarks:

```bash
# Event creation performance
cargo test --package cim-domain test_event_creation_performance

# CID chain performance
cargo test --package cim-ipld test_cid_chain_performance
```

## Integration Tests

For integration tests (may take longer to compile):

```bash
# Cross-domain integration
cargo test cross_domain_integration_test

# Error handling
cargo test error_handling_test

# Performance benchmarks
cargo test performance_benchmark_test
```

## Troubleshooting

### Compilation Timeout
If tests timeout during compilation:

1. Run individual package tests instead of the full suite
2. Use `--no-default-features` to reduce dependencies
3. Set `CARGO_BUILD_JOBS=1` to reduce memory usage

### Example:
```bash
CARGO_BUILD_JOBS=1 cargo test --package cim-domain-workflow --lib
```

### Check Specific Functionality

To verify specific fixes:

```bash
# Check GraphType fix
grep -r "GraphType::General" tests/

# Check ContentType fix  
grep -r "ContentType::Text" examples/

# Check Bevy conversion
grep -r "bevy-patched" . --include="*.toml"
```

## Test Categories by Compilation Time

### Fast (<10s)
- cim-domain-conceptualspaces
- cim-domain-workflow
- cim-domain-document
- cim-domain-location

### Medium (10-30s)
- cim-domain
- cim-ipld
- cim-domain-dialog

### Slow (>30s)
- Integration tests
- UI tests
- Full test suite

## Recommended Test Order

1. Run `./run_working_tests.sh` first
2. If all pass, try individual integration tests
3. Finally, attempt full test suite if needed

## Current Status

- ✅ Core domains compile and pass tests
- ✅ Performance metrics excellent
- ⚠️ Full compilation slow due to dependencies
- ⚠️ UI/Bevy tests need verification after conversion