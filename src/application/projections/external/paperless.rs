//! Paperless-NGx document management projection and ingestion

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaperlessConfig {
    pub api_url: String,
    pub api_token: String,
    pub auto_tag: bool,
    pub link_documents: bool,
}
