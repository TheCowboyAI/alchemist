# MerkleDag JSON Schema Documentation

## Overview

The MerkleDag supports two JSON schema formats:
1. **Native Schema** - Optimized for internal serialization
2. **Arrows.app Schema** - Compatible with arrows.app for visualization

## Native Schema Format

The native schema is used for efficient serialization and deserialization of the MerkleDag state.

```json
{
  "nodes": [
    [
      0,  // NodeIndex
      {
        "cid": "QmXYZ...",
        "links": ["QmABC...", "QmDEF..."],
        "position": { "x": 0.0, "y": 0.0, "z": 0.0 },
        "render_state": {
          "color": [1.0, 0.5, 0.0, 1.0],
          "size": 0.5,
          "visible": true
        },
        "metadata": {
          "name": "Process A",
          "type": "bounded_context"
        },
        "uuid": "550e8400-e29b-41d4-a716-446655440000"
      }
    ]
  ],
  "edges": [
    [
      0,  // Source NodeIndex
      1,  // Target NodeIndex
      {
        "weight": 1.0,
        "proof": [],
        "thickness": 0.1,
        "style": "Solid"
      }
    ]
  ]
}
```

## Arrows.app Compatible Schema

The arrows.app schema follows the format expected by [arrows.app](https://arrows.app) for graph visualization and modeling.

### Example

```json
{
  "nodes": [
    {
      "id": "QmXYZ...",
      "labels": ["MerkleNode"],
      "properties": {
        "cid": "QmXYZ...",
        "uuid": "550e8400-e29b-41d4-a716-446655440000",
        "size": 0.5,
        "visible": true,
        "color": "#ff8000",
        "meta_name": "Process A",
        "meta_type": "bounded_context"
      },
      "position": {
        "x": 100.0,
        "y": 200.0
      }
    }
  ],
  "relationships": [
    {
      "id": "edge_0",
      "type": "LINKS_TO",
      "fromId": "QmXYZ...",
      "toId": "QmABC...",
      "properties": {
        "weight": 1.0,
        "thickness": 0.1,
        "style": "Solid",
        "has_proof": false
      }
    }
  ]
}
```

### Schema Details

#### Node Properties
- `id`: The CID (Content Identifier) of the node
- `labels`: Array of node types (always includes "MerkleNode")
- `properties`: Key-value pairs including:
  - `cid`: Content identifier
  - `uuid`: Unique identifier
  - `size`: Visual size (0.0-1.0)
  - `visible`: Boolean visibility flag
  - `color`: Hex color string (e.g., "#ff8000")
  - `meta_*`: Metadata fields prefixed with "meta_"
- `position`: 2D coordinates for visualization

#### Relationship Properties
- `id`: Unique edge identifier
- `type`: Relationship type (default: "LINKS_TO")
- `fromId`: Source node CID
- `toId`: Target node CID
- `properties`: Edge attributes including:
  - `weight`: Numeric weight
  - `thickness`: Visual thickness
  - `style`: "Solid", "Dashed", or "Dotted"
  - `has_proof`: Boolean indicating if cryptographic proof exists

## Usage Examples

### Export to arrows.app

```rust
// Export the DAG to arrows.app format
let json = merkle_dag.to_arrows_json()?;
std::fs::write("graph.json", json)?;
```

### Import from arrows.app

```rust
// Load a graph created in arrows.app
let json = std::fs::read_to_string("graph.json")?;
let dag = MerkleDag::from_arrows_json(&json)?;
```

### Convert between formats

```rust
// Native to arrows.app
let native_json = dag.to_json()?;
let arrows_json = dag.to_arrows_json()?;

// arrows.app to native
let imported_dag = MerkleDag::from_arrows_json(&arrows_json)?;
let native_json = imported_dag.to_json()?;
```

## Compatibility Notes

1. **CID Preservation**: Node IDs in arrows.app format are CIDs, preserving content addressing
2. **Metadata**: Custom properties are preserved with "meta_" prefix
3. **Color Format**: Colors are converted between RGBA floats and hex strings
4. **Position**: 3D positions are projected to 2D for arrows.app (x,z -> x,y)
5. **Edge Styles**: Limited to Solid, Dashed, and Dotted for compatibility

## Schema Validation

Both schemas support standard JSON schema validation. The structures are designed to be:
- Self-contained (no external references)
- Type-safe (using Rust's serde)
- Forward-compatible (unknown fields are preserved)
