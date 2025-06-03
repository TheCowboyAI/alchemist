# Phase 5: Export Completion Plan

## Overview

This plan outlines the implementation steps needed to complete Phase 5 by adding the missing export functionality. Currently, only import is implemented.

## Current State

- ✅ Import from JSON (hardcoded to `assets/models/CIM.json`)
- ✅ External format deserialization
- ✅ Graph creation from imported data
- ❌ Export to JSON
- ❌ File dialog integration
- ❌ Round-trip data preservation
- ❌ Multiple format support

## Implementation Plan

### Step 1: Create Export Service (Day 1-2)

**File**: `src/contexts/graph_management/exporter.rs`

```rust
use crate::contexts::graph_management::domain::*;
use crate::contexts::graph_management::storage::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Internal graph format for export
#[derive(Debug, Serialize, Deserialize)]
pub struct InternalGraphFormat {
    pub version: String,
    pub metadata: GraphMetadata,
    pub nodes: Vec<InternalNode>,
    pub edges: Vec<InternalEdge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalNode {
    pub id: String,
    pub position: Position3D,
    pub content: NodeContent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relationship: EdgeRelationship,
}

/// Service to export graphs
pub struct GraphExporter;

impl GraphExporter {
    pub fn export_to_json(
        graph_id: GraphIdentity,
        storage: &GraphStorage,
        nodes: &Query<(&NodeIdentity, &NodeContent, &SpatialPosition)>,
        edges: &Query<(&EdgeIdentity, &EdgeRelationship)>,
        graphs: &Query<&GraphMetadata>,
    ) -> Result<String, ExportError> {
        // Implementation
    }

    pub fn save_to_file(
        path: &Path,
        json_content: &str,
    ) -> Result<(), std::io::Error> {
        fs::write(path, json_content)
    }
}

#[derive(Debug)]
pub enum ExportError {
    GraphNotFound,
    SerializationError(String),
    IoError(std::io::Error),
}
```

### Step 2: Add Export System (Day 3)

**File**: Update `src/contexts/graph_management/plugin.rs`

```rust
// Add to imports
use crate::contexts::graph_management::exporter::{export_graph_to_file, GraphExporter};

// Add system to plugin
app.add_systems(
    Update,
    (
        // Existing systems...
        export_graph_to_file
            .run_if(export_requested)
            .in_set(GraphManagementSet::Export),
    ),
);
```

**File**: Add to `src/contexts/graph_management/exporter.rs`

```rust
/// System to handle export requests
pub fn export_graph_to_file(
    keyboard: Res<ButtonInput<KeyCode>>,
    storage: Res<GraphStorage>,
    graphs: Query<(&GraphIdentity, &GraphMetadata)>,
    nodes: Query<(&NodeIdentity, &NodeContent, &SpatialPosition)>,
    edges: Query<(&EdgeIdentity, &EdgeRelationship)>,
) {
    // Check for Ctrl+S
    if (keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight))
        && keyboard.just_pressed(KeyCode::KeyS)
    {
        // Find active graph (for now, just use the first one)
        if let Some((graph_id, metadata)) = graphs.iter().next() {
            match GraphExporter::export_to_json(
                *graph_id,
                &storage,
                &nodes,
                &edges,
                &graphs,
            ) {
                Ok(json) => {
                    // For now, save to a fixed location
                    let path = Path::new("exported_graph.json");
                    match GraphExporter::save_to_file(path, &json) {
                        Ok(_) => info!("Graph exported successfully to {:?}", path),
                        Err(e) => error!("Failed to save file: {}", e),
                    }
                }
                Err(e) => error!("Failed to export graph: {:?}", e),
            }
        } else {
            warn!("No graph to export");
        }
    }
}

/// Condition to check if export was requested
pub fn export_requested(keyboard: Res<ButtonInput<KeyCode>>) -> bool {
    (keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight))
        && keyboard.just_pressed(KeyCode::KeyS)
}
```

### Step 3: Implement Export Logic (Day 4)

Complete the `export_to_json` implementation:

```rust
impl GraphExporter {
    pub fn export_to_json(
        graph_id: GraphIdentity,
        storage: &GraphStorage,
        nodes: &Query<(&NodeIdentity, &NodeContent, &SpatialPosition)>,
        edges: &Query<(&EdgeIdentity, &EdgeRelationship)>,
        graphs: &Query<&GraphMetadata>,
    ) -> Result<String, ExportError> {
        // Get graph metadata
        let metadata = graphs
            .iter()
            .find(|m| /* match graph_id */)
            .ok_or(ExportError::GraphNotFound)?
            .clone();

        // Collect nodes
        let mut internal_nodes = Vec::new();
        for (node_id, content, position) in nodes.iter() {
            // Filter by graph_id
            internal_nodes.push(InternalNode {
                id: node_id.0.to_string(),
                position: Position3D {
                    x: position.coordinates_3d.x,
                    y: position.coordinates_3d.y,
                    z: position.coordinates_3d.z,
                },
                content: content.clone(),
            });
        }

        // Collect edges
        let mut internal_edges = Vec::new();
        for (edge_id, relationship) in edges.iter() {
            // Filter by graph_id
            internal_edges.push(InternalEdge {
                id: edge_id.0.to_string(),
                source_id: relationship.source.0.to_string(),
                target_id: relationship.target.0.to_string(),
                relationship: relationship.clone(),
            });
        }

        // Create internal format
        let internal_format = InternalGraphFormat {
            version: "1.0.0".to_string(),
            metadata,
            nodes: internal_nodes,
            edges: internal_edges,
        };

        // Serialize to JSON
        serde_json::to_string_pretty(&internal_format)
            .map_err(|e| ExportError::SerializationError(e.to_string()))
    }
}
```

### Step 4: Add File Dialog Support (Day 5)

**Update `Cargo.toml`**:
```toml
[dependencies]
rfd = "0.15"
```

**Update export system**:
```rust
use rfd::FileDialog;

pub fn export_graph_to_file(
    // ... existing parameters
) {
    if export_requested {
        // Show save dialog
        if let Some(path) = FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_file_name("graph.json")
            .save_file()
        {
            // Export logic with selected path
        }
    }
}
```

### Step 5: Update Import for File Dialog (Day 6)

Update the import system to use file dialog:

```rust
pub fn import_graph_from_file(
    // ... existing parameters
) {
    if import_requested {
        // Show open dialog
        if let Some(path) = FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            // Import logic with selected path
        }
    }
}
```

### Step 6: Test Round-Trip (Day 7)

1. Create a test graph
2. Export it to JSON
3. Clear the current graph
4. Import the exported JSON
5. Verify all data is preserved

### Step 7: Write Tests (Day 8-9)

Create `src/contexts/graph_management/tests/import_export_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_empty_graph() {
        // Test exporting a graph with no nodes/edges
    }

    #[test]
    fn test_export_complex_graph() {
        // Test exporting a graph with multiple nodes and edges
    }

    #[test]
    fn test_round_trip_preservation() {
        // Test that export->import preserves all data
    }

    #[test]
    fn test_export_error_handling() {
        // Test error cases
    }
}
```

## Success Criteria

1. ✅ Can export any graph to JSON file
2. ✅ Can choose export location via file dialog
3. ✅ Can import from any JSON file via file dialog
4. ✅ Round-trip (export then import) preserves all data
5. ✅ Proper error handling and user feedback
6. ✅ Tests pass for all import/export scenarios

## Timeline

- **Days 1-2**: Implement export service and data structures
- **Day 3**: Add export system and keyboard handler
- **Day 4**: Complete export logic implementation
- **Day 5**: Add file dialog for export
- **Day 6**: Update import to use file dialog
- **Day 7**: Test round-trip functionality
- **Days 8-9**: Write comprehensive tests

**Total**: 9 working days (approximately 2 weeks with buffer)

## Dependencies

- Add `rfd` crate for file dialogs
- Ensure `serde` and `serde_json` are available
- No breaking changes to existing code

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| File dialog doesn't work on all platforms | Test on Linux/Windows/Mac, have fallback |
| Large graphs cause performance issues | Add progress indicator, optimize serialization |
| Data loss during export | Validate before saving, keep backups |
| Format incompatibility | Version the format, add migration support |

## Next Steps After Completion

1. Add support for other formats (GraphML, DOT)
2. Implement auto-save functionality
3. Add export presets/templates
4. Create import/export documentation

---

*Created*: December 2024
*Priority*: HIGH - Critical for application usability
