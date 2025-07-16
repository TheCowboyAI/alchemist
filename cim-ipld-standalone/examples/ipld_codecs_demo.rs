//! Example demonstrating standard IPLD codecs and CIM-specific JSON types

use cim_ipld::{
    Result, CodecRegistry,
    codec::ipld_codecs::{
        CodecOperations, DagCborCodec, DagJsonCodec, 
        standard, cim_json,
        types::{
            AlchemistConfig, WorkflowGraph, WorkflowNode, Position, WorkflowMetadata,
            WorkflowEdge, DomainConfig, InfrastructureConfig, StorageConfig,
            ConceptSpace, Dimension, Concept, ConceptSpaceMetadata,
            DomainModel, Aggregate, Property, DomainModelMetadata,
            EventStream, StreamEvent, EventStreamMetadata,
        },
    },
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== CIM-IPLD Codec Demo ===\n");

    // Initialize codec registry
    let _registry = CodecRegistry::new();
    
    // Show registered codecs
    println!("Registered standard IPLD codecs:");
    println!("  - Raw (0x{:x})", standard::RAW);
    println!("  - JSON (0x{:x})", standard::JSON);
    println!("  - DAG-CBOR (0x{:x})", standard::DAG_CBOR);
    println!("  - DAG-JSON (0x{:x})", standard::DAG_JSON);
    
    println!("\nRegistered CIM-specific JSON codecs:");
    println!("  - Alchemist (0x{:x})", cim_json::ALCHEMIST);
    println!("  - Workflow Graph (0x{:x})", cim_json::WORKFLOW_GRAPH);
    println!("  - Context Graph (0x{:x})", cim_json::CONTEXT_GRAPH);
    
    // Example 1: Using DAG-CBOR
    println!("\n--- Example 1: DAG-CBOR Encoding ---");
    demo_dag_cbor()?;
    
    // Example 2: Using DAG-JSON
    println!("\n--- Example 2: DAG-JSON Encoding ---");
    demo_dag_json()?;
    
    // Example 3: CIM Alchemist Config
    println!("\n--- Example 3: CIM Alchemist Config ---");
    demo_alchemist_config()?;
    
    // Example 4: CIM Workflow Graph
    println!("\n--- Example 4: CIM Workflow Graph ---");
    demo_workflow_graph()?;
    
    // Example 5: Using the CodecOperations trait
    println!("\n--- Example 5: CodecOperations Trait ---");
    demo_codec_operations()?;
    
    // Example 6: CIM Concept Space
    println!("\n--- Example 6: CIM Concept Space ---");
    demo_concept_space()?;
    
    // Example 7: CIM Domain Model
    println!("\n--- Example 7: CIM Domain Model ---");
    demo_domain_model()?;
    
    // Example 8: CIM Event Stream
    println!("\n--- Example 8: CIM Event Stream ---");
    demo_event_stream()?;
    
    Ok(())
}

fn demo_dag_cbor() -> Result<()> {
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct GraphNode {
        id: String,
        label: String,
        connections: Vec<String>,
        metadata: HashMap<String, String>,
    }
    
    let node = GraphNode {
        id: "node-1".to_string(),
        label: "Start Node".to_string(),
        connections: vec!["node-2".to_string(), "node-3".to_string()],
        metadata: {
            let mut m = HashMap::new();
            m.insert("type".to_string(), "process".to_string());
            m.insert("priority".to_string(), "high".to_string());
            m
        },
    };
    
    // Encode
    let encoded = DagCborCodec::encode(&node)?;
    println!("Encoded DAG-CBOR: {encoded.len(} bytes"));
    println!("First 32 bytes: {:?}", &encoded[..32.min(encoded.len())]);
    
    // Decode
    let decoded: GraphNode = DagCborCodec::decode(&encoded)?;
    println!("Decoded: {:?}", decoded);
    
    Ok(())
}

fn demo_dag_json() -> Result<()> {
    use serde_json::json;
    
    let data = json!({
        "type": "event",
        "timestamp": 1234567890,
        "payload": {
            "action": "node_added",
            "node_id": "node-42",
            "attributes": {
                "color": "blue",
                "size": 10
            }
        }
    });
    
    // Encode
    let encoded = DagJsonCodec::encode(&data)?;
    println!("Encoded DAG-JSON: {encoded.len(} bytes"));
    
    // Pretty print
    let pretty = DagJsonCodec::encode_pretty(&data)?;
    println!("Pretty DAG-JSON:\n{pretty}");
    
    // Decode
    let decoded: serde_json::Value = DagJsonCodec::decode(&encoded)?;
    assert_eq!(data, decoded);
    
    Ok(())
}

fn demo_alchemist_config() -> Result<()> {
    let config = AlchemistConfig {
        version: "1.0.0".to_string(),
        domains: vec![
            DomainConfig {
                name: "graph".to_string(),
                enabled: true,
                settings: {
                    let mut s = HashMap::new();
                    s.insert("max_nodes".to_string(), serde_json::json!(10000));
                    s.insert("auto_layout".to_string(), serde_json::json!(true));
                    s
                },
            },
            DomainConfig {
                name: "workflow".to_string(),
                enabled: true,
                settings: {
                    let mut s = HashMap::new();
                    s.insert("execution_mode".to_string(), serde_json::json!("async"));
                    s.insert("max_parallel".to_string(), serde_json::json!(5));
                    s
                },
            },
        ],
        infrastructure: InfrastructureConfig {
            nats_url: "nats://localhost:4222".to_string(),
            storage: StorageConfig {
                backend: "jetstream".to_string(),
                options: {
                    let mut o = HashMap::new();
                    o.insert("bucket".to_string(), serde_json::json!("cim-storage"));
                    o.insert("replicas".to_string(), serde_json::json!(3));
                    o
                },
            },
        },
        metadata: {
            let mut m = HashMap::new();
            m.insert("environment".to_string(), serde_json::json!("production"));
            m.insert("region".to_string(), serde_json::json!("us-west-2"));
            m
        },
    };
    
    // Encode as DAG-JSON
    let json = config.to_dag_json_pretty()?;
    println!("Alchemist Config (DAG-JSON):\n{json}");
    
    // Encode as DAG-CBOR
    let cbor = config.to_dag_cbor()?;
    println!("\nAlchemist Config (DAG-CBOR): {cbor.len(} bytes"));
    
    Ok(())
}

fn demo_workflow_graph() -> Result<()> {
    let workflow = WorkflowGraph {
        id: "wf-001".to_string(),
        name: "Data Processing Pipeline".to_string(),
        nodes: vec![
            WorkflowNode {
                id: "start".to_string(),
                node_type: "trigger".to_string(),
                label: "Start".to_string(),
                position: Position { x: 0.0, y: 0.0, z: None },
                data: HashMap::new(),
            },
            WorkflowNode {
                id: "validate".to_string(),
                node_type: "process".to_string(),
                label: "Validate Input".to_string(),
                position: Position { x: 200.0, y: 0.0, z: None },
                data: {
                    let mut d = HashMap::new();
                    d.insert("validator".to_string(), serde_json::json!("schema-v1"));
                    d
                },
            },
            WorkflowNode {
                id: "transform".to_string(),
                node_type: "process".to_string(),
                label: "Transform Data".to_string(),
                position: Position { x: 400.0, y: 0.0, z: None },
                data: {
                    let mut d = HashMap::new();
                    d.insert("transformer".to_string(), serde_json::json!("etl-v2"));
                    d
                },
            },
            WorkflowNode {
                id: "end".to_string(),
                node_type: "output".to_string(),
                label: "Complete".to_string(),
                position: Position { x: 600.0, y: 0.0, z: None },
                data: HashMap::new(),
            },
        ],
        edges: vec![
            WorkflowEdge {
                id: "e1".to_string(),
                source: "start".to_string(),
                target: "validate".to_string(),
                edge_type: "sequence".to_string(),
                data: HashMap::new(),
            },
            WorkflowEdge {
                id: "e2".to_string(),
                source: "validate".to_string(),
                target: "transform".to_string(),
                edge_type: "conditional".to_string(),
                data: {
                    let mut d = HashMap::new();
                    d.insert("condition".to_string(), serde_json::json!("valid == true"));
                    d
                },
            },
            WorkflowEdge {
                id: "e3".to_string(),
                source: "transform".to_string(),
                target: "end".to_string(),
                edge_type: "sequence".to_string(),
                data: HashMap::new(),
            },
        ],
        metadata: WorkflowMetadata {
            created_at: 1234567890,
            updated_at: 1234567900,
            version: "1.0.0".to_string(),
            tags: vec!["data-pipeline".to_string(), "etl".to_string()],
        },
    };
    
    // Encode as DAG-JSON
    let json = workflow.to_dag_json_pretty()?;
    println!("Workflow Graph (DAG-JSON):\n{json}");
    
    // Show codec info
    println!("\nThis would be stored with codec: 0x{:x} (cim-workflow-graph-json)", cim_json::WORKFLOW_GRAPH);
    
    Ok(())
}

fn demo_codec_operations() -> Result<()> {
    // Any serializable type can use CodecOperations
    let data = vec![
        ("temperature", 23.5),
        ("humidity", 65.0),
        ("pressure", 1013.25),
    ];
    
    println!("Original data: {:?}", data);
    
    // Encode as DAG-CBOR
    let cbor = data.to_dag_cbor()?;
    println!("\nDAG-CBOR encoding: {cbor.len(} bytes"));
    
    // Encode as DAG-JSON
    let json = data.to_dag_json()?;
    println!("DAG-JSON encoding: {json.len(} bytes"));
    println!("DAG-JSON content: {String::from_utf8_lossy(&json}"));
    
    // Pretty print
    let pretty = data.to_dag_json_pretty()?;
    println!("\nPretty DAG-JSON:\n{pretty}");
    
    Ok(())
}

fn demo_concept_space() -> Result<()> {
    let space = ConceptSpace {
        id: "cs-colors".to_string(),
        name: "Color Concept Space".to_string(),
        dimensions: vec![
            Dimension {
                name: "hue".to_string(),
                dimension_type: "circular".to_string(),
                range: (0.0, 360.0),
            },
            Dimension {
                name: "saturation".to_string(),
                dimension_type: "linear".to_string(),
                range: (0.0, 100.0),
            },
            Dimension {
                name: "lightness".to_string(),
                dimension_type: "linear".to_string(),
                range: (0.0, 100.0),
            },
        ],
        concepts: vec![
            Concept {
                id: "red".to_string(),
                label: "Red".to_string(),
                coordinates: vec![0.0, 100.0, 50.0],
                attributes: {
                    let mut a = HashMap::new();
                    a.insert("wavelength".to_string(), serde_json::json!("700nm"));
                    a.insert("emotion".to_string(), serde_json::json!("passion"));
                    a
                },
            },
            Concept {
                id: "blue".to_string(),
                label: "Blue".to_string(),
                coordinates: vec![240.0, 100.0, 50.0],
                attributes: {
                    let mut a = HashMap::new();
                    a.insert("wavelength".to_string(), serde_json::json!("450nm"));
                    a.insert("emotion".to_string(), serde_json::json!("calm"));
                    a
                },
            },
        ],
        metadata: ConceptSpaceMetadata {
            created_at: 1234567890,
            updated_at: 1234567890,
            version: "1.0".to_string(),
            algorithm: "manual-mapping".to_string(),
        },
    };
    
    let json = space.to_dag_json_pretty()?;
    println!("Concept Space (DAG-JSON):\n{json}");
    println!("\nThis would be stored with codec: 0x{:x} (cim-concept-space-json)", cim_json::CONCEPT_SPACE);
    
    Ok(())
}

fn demo_domain_model() -> Result<()> {
    let model = DomainModel {
        id: "dm-order".to_string(),
        name: "Order Management Domain".to_string(),
        aggregates: vec![
            Aggregate {
                id: "order".to_string(),
                name: "Order".to_string(),
                properties: vec![
                    Property {
                        name: "order_id".to_string(),
                        property_type: "uuid".to_string(),
                        required: true,
                        constraints: vec!["unique".to_string()],
                    },
                    Property {
                        name: "customer_id".to_string(),
                        property_type: "uuid".to_string(),
                        required: true,
                        constraints: vec!["foreign_key:customer".to_string()],
                    },
                    Property {
                        name: "total_amount".to_string(),
                        property_type: "decimal".to_string(),
                        required: true,
                        constraints: vec!["min:0".to_string()],
                    },
                ],
                invariants: vec![
                    "Total amount must equal sum of line items".to_string(),
                    "Order must have at least one line item".to_string(),
                ],
            },
        ],
        events: vec![],
        commands: vec![],
        metadata: DomainModelMetadata {
            version: "2.0".to_string(),
            bounded_context: "order-management".to_string(),
            created_at: 1234567890,
            tags: vec!["core".to_string(), "ecommerce".to_string()],
        },
    };
    
    let cbor = model.to_dag_cbor()?;
    println!("Domain Model (DAG-CBOR): {cbor.len(} bytes"));
    
    let json = model.to_dag_json_pretty()?;
    println!("\nDomain Model (DAG-JSON excerpt):");
    let lines: Vec<&str> = json.lines().take(20).collect();
    println!("{lines.join("\n"}"));
    println!("... (truncated)");
    
    println!("\nThis would be stored with codec: 0x{:x} (cim-domain-model-json)", cim_json::DOMAIN_MODEL);
    
    Ok(())
}

fn demo_event_stream() -> Result<()> {
    use serde_json::json;
    
    let stream = EventStream {
        id: "es-orders-2024".to_string(),
        stream_name: "order-events".to_string(),
        events: vec![
            StreamEvent {
                id: "evt-001".to_string(),
                event_type: "OrderCreated".to_string(),
                timestamp: 1234567890,
                aggregate_id: "order-123".to_string(),
                sequence: 1,
                payload: json!({
                    "order_id": "order-123",
                    "customer_id": "cust-456",
                    "items": [
                        {
                            "product_id": "prod-789",
                            "quantity": 2,
                            "price": 29.99
                        }
                    ],
                    "total": 59.98
                }),
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("user_id".to_string(), json!("user-001"));
                    m.insert("source".to_string(), json!("web"));
                    m
                },
            },
            StreamEvent {
                id: "evt-002".to_string(),
                event_type: "OrderShipped".to_string(),
                timestamp: 1234567950,
                aggregate_id: "order-123".to_string(),
                sequence: 2,
                payload: json!({
                    "order_id": "order-123",
                    "tracking_number": "TRACK-12345",
                    "carrier": "FedEx",
                    "estimated_delivery": "2024-01-15"
                }),
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("warehouse".to_string(), json!("warehouse-west"));
                    m
                },
            },
        ],
        metadata: EventStreamMetadata {
            created_at: 1234567890,
            last_event_at: 1234567950,
            event_count: 2,
            stream_version: "1.0".to_string(),
        },
    };
    
    let json = stream.to_dag_json_pretty()?;
    println!("Event Stream (DAG-JSON):\n{json}");
    
    println!("\nThis would be stored with codec: 0x{:x} (cim-event-stream-json)", cim_json::EVENT_STREAM);
    
    Ok(())
} 