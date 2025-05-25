use bevy::prelude::*;
use std::collections::HashMap;

/// Component for storing arbitrary metadata
#[derive(Component, Default)]
pub struct Metadata {
    pub tags: Vec<String>,
    pub properties: HashMap<String, String>,
    pub created_at: f64,
    pub modified_at: f64,
}

/// Component for domain-specific labels
#[derive(Component, Default)]
pub struct Labels {
    pub primary: String,
    pub secondary: Vec<String>,
}

/// Component for node/edge descriptions
#[derive(Component, Default)]
pub struct Description {
    pub short: String,
    pub long: String,
}

/// Component for version tracking
#[derive(Component)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub revision: String,
}

impl Default for Version {
    fn default() -> Self {
        Self {
            major: 0,
            minor: 1,
            patch: 0,
            revision: String::new(),
        }
    }
}
