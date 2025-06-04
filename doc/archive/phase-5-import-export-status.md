# Phase 5: Import/Export - Status Update

## Overview

Phase 5 was intended to implement full import/export capabilities for the graph editor. This document provides an accurate assessment of what has been implemented versus what is still missing.

## Phase 5 Requirements (From Plan)

According to the incremental implementation plan, Phase 5 should include:

1. **JSON Serialization Service** (`SerializeGraphToJson`)
   - Export graph to JSON file
   - Preserve all node and edge data
   - Maintain graph metadata

2. **JSON Deserialization Service** (`DeserializeGraphFromJson`)
   - Import JSON to create graph
   - Round-trip data preservation
   - Error handling for invalid data

## Current Implementation Status

### ✅ What's Implemented

#### 1. JSON Import (Partial) ✅
- **File**: `src/contexts/graph_management/importer.rs`
- **Functionality**:
  - `GraphImporter::load_from_file()` - Loads graphs from JSON files
  - `import_graph_from_file()` system - Handles Ctrl+O keyboard shortcut
  - Supports external graph format (nodes, relationships, properties)
  - Creates graph entities in ECS
  - Syncs with Daggy storage
  - Clears existing graphs before import
  - Hardcoded to load `assets/models/CIM.json`

#### 2. External Format Support ✅
- Deserializes from specific JSON schema:
  ```rust
  pub struct ExternalGraphFormat {
      pub nodes: Vec<ExternalNode>,
      pub relationships: Vec<ExternalRelationship>,
      pub style: Option<serde_json::Value>,
  }
  ```
- Maps external IDs to internal `NodeIdentity` and `EdgeIdentity`
- Scales and transforms coordinates appropriately

### ❌ What's Missing

#### 1. JSON Export (Not Implemented) ❌
- No `SerializeGraphToJson` service
- No export functionality whatsoever
- Cannot save graphs to files
- No keyboard shortcut for export (e.g., Ctrl+S)

#### 2. Round-Trip Capability ❌
- Cannot export and re-import the same graph
- No preservation of internal graph format
- No way to save work in progress

#### 3. File Dialog Integration ❌
- Import hardcoded to specific file
- No file browser/picker
- No way to choose import location
- No way to specify export location

#### 4. Multiple Format Support ❌
- Only supports one specific JSON schema
- No support for other formats (GraphML, DOT, etc.)
- No format conversion capabilities

#### 5. Error Handling UI ❌
- Errors only logged to console
- No user feedback for import failures
- No validation before import

## Technical Debt

### Import Limitations
1. **Hardcoded File Path**: Always loads `assets/models/CIM.json`
2. **No File Dialog**: Cannot choose different files
3. **Destructive Import**: Clears all existing graphs
4. **No Merge Capability**: Cannot import into existing graph

### Missing Export Architecture
1. **No Serialization Service**: Need to implement `SerializeGraphToJson`
2. **No Export System**: Need system to handle Ctrl+S or export command
3. **No Format Definition**: Need to define internal JSON schema
4. **No File Writing**: Need file system integration

## Implementation Gap Analysis

| Feature | Planned | Implemented | Gap |
|---------|---------|-------------|-----|
| Import from JSON | ✅ | ✅ | File dialog missing |
| Export to JSON | ✅ | ❌ | Completely missing |
| Round-trip preservation | ✅ | ❌ | No export = no round-trip |
| Multiple file support | ✅ | ❌ | Hardcoded path |
| Error handling | ✅ | ⚠️ | Console only |
| Format validation | ✅ | ⚠️ | Basic only |

## Required Work to Complete Phase 5

### 1. Implement Export Service
```rust
// In src/contexts/graph_management/exporter.rs
pub struct GraphExporter;

impl GraphExporter {
    pub fn export_to_json(
        graph_id: GraphIdentity,
        storage: &GraphStorage,
        nodes: Query<(&NodeIdentity, &NodeContent, &SpatialPosition)>,
        edges: Query<(&EdgeIdentity, &EdgeRelationship)>,
    ) -> Result<String, ExportError> {
        // Implementation needed
    }
}
```

### 2. Add Export System
```rust
pub fn export_graph_to_file(
    keyboard: Res<ButtonInput<KeyCode>>,
    storage: Res<GraphStorage>,
    active_graph: Query<&GraphIdentity, With<ActiveGraph>>,
    // ... other queries
) {
    if keyboard.just_pressed(KeyCode::KeyS) && keyboard.pressed(KeyCode::ControlLeft) {
        // Export logic
    }
}
```

### 3. Define Internal JSON Schema
```rust
#[derive(Serialize, Deserialize)]
pub struct InternalGraphFormat {
    pub version: String,
    pub graph: GraphMetadata,
    pub nodes: Vec<InternalNode>,
    pub edges: Vec<InternalEdge>,
}
```

### 4. Implement File Dialog
- Use `rfd` crate or similar for native file dialogs
- Allow user to choose import/export locations
- Support multiple file formats

## Recommendation

Phase 5 is **INCOMPLETE**. Only import functionality exists, and even that is limited. To truly complete Phase 5:

1. **Priority 1**: Implement basic JSON export
2. **Priority 2**: Add file dialog for import/export
3. **Priority 3**: Ensure round-trip data preservation
4. **Priority 4**: Add support for multiple formats

## Estimated Effort

- **Export Implementation**: 2-3 days
- **File Dialog Integration**: 1 day
- **Round-trip Testing**: 1 day
- **Multiple Format Support**: 2-3 days

**Total**: 1-2 weeks to fully complete Phase 5

---

*Status*: PARTIALLY COMPLETE (Import only)
*Last Updated*: December 2024
