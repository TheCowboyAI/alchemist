//! Git version control projection and ingestion

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitConfig {
    pub repos: Vec<String>,
    pub webhook_secret: Option<String>,
    pub watch_branches: Vec<String>,
    pub include_commits: bool,
    pub include_issues: bool,
    pub include_pull_requests: bool,
}
