//! Email notification projection

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub from_address: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub digest_interval_secs: u64,
}
