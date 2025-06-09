# Test Migration Status

## Overview

All existing tests have been archived to `doc/archive/2024-12-tests/` as they were built around the previous architecture. We are rebuilding the test suite from scratch to align with the GraphComposition architecture.

## Migration Strategy

### Phase 1: Core Domain Tests (Starting Now)
- [ ] ContentGraph aggregate tests
- [ ] GraphComposition recursive structure tests
- [ ] Lazy CID evaluation tests
- [ ] Command handling tests
- [ ] Event generation tests

### Phase 2: Integration Tests
- [ ] NATS event bridge tests
- [ ] Bevy system integration tests
- [ ] CID/IPLD storage tests
- [ ] Cross-aggregate communication tests

### Phase 3: End-to-End Tests
- [ ] Complete workflow scenarios
- [ ] Graph manipulation flows
- [ ] Pattern detection and analysis
- [ ] Performance benchmarks

## Test Design Principles

1. **Test-First Development**: Write failing tests before implementation
2. **Domain Focus**: Tests express business intent, not technical details
3. **Event Verification**: Validate state changes through events
4. **Mermaid Documentation**: Each test includes a graph showing what's being tested
5. **No Rendering Tests**: Focus on data and behavior, not visual output

## Archived Test Reference

The archived tests contain valuable patterns and edge cases. When implementing new features:
1. Check archived tests for similar scenarios
2. Extract the business logic being tested
3. Rewrite using GraphComposition patterns
4. Ensure proper DDD naming conventions

## Current Status

- ✅ All old tests archived
- ✅ Clean test directory created
- 🔄 Ready to begin TDD with GraphComposition
- ⏳ No tests written yet (intentional - will add as we implement features)
