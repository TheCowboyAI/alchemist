//! Neo4j graph database projection and ingestion

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Neo4jConfig {
    pub uri: String,
    pub username: String,
    pub password: String,
    pub database: Option<String>,
    pub batch_size: usize,
    pub flush_interval_secs: u64,
}
