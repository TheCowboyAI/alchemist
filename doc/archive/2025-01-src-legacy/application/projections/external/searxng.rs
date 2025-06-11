//! SearXNG federated search projection and ingestion

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearxngConfig {
    pub instance_url: String,
    pub api_key: Option<String>,
    pub index_content: bool,
    pub index_metadata: bool,
}
