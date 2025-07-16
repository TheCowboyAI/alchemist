# IPLD Codec Support in CIM-IPLD

## Overview

CIM-IPLD provides comprehensive support for standard IPLD (InterPlanetary Linked Data) codecs alongside custom CIM-specific JSON types. This enables seamless integration with the broader IPLD ecosystem while maintaining type safety and domain-specific optimizations.

## Standard IPLD Codecs

### Supported Codecs

| Codec      | Code   | Description        | Status      |
| ---------- | ------ | ------------------ | ----------- |
| raw        | 0x55   | Raw binary data    | ✅ Supported |
| json       | 0x0200 | Standard JSON      | ✅ Supported |
| cbor       | 0x51   | Standard CBOR      | ✅ Supported |
| dag-pb     | 0x70   | MerkleDAG protobuf | ✅ Supported |
| dag-cbor   | 0x71   | MerkleDAG CBOR     | ✅ Supported |
| dag-json   | 0x0129 | MerkleDAG JSON     | ✅ Supported |
| libp2p-key | 0x72   | Libp2p public key  | ✅ Supported |
| git-raw    | 0x78   | Git objects        | ✅ Supported |

### DAG-CBOR

DAG-CBOR is the primary binary encoding format for IPLD. It provides:
- Compact binary representation
- Deterministic encoding
- Support for IPLD links (CIDs)
- Efficient serialization/deserialization

```rust
use cim_ipld::{DagCborCodec, Result};

// Encode any serializable type
let data = MyStruct { field: "value" };
let encoded = DagCborCodec::encode(&data)?;

// Decode back
let decoded: MyStruct = DagCborCodec::decode(&encoded)?;
```

### DAG-JSON

DAG-JSON provides human-readable encoding with IPLD semantics:
- JSON-based format
- Support for IPLD links
- Pretty-printing for debugging
- Compatible with standard JSON tools

```rust
use cim_ipld::{DagJsonCodec, Result};

// Encode to JSON bytes
let encoded = DagJsonCodec::encode(&data)?;

// Pretty-print for human readability
let pretty = DagJsonCodec::encode_pretty(&data)?;
println!("{}", pretty);
```

## CIM-Specific JSON Types

### Type Registry

| Type                | Code     | Description                  |
| ------------------- | -------- | ---------------------------- |
| alchemist           | 0x340000 | Alchemist configuration      |
| workflow-graph      | 0x340001 | Workflow graph definitions   |
| context-graph       | 0x340002 | Context graph structures     |
| concept-space       | 0x340003 | Conceptual space definitions |
| domain-model        | 0x340004 | Domain model specifications  |
| event-stream        | 0x340005 | Event stream metadata        |
| command-batch       | 0x340006 | Command batch definitions    |
| query-result        | 0x340007 | Query result structures      |
| graph-layout        | 0x340100 | Graph layout information     |
| graph-metadata      | 0x340101 | Graph metadata               |
| node-collection     | 0x340102 | Collection of nodes          |
| edge-collection     | 0x340103 | Collection of edges          |
| workflow-definition | 0x340200 | Workflow definitions         |
| workflow-state      | 0x340201 | Workflow execution state     |
| workflow-history    | 0x340202 | Workflow execution history   |
| workflow-template   | 0x340203 | Reusable workflow templates  |

### Alchemist Configuration

The Alchemist configuration format stores CIM system configuration:

```rust
use cim_ipld::codec_types::{AlchemistConfig, DomainConfig, InfrastructureConfig};

let config = AlchemistConfig {
    version: "1.0.0".to_string(),
    domains: vec![
        DomainConfig {
            name: "graph".to_string(),
            enabled: true,
            settings: HashMap::new(),
        },
    ],
    infrastructure: InfrastructureConfig {
        nats_url: "nats://localhost:4222".to_string(),
        storage: StorageConfig {
            backend: "jetstream".to_string(),
            options: HashMap::new(),
        },
    },
    metadata: HashMap::new(),
};
```

### Workflow Graph

Workflow graphs represent executable business processes:

```rust
use cim_ipld::codec_types::{WorkflowGraph, WorkflowNode, WorkflowEdge};

let workflow = WorkflowGraph {
    id: "wf-001".to_string(),
    name: "Data Pipeline".to_string(),
    nodes: vec![/* nodes */],
    edges: vec![/* edges */],
    metadata: WorkflowMetadata { /* ... */ },
};
```

### Context Graph

Context graphs capture domain relationships and bounded contexts:

```rust
use cim_ipld::codec_types::{ContextGraph, Entity, Relationship};

let context = ContextGraph {
    id: "ctx-001".to_string(),
    context: "order-management".to_string(),
    entities: vec![/* entities */],
    relationships: vec![/* relationships */],
    metadata: ContextMetadata { /* ... */ },
};
```

## Codec Operations Trait

The `CodecOperations` trait provides convenient encoding methods for any serializable type:

```rust
use cim_ipld::CodecOperations;

#[derive(Serialize, Deserialize)]
struct MyData {
    name: String,
    value: u64,
}

let data = MyData { 
    name: "example".to_string(), 
    value: 42 
};

// Use trait methods
let cbor = data.to_dag_cbor()?;
let json = data.to_dag_json()?;
let pretty = data.to_dag_json_pretty()?;
```

## Codec Registry

The codec registry manages all available codecs:

```rust
use cim_ipld::{CodecRegistry, CimCodec};
use std::sync::Arc;

// Create registry (automatically includes standard codecs)
let registry = CodecRegistry::new();

// Check if a codec is registered
if registry.contains(0x71) {
    let codec = registry.get(0x71).unwrap();
    println!("Found codec: {}", codec.name());
}

// Register custom codec
struct MyCodec;
impl CimCodec for MyCodec {
    fn code(&self) -> u64 { 0x350000 }
    fn name(&self) -> &str { "my-codec" }
}

registry.register(Arc::new(MyCodec))?;
```

## Integration with CIM-IPLD

### Content Type Mapping

IPLD codecs are automatically selected based on content type:

```rust
use cim_ipld::{ContentService, ContentType};

// Workflow graphs use dag-json by default
let workflow = create_workflow();
let cid = service.store_content(
    workflow.to_dag_json()?,
    ContentType::Custom(cim_json::WORKFLOW_GRAPH),
).await?;

// Binary data uses dag-cbor for efficiency
let large_data = generate_data();
let cid = service.store_content(
    large_data.to_dag_cbor()?,
    ContentType::Custom(0x350000),
).await?;
```

### CID Generation

All IPLD codecs produce consistent CIDs:

```rust
use cim_ipld::{generate_cid, DagCborCodec, DagJsonCodec};

let data = MyStruct { /* ... */ };

// Same data produces same CID regardless of codec
let cbor_bytes = data.to_dag_cbor()?;
let cbor_cid = generate_cid(&cbor_bytes)?;

// JSON encoding produces different CID (different bytes)
let json_bytes = data.to_dag_json()?;
let json_cid = generate_cid(&json_bytes)?;

assert_ne!(cbor_cid, json_cid); // Different encodings = different CIDs
```

## Best Practices

### Choosing a Codec

1. **DAG-CBOR**: Use for efficient storage and transmission
   - Binary data
   - Performance-critical paths
   - Cross-language compatibility

2. **DAG-JSON**: Use for human-readable formats
   - Configuration files
   - API responses
   - Debugging and logging

3. **Custom CIM Types**: Use for domain-specific data
   - Workflow definitions
   - Graph structures
   - Domain models

### Performance Considerations

```rust
// Benchmark results (approximate)
// DAG-CBOR: 100KB data → 45KB encoded (55% compression)
// DAG-JSON: 100KB data → 120KB encoded (20% expansion)

// For large datasets, prefer CBOR
let large_dataset = load_dataset(); // 10MB
let cbor = large_dataset.to_dag_cbor()?; // ~4.5MB
let json = large_dataset.to_dag_json()?; // ~12MB
```

### Error Handling

```rust
use cim_ipld::{Result, Error};

fn process_data(data: &[u8]) -> Result<MyType> {
    // Try CBOR first (more efficient)
    match DagCborCodec::decode::<MyType>(data) {
        Ok(decoded) => Ok(decoded),
        Err(Error::CborError(_)) => {
            // Fall back to JSON
            DagJsonCodec::decode::<MyType>(data)
                .map_err(|_| Error::InvalidContent("Unknown format".into()))
        }
        Err(e) => Err(e),
    }
}
```

## Examples

### Complete Example: Storing Workflow

```rust
use cim_ipld::{
    CodecOperations, ContentService, ContentType,
    codec_types::{WorkflowGraph, WorkflowNode, Position},
    cim_json::WORKFLOW_GRAPH,
};

async fn store_workflow(service: &ContentService) -> Result<Cid> {
    // Create workflow
    let workflow = WorkflowGraph {
        id: "wf-001".to_string(),
        name: "Order Processing".to_string(),
        nodes: vec![
            WorkflowNode {
                id: "start".to_string(),
                node_type: "trigger".to_string(),
                label: "Order Received".to_string(),
                position: Position { x: 0.0, y: 0.0, z: None },
                data: HashMap::new(),
            },
            // ... more nodes
        ],
        edges: vec![/* ... */],
        metadata: Default::default(),
    };

    // Encode as DAG-JSON for readability
    let encoded = workflow.to_dag_json()?;

    // Store with appropriate content type
    let cid = service.store_content(
        encoded,
        ContentType::Custom(WORKFLOW_GRAPH),
    ).await?;

    println!("Stored workflow with CID: {}", cid);
    Ok(cid)
}
```

### Cross-Codec Compatibility

```rust
// Data can be transcoded between formats
let original = MyData { /* ... */ };

// Store as CBOR
let cbor = original.to_dag_cbor()?;
let cid = store_content(cbor, ContentType::Custom(0x350000)).await?;

// Retrieve and convert to JSON for API
let retrieved = retrieve_content(&cid).await?;
let decoded: MyData = DagCborCodec::decode(&retrieved)?;
let json_response = decoded.to_dag_json_pretty()?;

// Send JSON to client
send_response(json_response);
```

## Future Extensions

### Planned Codec Support

- **dag-jose**: Encrypted/signed JSON objects
- **car**: Content-addressed archives
- **unixfs**: File system representation
- **bitcoin-block**: Bitcoin blockchain data
- **ethereum-block**: Ethereum blockchain data

### Custom Codec Development

To add new codecs:

1. Define codec constant in appropriate range
2. Implement `CimCodec` trait
3. Add to codec registry
4. Document usage patterns

```rust
pub struct MyCustomCodec;

impl CimCodec for MyCustomCodec {
    fn code(&self) -> u64 { 0x360000 } // Custom range
    fn name(&self) -> &str { "my-custom-codec" }
}

// Optional: Add encoding/decoding methods
impl MyCustomCodec {
    pub fn encode(data: &MyType) -> Result<Vec<u8>> { /* ... */ }
    pub fn decode(data: &[u8]) -> Result<MyType> { /* ... */ }
}
```

## Conclusion

CIM-IPLD's codec support provides:
- Full compatibility with IPLD ecosystem
- Type-safe domain-specific formats
- Efficient encoding options
- Extensible architecture

This enables CIM to participate in the broader IPLD ecosystem while maintaining its domain-driven design principles and performance requirements. 