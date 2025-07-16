//! Standard IPLD codecs and CIM-specific JSON types
//!
//! This module implements the standard IPLD codecs (dag-cbor, dag-json, etc.)
//! and adds support for CIM-specific JSON types like alchemist and workflow-graph.

use crate::{CimCodec, Result, Error};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Standard IPLD codec constants (from multicodec table)
pub mod standard {
    // Raw binary
    pub const RAW: u64 = 0x55;
    
    // JSON variants
    pub const JSON: u64 = 0x0200;
    
    // CBOR variants
    pub const CBOR: u64 = 0x51;
    
    // IPLD formats
    pub const DAG_PB: u64 = 0x70;     // MerkleDAG protobuf
    pub const DAG_CBOR: u64 = 0x71;   // MerkleDAG CBOR
    pub const DAG_JSON: u64 = 0x0129;  // MerkleDAG JSON
    pub const LIBP2P_KEY: u64 = 0x72; // Libp2p public key
    
    // Git formats
    pub const GIT_RAW: u64 = 0x78;
    
    // Bitcoin/Ethereum formats
    pub const BITCOIN_BLOCK: u64 = 0xb0;
    pub const BITCOIN_TX: u64 = 0xb1;
    pub const ETH_BLOCK: u64 = 0x90;
    pub const ETH_TX: u64 = 0x93;
}

// CIM-specific JSON types (using custom codec range)
pub mod cim_json {
    // Base CIM JSON types (0x340000-0x34FFFF)
    pub const ALCHEMIST: u64 = 0x340000;
    pub const WORKFLOW_GRAPH: u64 = 0x340001;
    pub const CONTEXT_GRAPH: u64 = 0x340002;
    pub const CONCEPT_SPACE: u64 = 0x340003;
    pub const DOMAIN_MODEL: u64 = 0x340004;
    pub const EVENT_STREAM: u64 = 0x340005;
    pub const COMMAND_BATCH: u64 = 0x340006;
    pub const QUERY_RESULT: u64 = 0x340007;
    
    // Graph-specific JSON types
    pub const GRAPH_LAYOUT: u64 = 0x340100;
    pub const GRAPH_METADATA: u64 = 0x340101;
    pub const NODE_COLLECTION: u64 = 0x340102;
    pub const EDGE_COLLECTION: u64 = 0x340103;
    
    // Workflow-specific JSON types
    pub const WORKFLOW_DEFINITION: u64 = 0x340200;
    pub const WORKFLOW_STATE: u64 = 0x340201;
    pub const WORKFLOW_HISTORY: u64 = 0x340202;
    pub const WORKFLOW_TEMPLATE: u64 = 0x340203;
}

/// Standard IPLD DAG-CBOR codec
pub struct DagCborCodec;

impl CimCodec for DagCborCodec {
    fn code(&self) -> u64 {
        standard::DAG_CBOR
    }

    fn name(&self) -> &str {
        "dag-cbor"
    }
}

impl DagCborCodec {
    /// Encode data as DAG-CBOR
    pub fn encode<T: Serialize>(data: &T) -> Result<Vec<u8>> {
        serde_cbor::to_vec(data)
            .map_err(|e| Error::CborError(e.to_string()))
    }

    /// Decode DAG-CBOR data
    pub fn decode<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Result<T> {
        serde_cbor::from_slice(data)
            .map_err(|e| Error::CborError(e.to_string()))
    }
}

/// Standard IPLD DAG-JSON codec
pub struct DagJsonCodec;

impl CimCodec for DagJsonCodec {
    fn code(&self) -> u64 {
        standard::DAG_JSON
    }

    fn name(&self) -> &str {
        "dag-json"
    }
}

impl DagJsonCodec {
    /// Encode data as DAG-JSON
    pub fn encode<T: Serialize>(data: &T) -> Result<Vec<u8>> {
        serde_json::to_vec(data)
            .map_err(Error::SerializationError)
    }

    /// Decode DAG-JSON data
    pub fn decode<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Result<T> {
        serde_json::from_slice(data)
            .map_err(Error::SerializationError)
    }

    /// Pretty-print encode
    pub fn encode_pretty<T: Serialize>(data: &T) -> Result<String> {
        serde_json::to_string_pretty(data)
            .map_err(Error::SerializationError)
    }
}

/// Raw binary codec
pub struct RawCodec;

impl CimCodec for RawCodec {
    fn code(&self) -> u64 {
        standard::RAW
    }

    fn name(&self) -> &str {
        "raw"
    }
}

/// Standard JSON codec
pub struct JsonCodec;

impl CimCodec for JsonCodec {
    fn code(&self) -> u64 {
        standard::JSON
    }

    fn name(&self) -> &str {
        "json"
    }
}

/// CIM Alchemist JSON codec
pub struct AlchemistJsonCodec;

impl CimCodec for AlchemistJsonCodec {
    fn code(&self) -> u64 {
        cim_json::ALCHEMIST
    }

    fn name(&self) -> &str {
        "cim-alchemist-json"
    }
}

/// CIM Workflow Graph JSON codec
pub struct WorkflowGraphJsonCodec;

impl CimCodec for WorkflowGraphJsonCodec {
    fn code(&self) -> u64 {
        cim_json::WORKFLOW_GRAPH
    }

    fn name(&self) -> &str {
        "cim-workflow-graph-json"
    }
}

/// CIM Context Graph JSON codec
pub struct ContextGraphJsonCodec;

impl CimCodec for ContextGraphJsonCodec {
    fn code(&self) -> u64 {
        cim_json::CONTEXT_GRAPH
    }

    fn name(&self) -> &str {
        "cim-context-graph-json"
    }
}

/// DAG-PB (MerkleDAG protobuf) codec
pub struct DagPbCodec;

impl CimCodec for DagPbCodec {
    fn code(&self) -> u64 {
        standard::DAG_PB
    }

    fn name(&self) -> &str {
        "dag-pb"
    }
}

/// Git raw object codec
pub struct GitRawCodec;

impl CimCodec for GitRawCodec {
    fn code(&self) -> u64 {
        standard::GIT_RAW
    }

    fn name(&self) -> &str {
        "git-raw"
    }
}

/// Libp2p key codec
pub struct Libp2pKeyCodec;

impl CimCodec for Libp2pKeyCodec {
    fn code(&self) -> u64 {
        standard::LIBP2P_KEY
    }

    fn name(&self) -> &str {
        "libp2p-key"
    }
}

/// CIM Concept Space JSON codec
pub struct ConceptSpaceJsonCodec;

impl CimCodec for ConceptSpaceJsonCodec {
    fn code(&self) -> u64 {
        cim_json::CONCEPT_SPACE
    }

    fn name(&self) -> &str {
        "cim-concept-space-json"
    }
}

/// CIM Domain Model JSON codec
pub struct DomainModelJsonCodec;

impl CimCodec for DomainModelJsonCodec {
    fn code(&self) -> u64 {
        cim_json::DOMAIN_MODEL
    }

    fn name(&self) -> &str {
        "cim-domain-model-json"
    }
}

/// CIM Event Stream JSON codec
pub struct EventStreamJsonCodec;

impl CimCodec for EventStreamJsonCodec {
    fn code(&self) -> u64 {
        cim_json::EVENT_STREAM
    }

    fn name(&self) -> &str {
        "cim-event-stream-json"
    }
}

/// Register all standard IPLD codecs
pub fn register_ipld_codecs(registry: &mut crate::CodecRegistry) -> Result<()> {
    // Standard IPLD codecs
    registry.register_standard(Arc::new(RawCodec))?;
    registry.register_standard(Arc::new(JsonCodec))?;
    registry.register_standard(Arc::new(DagCborCodec))?;
    registry.register_standard(Arc::new(DagJsonCodec))?;
    registry.register_standard(Arc::new(DagPbCodec))?;
    registry.register_standard(Arc::new(GitRawCodec))?;
    registry.register_standard(Arc::new(Libp2pKeyCodec))?;
    
    Ok(())
}

/// Register all CIM-specific JSON codecs
pub fn register_cim_json_codecs(registry: &mut crate::CodecRegistry) -> Result<()> {
    // CIM JSON codecs
    registry.register(Arc::new(AlchemistJsonCodec))?;
    registry.register(Arc::new(WorkflowGraphJsonCodec))?;
    registry.register(Arc::new(ContextGraphJsonCodec))?;
    registry.register(Arc::new(ConceptSpaceJsonCodec))?;
    registry.register(Arc::new(DomainModelJsonCodec))?;
    registry.register(Arc::new(EventStreamJsonCodec))?;
    
    Ok(())
}

/// Helper trait for encoding/decoding with specific codecs
pub trait CodecOperations {
    /// Encode using DAG-CBOR
    fn to_dag_cbor(&self) -> Result<Vec<u8>>
    where
        Self: Serialize + Sized,
    {
        DagCborCodec::encode(self)
    }

    /// Encode using DAG-JSON
    fn to_dag_json(&self) -> Result<Vec<u8>>
    where
        Self: Serialize + Sized,
    {
        DagJsonCodec::encode(self)
    }

    /// Encode using DAG-JSON (pretty-printed)
    fn to_dag_json_pretty(&self) -> Result<String>
    where
        Self: Serialize + Sized,
    {
        DagJsonCodec::encode_pretty(self)
    }
}

// Implement for all types
impl<T> CodecOperations for T {}

/// CIM-specific JSON type definitions
pub mod types {
    use serde::{Serialize, Deserialize};
    use std::collections::HashMap;
    
    /// Alchemist configuration format
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AlchemistConfig {
        pub version: String,
        pub domains: Vec<DomainConfig>,
        pub infrastructure: InfrastructureConfig,
        pub metadata: HashMap<String, serde_json::Value>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DomainConfig {
        pub name: String,
        pub enabled: bool,
        pub settings: HashMap<String, serde_json::Value>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InfrastructureConfig {
        pub nats_url: String,
        pub storage: StorageConfig,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StorageConfig {
        pub backend: String,
        pub options: HashMap<String, serde_json::Value>,
    }
    
    /// Workflow graph format
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WorkflowGraph {
        pub id: String,
        pub name: String,
        pub nodes: Vec<WorkflowNode>,
        pub edges: Vec<WorkflowEdge>,
        pub metadata: WorkflowMetadata,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WorkflowNode {
        pub id: String,
        pub node_type: String,
        pub label: String,
        pub position: Position,
        pub data: HashMap<String, serde_json::Value>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WorkflowEdge {
        pub id: String,
        pub source: String,
        pub target: String,
        pub edge_type: String,
        pub data: HashMap<String, serde_json::Value>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WorkflowMetadata {
        pub created_at: u64,
        pub updated_at: u64,
        pub version: String,
        pub tags: Vec<String>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Position {
        pub x: f64,
        pub y: f64,
        pub z: Option<f64>,
    }
    
    /// Context graph format
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ContextGraph {
        pub id: String,
        pub context: String,
        pub entities: Vec<Entity>,
        pub relationships: Vec<Relationship>,
        pub metadata: ContextMetadata,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Entity {
        pub id: String,
        pub entity_type: String,
        pub attributes: HashMap<String, serde_json::Value>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Relationship {
        pub id: String,
        pub source: String,
        pub target: String,
        pub relationship_type: String,
        pub attributes: HashMap<String, serde_json::Value>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ContextMetadata {
        pub domain: String,
        pub version: String,
        pub created_at: u64,
        pub tags: Vec<String>,
    }
    
    /// Concept space format
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ConceptSpace {
        pub id: String,
        pub name: String,
        pub dimensions: Vec<Dimension>,
        pub concepts: Vec<Concept>,
        pub metadata: ConceptSpaceMetadata,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Dimension {
        pub name: String,
        pub dimension_type: String,
        pub range: (f64, f64),
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Concept {
        pub id: String,
        pub label: String,
        pub coordinates: Vec<f64>,
        pub attributes: HashMap<String, serde_json::Value>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ConceptSpaceMetadata {
        pub created_at: u64,
        pub updated_at: u64,
        pub version: String,
        pub algorithm: String,
    }
    
    /// Domain model format
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DomainModel {
        pub id: String,
        pub name: String,
        pub aggregates: Vec<Aggregate>,
        pub events: Vec<DomainEvent>,
        pub commands: Vec<Command>,
        pub metadata: DomainModelMetadata,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Aggregate {
        pub id: String,
        pub name: String,
        pub properties: Vec<Property>,
        pub invariants: Vec<String>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Property {
        pub name: String,
        pub property_type: String,
        pub required: bool,
        pub constraints: Vec<String>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DomainEvent {
        pub id: String,
        pub name: String,
        pub aggregate_id: String,
        pub payload: HashMap<String, serde_json::Value>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Command {
        pub id: String,
        pub name: String,
        pub aggregate_id: String,
        pub parameters: HashMap<String, serde_json::Value>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DomainModelMetadata {
        pub version: String,
        pub bounded_context: String,
        pub created_at: u64,
        pub tags: Vec<String>,
    }
    
    /// Event stream format
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EventStream {
        pub id: String,
        pub stream_name: String,
        pub events: Vec<StreamEvent>,
        pub metadata: EventStreamMetadata,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StreamEvent {
        pub id: String,
        pub event_type: String,
        pub timestamp: u64,
        pub aggregate_id: String,
        pub sequence: u64,
        pub payload: serde_json::Value,
        pub metadata: HashMap<String, serde_json::Value>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EventStreamMetadata {
        pub created_at: u64,
        pub last_event_at: u64,
        pub event_count: u64,
        pub stream_version: String,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dag_cbor_roundtrip() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct TestData {
            name: String,
            value: u64,
        }
        
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        
        let encoded = DagCborCodec::encode(&data).unwrap();
        let decoded: TestData = DagCborCodec::decode(&encoded).unwrap();
        
        assert_eq!(data, decoded);
    }
    
    #[test]
    fn test_dag_json_roundtrip() {
        use types::WorkflowNode;
        
        let node = WorkflowNode {
            id: "node-1".to_string(),
            node_type: "process".to_string(),
            label: "Process Data".to_string(),
            position: types::Position { x: 100.0, y: 200.0, z: None },
            data: Default::default(),
        };
        
        let encoded = DagJsonCodec::encode(&node).unwrap();
        let decoded: WorkflowNode = DagJsonCodec::decode(&encoded).unwrap();
        
        assert_eq!(node.id, decoded.id);
        assert_eq!(node.label, decoded.label);
    }
    
    #[test]
    fn test_codec_operations_trait() {
        let data = vec![1, 2, 3, 4, 5];
        
        let cbor = data.to_dag_cbor().unwrap();
        let json = data.to_dag_json().unwrap();
        
        assert!(!cbor.is_empty());
        assert!(!json.is_empty());
    }
    
    #[test]
    fn test_concept_space_serialization() {
        use types::{ConceptSpace, Dimension, Concept, ConceptSpaceMetadata};
        
        let space = ConceptSpace {
            id: "cs-001".to_string(),
            name: "Color Space".to_string(),
            dimensions: vec![
                Dimension {
                    name: "hue".to_string(),
                    dimension_type: "circular".to_string(),
                    range: (0.0, 360.0),
                },
                Dimension {
                    name: "saturation".to_string(),
                    dimension_type: "linear".to_string(),
                    range: (0.0, 1.0),
                },
            ],
            concepts: vec![
                Concept {
                    id: "red".to_string(),
                    label: "Red".to_string(),
                    coordinates: vec![0.0, 1.0],
                    attributes: Default::default(),
                },
            ],
            metadata: ConceptSpaceMetadata {
                created_at: 1234567890,
                updated_at: 1234567900,
                version: "1.0".to_string(),
                algorithm: "t-sne".to_string(),
            },
        };
        
        // Test CBOR
        let cbor = space.to_dag_cbor().unwrap();
        let decoded_cbor: ConceptSpace = DagCborCodec::decode(&cbor).unwrap();
        assert_eq!(space.id, decoded_cbor.id);
        assert_eq!(space.dimensions.len(), decoded_cbor.dimensions.len());
        
        // Test JSON
        let json = space.to_dag_json().unwrap();
        let decoded_json: ConceptSpace = DagJsonCodec::decode(&json).unwrap();
        assert_eq!(space.id, decoded_json.id);
        assert_eq!(space.concepts.len(), decoded_json.concepts.len());
    }
    
    #[test]
    fn test_domain_model_serialization() {
        use types::{DomainModel, Aggregate, Property, DomainModelMetadata};
        
        let model = DomainModel {
            id: "dm-001".to_string(),
            name: "Order Domain".to_string(),
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
                    ],
                    invariants: vec!["Total must be positive".to_string()],
                },
            ],
            events: vec![],
            commands: vec![],
            metadata: DomainModelMetadata {
                version: "1.0".to_string(),
                bounded_context: "order-management".to_string(),
                created_at: 1234567890,
                tags: vec!["core".to_string()],
            },
        };
        
        let json = model.to_dag_json_pretty().unwrap();
        assert!(json.contains("Order Domain"));
        assert!(json.contains("order_id"));
    }
    
    #[test]
    fn test_event_stream_serialization() {
        use types::{EventStream, StreamEvent, EventStreamMetadata};
        use serde_json::json;
        
        let stream = EventStream {
            id: "es-001".to_string(),
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
                        "total": 99.99
                    }),
                    metadata: Default::default(),
                },
            ],
            metadata: EventStreamMetadata {
                created_at: 1234567890,
                last_event_at: 1234567890,
                event_count: 1,
                stream_version: "1.0".to_string(),
            },
        };
        
        let cbor = stream.to_dag_cbor().unwrap();
        let decoded: EventStream = DagCborCodec::decode(&cbor).unwrap();
        assert_eq!(stream.id, decoded.id);
        assert_eq!(stream.events.len(), decoded.events.len());
        assert_eq!(stream.events[0].event_type, decoded.events[0].event_type);
    }
} 