# Documentation Cleanup Summary - June 28, 2025

## Overview
Major cleanup of `/doc` folder structure to ensure only current, actively-used documentation remains in primary locations.

## Changes Made

### 1. Design Folder Restructuring
- Moved completed/outdated designs from `/doc/design` to this archive
- Eliminated the `current/` subfolder - all current designs now at top level
- Updated `readme.md` to reflect only active design documents

### 2. Archived Documents
The following categories of documents were archived:

#### Completed Implementations
- `graph-abstraction-layer.md` - Fully implemented in 4 phases
- `identity-domain-*.md` - Identity domain refactoring complete
- `cim-bridge-*.md` - Bridge implementation complete
- `dialog-domain-design.md` - Dialog domain implemented

#### Workflow Documentation
- All `workflow_*.md` files from main doc folder
- These documented the workflow domain implementation progress

#### Infrastructure/Security Designs
- `correlation-causation-*.md` - Implemented in cim-subject
- `event-correlation-*.md` - Event correlation complete
- `authentication-composition-example.md` - Authentication patterns implemented
- `policy-authentication-composition.md` - Policy domain complete

## Result
- `/doc/design` now contains only current, active design documents
- Historical context preserved in archive for reference
- Clear separation between active and completed work

## Naming Convention
All files renamed to lowercase following project conventions (no more UPPERCASE filenames).

## Archive Location
`/doc/archive/2025-06-28-design-cleanup/` 