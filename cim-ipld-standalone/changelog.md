# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **Canonical Payload Support**: Added `canonical_payload()` method to `TypedContent` trait
  - Allows content types to define which fields are included in CID calculation
  - Excludes transient metadata (timestamps, UUIDs, correlation IDs) from CIDs
  - Ensures identical content always produces the same CID
  - Default implementation uses full serialization for backward compatibility
  - See `examples/event_cid_example.rs` for usage patterns
  - See `docs/CID_CALCULATION_GUIDE.md` for detailed documentation

### Changed
- `TypedContent::calculate_cid()` now uses `canonical_payload()` instead of `to_bytes()`
  - This is a breaking change if you rely on CIDs including all fields
  - To maintain old behavior, don't override `canonical_payload()`

### Documentation
- Added comprehensive CID calculation guide (`docs/CID_CALCULATION_GUIDE.md`)
- Added example demonstrating canonical payload patterns (`examples/event_cid_example.rs`)
- Added tests for canonical payload functionality (`tests/canonical_payload_tests.rs`)

## [0.1.0] - Initial Release

- Initial implementation of CIM-IPLD
- Content-addressed storage with NATS Object Store
- CID chain support for event sourcing
- TypedContent trait for type-safe content storage
