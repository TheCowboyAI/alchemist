//! n8n workflow automation projection and ingestion

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct N8nConfig {
    pub webhook_url: String,
    pub api_key: Option<String>,
    pub webhook_port: u16,
}
