//! Tests for standard IPLD codecs and CIM-specific JSON types

use cim_ipld::{
    cim_json,
    codec_types::{AlchemistConfig, Position, WorkflowGraph, WorkflowMetadata, WorkflowNode},
    standard, CodecOperations, CodecRegistry, DagCborCodec, DagJsonCodec, Result,
};
use std::collections::HashMap;

#[test]
fn test_codec_registry_initialization() {
    let registry = CodecRegistry::new();

    // Check standard codecs are registered
    assert!(registry.contains(standard::RAW));
    assert!(registry.contains(standard::JSON));
    assert!(registry.contains(standard::DAG_CBOR));
    assert!(registry.contains(standard::DAG_JSON));

    // Check CIM-specific codecs are registered
    assert!(registry.contains(cim_json::ALCHEMIST));
    assert!(registry.contains(cim_json::WORKFLOW_GRAPH));
    assert!(registry.contains(cim_json::CONTEXT_GRAPH));

    // Check codec names
    assert_eq!(registry.get(standard::DAG_CBOR).unwrap().name(), "dag-cbor");
    assert_eq!(registry.get(standard::DAG_JSON).unwrap().name(), "dag-json");
    assert_eq!(
        registry.get(cim_json::ALCHEMIST).unwrap().name(),
        "cim-alchemist-json"
    );
}

#[test]
fn test_dag_cbor_encoding() {
    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct TestData {
        id: u64,
        name: String,
        values: Vec<f64>,
        metadata: HashMap<String, String>,
    }

    let data = TestData {
        id: 12345,
        name: "test-node".to_string(),
        values: vec![1.0, 2.5, 3.14159],
        metadata: {
            let mut m = HashMap::new();
            m.insert("type".to_string(), "sensor".to_string());
            m.insert("location".to_string(), "lab-1".to_string());
            m
        },
    };

    // Encode
    let encoded = DagCborCodec::encode(&data).unwrap();
    assert!(!encoded.is_empty());

    // Decode
    let decoded: TestData = DagCborCodec::decode(&encoded).unwrap();
    assert_eq!(data, decoded);
}

#[test]
fn test_dag_json_encoding() {
    use serde_json::json;

    let data = json!({
        "graph": {
            "nodes": [
                {"id": 1, "label": "A"},
                {"id": 2, "label": "B"},
                {"id": 3, "label": "C"}
            ],
            "edges": [
                {"from": 1, "to": 2},
                {"from": 2, "to": 3}
            ]
        },
        "metadata": {
            "created": "2024-01-01",
            "version": "1.0"
        }
    });

    // Encode
    let encoded = DagJsonCodec::encode(&data).unwrap();
    assert!(!encoded.is_empty());

    // Decode
    let decoded: serde_json::Value = DagJsonCodec::decode(&encoded).unwrap();
    assert_eq!(data, decoded);

    // Pretty print
    let pretty = DagJsonCodec::encode_pretty(&data).unwrap();
    assert!(pretty.contains("\"graph\""));
    assert!(pretty.contains("\"nodes\""));
    assert!(pretty.contains("\"edges\""));
}

#[test]
fn test_codec_operations_trait() {
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Event {
        event_type: String,
        timestamp: u64,
        data: HashMap<String, serde_json::Value>,
    }

    let event = Event {
        event_type: "node_created".to_string(),
        timestamp: 1234567890,
        data: {
            let mut d = HashMap::new();
            d.insert("node_id".to_string(), serde_json::json!("node-123"));
            d.insert(
                "position".to_string(),
                serde_json::json!([10.0, 20.0, 30.0]),
            );
            d
        },
    };

    // Test DAG-CBOR encoding via trait
    let cbor = event.to_dag_cbor().unwrap();
    assert!(!cbor.is_empty());

    // Test DAG-JSON encoding via trait
    let json = event.to_dag_json().unwrap();
    assert!(!json.is_empty());

    // Test pretty printing
    let pretty = event.to_dag_json_pretty().unwrap();
    assert!(pretty.contains("\"event_type\""));
    assert!(pretty.contains("\"node_created\""));
}

#[test]
fn test_alchemist_config_serialization() {
    use cim_ipld::codec_types::{DomainConfig, InfrastructureConfig, StorageConfig};

    let config = AlchemistConfig {
        version: "2.0.0".to_string(),
        domains: vec![DomainConfig {
            name: "test-domain".to_string(),
            enabled: true,
            settings: HashMap::new(),
        }],
        infrastructure: InfrastructureConfig {
            nats_url: "nats://test:4222".to_string(),
            storage: StorageConfig {
                backend: "memory".to_string(),
                options: HashMap::new(),
            },
        },
        metadata: HashMap::new(),
    };

    // Test CBOR round-trip
    let cbor = config.to_dag_cbor().unwrap();
    let decoded_cbor: AlchemistConfig = DagCborCodec::decode(&cbor).unwrap();
    assert_eq!(config.version, decoded_cbor.version);
    assert_eq!(config.domains.len(), decoded_cbor.domains.len());

    // Test JSON round-trip
    let json = config.to_dag_json().unwrap();
    let decoded_json: AlchemistConfig = DagJsonCodec::decode(&json).unwrap();
    assert_eq!(config.version, decoded_json.version);
    assert_eq!(
        config.infrastructure.nats_url,
        decoded_json.infrastructure.nats_url
    );
}

#[test]
fn test_workflow_graph_serialization() {
    use cim_ipld::codec_types::WorkflowEdge;

    let workflow = WorkflowGraph {
        id: "test-workflow".to_string(),
        name: "Test Workflow".to_string(),
        nodes: vec![
            WorkflowNode {
                id: "n1".to_string(),
                node_type: "start".to_string(),
                label: "Start".to_string(),
                position: Position {
                    x: 0.0,
                    y: 0.0,
                    z: Some(0.0),
                },
                data: HashMap::new(),
            },
            WorkflowNode {
                id: "n2".to_string(),
                node_type: "end".to_string(),
                label: "End".to_string(),
                position: Position {
                    x: 100.0,
                    y: 0.0,
                    z: Some(0.0),
                },
                data: HashMap::new(),
            },
        ],
        edges: vec![WorkflowEdge {
            id: "e1".to_string(),
            source: "n1".to_string(),
            target: "n2".to_string(),
            edge_type: "flow".to_string(),
            data: HashMap::new(),
        }],
        metadata: WorkflowMetadata {
            created_at: 1000,
            updated_at: 2000,
            version: "1.0".to_string(),
            tags: vec!["test".to_string()],
        },
    };

    // Test CBOR encoding
    let cbor = workflow.to_dag_cbor().unwrap();
    let decoded_cbor: WorkflowGraph = DagCborCodec::decode(&cbor).unwrap();
    assert_eq!(workflow.id, decoded_cbor.id);
    assert_eq!(workflow.nodes.len(), decoded_cbor.nodes.len());
    assert_eq!(workflow.edges.len(), decoded_cbor.edges.len());

    // Test JSON pretty printing
    let pretty = workflow.to_dag_json_pretty().unwrap();
    assert!(pretty.contains("\"test-workflow\""));
    assert!(pretty.contains("\"Start\""));
    assert!(pretty.contains("\"End\""));
}

#[test]
fn test_codec_error_handling() {
    // Test decoding invalid CBOR
    let invalid_cbor = vec![0xFF, 0xFF, 0xFF];
    let result: Result<serde_json::Value> = DagCborCodec::decode(&invalid_cbor);
    assert!(result.is_err());

    // Test decoding invalid JSON
    let invalid_json = b"{ invalid json }";
    let result: Result<serde_json::Value> = DagJsonCodec::decode(invalid_json);
    assert!(result.is_err());
}

#[test]
fn test_standard_codec_constants() {
    // Verify standard codec values match IPLD spec
    assert_eq!(standard::RAW, 0x55);
    assert_eq!(standard::JSON, 0x0200);
    assert_eq!(standard::CBOR, 0x51);
    assert_eq!(standard::DAG_PB, 0x70);
    assert_eq!(standard::DAG_CBOR, 0x71);
    assert_eq!(standard::DAG_JSON, 0x0129);
    assert_eq!(standard::LIBP2P_KEY, 0x72);
    assert_eq!(standard::GIT_RAW, 0x78);
}

#[test]
fn test_cim_codec_range() {
    // Verify CIM codecs are in the correct range
    assert!((0x340000..=0x34FFFF).contains(&cim_json::ALCHEMIST));
    assert!((0x340000..=0x34FFFF).contains(&cim_json::WORKFLOW_GRAPH));
    assert!((0x340000..=0x34FFFF).contains(&cim_json::CONTEXT_GRAPH));
    assert!((0x340000..=0x34FFFF).contains(&cim_json::DOMAIN_MODEL));

    // Verify graph-specific codecs
    assert!((0x340100..=0x3401FF).contains(&cim_json::GRAPH_LAYOUT));
    assert!((0x340100..=0x3401FF).contains(&cim_json::NODE_COLLECTION));

    // Verify workflow-specific codecs
    assert!((0x340200..=0x3402FF).contains(&cim_json::WORKFLOW_DEFINITION));
    assert!((0x340200..=0x3402FF).contains(&cim_json::WORKFLOW_STATE));
}
