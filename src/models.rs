use std::collections::HashMap;
use uuid::Uuid;

/// Data structure for nodes in the Snarl graph
#[derive(Debug, Clone)]
pub struct GraphNodeData {
    pub uuid: Uuid,
    pub name: String,
    pub properties: HashMap<String, String>,
    pub labels: Vec<String>,
    pub radius: f32,
}
