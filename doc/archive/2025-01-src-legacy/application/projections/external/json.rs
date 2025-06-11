//! JSON file export projection

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonConfig {
    pub output_dir: String,
    pub formats: Vec<String>,
    pub compression: Option<String>,
}
