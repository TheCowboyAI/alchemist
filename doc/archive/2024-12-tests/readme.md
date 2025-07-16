# Archived Tests (December 2024)

## Overview

This directory contains the complete test suite from before the GraphComposition architecture refactoring. These tests are considered **obsolete** but remain **useful as reference material** for:

1. **Test Patterns**: Examples of how we tested various components
2. **Domain Logic**: Business rules that were validated
3. **Integration Points**: How different parts of the system were tested together
4. **Edge Cases**: Specific scenarios and error conditions we handled

## Why Archived?

The previous test suite was built around the older architecture that didn't fully embrace:
- ContentGraph as the primary abstraction
- Lazy CID evaluation patterns
- Recursive graph composition
- Proper DDD naming conventions (e.g., RelationshipPredicate → RelatedBy)

## New Testing Strategy

Going forward, tests will be rebuilt incrementally following:
- **Test-Driven Development (TDD)**: Write tests first, then implementation
- **GraphComposition Focus**: All tests align with the new recursive graph architecture
- **Domain-First**: Tests express business intent, not technical implementation
- **Event-Driven**: Tests validate event flows and state transitions

## Contents

- `domain/`: Domain layer tests (aggregates, commands, events)
- `integration/`: Cross-layer integration tests
- Various test modules from the src directory

## Using These Tests

When implementing new features, you may find it helpful to:
1. Search for similar test cases in this archive
2. Extract the business logic being tested
3. Rewrite the test using the new GraphComposition patterns
4. Ensure the test aligns with current DDD conventions

Remember: These tests reflect the old architecture. Use them for inspiration, not direct copying.
