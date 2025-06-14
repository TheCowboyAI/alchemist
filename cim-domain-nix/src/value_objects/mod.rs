//! Value objects for the Nix domain
//!
//! This module contains immutable value objects that represent
//! core concepts in the Nix ecosystem.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// A reference to a Nix flake
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlakeRef {
    /// The URI of the flake (e.g., "github:owner/repo")
    pub uri: String,
    /// Optional revision/branch/tag
    pub revision: Option<String>,
    /// Optional subflake path
    pub subflake: Option<String>,
}

impl FlakeRef {
    /// Create a new flake reference
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            revision: None,
            subflake: None,
        }
    }

    /// Set the revision
    pub fn with_revision(mut self, revision: impl Into<String>) -> Self {
        self.revision = Some(revision.into());
        self
    }

    /// Set the subflake
    pub fn with_subflake(mut self, subflake: impl Into<String>) -> Self {
        self.subflake = Some(subflake.into());
        self
    }

    /// Convert to a Nix flake reference string
    pub fn to_nix_string(&self) -> String {
        let mut result = self.uri.clone();
        
        if let Some(rev) = &self.revision {
            result.push('/');
            result.push_str(rev);
        }
        
        if let Some(sub) = &self.subflake {
            result.push('#');
            result.push_str(sub);
        }
        
        result
    }
}

/// An attribute path in Nix (e.g., "packages.x86_64-linux.hello")
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributePath {
    /// The segments of the path
    pub segments: Vec<String>,
}

impl AttributePath {
    /// Create a new attribute path from segments
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    /// Create from a dot-separated string
    pub fn from_str(path: &str) -> Self {
        Self {
            segments: path.split('.').map(String::from).collect(),
        }
    }

    /// Convert to a dot-separated string
    pub fn to_string(&self) -> String {
        self.segments.join(".")
    }
}

impl std::fmt::Display for AttributePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.segments.join("."))
    }
}

/// A Nix derivation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Derivation {
    /// The derivation path
    pub drv_path: PathBuf,
    /// The output paths
    pub outputs: HashMap<String, PathBuf>,
    /// The system (e.g., "x86_64-linux")
    pub system: String,
    /// The builder command
    pub builder: String,
    /// Build arguments
    pub args: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
}

/// A NixOS module
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixModule {
    /// Module ID
    pub id: Uuid,
    /// Module name
    pub name: String,
    /// Module options
    pub options: HashMap<String, serde_json::Value>,
    /// Module config
    pub config: serde_json::Value,
    /// Module imports
    pub imports: Vec<PathBuf>,
}

/// An overlay for Nix packages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Overlay {
    /// Overlay ID
    pub id: Uuid,
    /// Overlay name
    pub name: String,
    /// Packages to override
    pub overrides: HashMap<String, serde_json::Value>,
    /// New packages to add
    pub additions: HashMap<String, serde_json::Value>,
}

/// A NixOS system configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixOSConfiguration {
    /// Configuration ID
    pub id: Uuid,
    /// Configuration name
    pub name: String,
    /// System architecture
    pub system: String,
    /// Configuration file path
    pub path: PathBuf,
    /// Hostname
    pub hostname: String,
    /// Imported modules
    pub modules: Vec<PathBuf>,
    /// Applied overlays
    pub overlays: Vec<String>,
    /// System packages
    pub packages: Vec<String>,
    /// Specialisations
    pub specialisations: HashMap<String, serde_json::Value>,
}

/// A parsed Nix store path
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorePath {
    /// The hash part of the store path
    pub hash: String,
    /// The name part of the store path
    pub name: String,
}

impl StorePath {
    /// Parse a store path string
    pub fn parse(path: &str) -> Result<Self, String> {
        let path_buf = PathBuf::from(path);
        
        // Check if it's in the Nix store
        if !path.starts_with("/nix/store/") {
            return Err("Not a valid Nix store path".to_string());
        }

        // Get the last component
        let file_name = path_buf
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("Invalid path")?;

        // Split on the first dash
        let parts: Vec<&str> = file_name.splitn(2, '-').collect();
        if parts.len() != 2 {
            return Err("Invalid store path format".to_string());
        }

        Ok(Self {
            hash: parts[0].to_string(),
            name: parts[1].to_string(),
        })
    }

    /// Convert back to a string
    pub fn to_string(&self) -> String {
        format!("/nix/store/{}-{}", self.hash, self.name)
    }
}

impl std::fmt::Display for StorePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "/nix/store/{}-{}", self.hash, self.name)
    }
}

/// Flake inputs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlakeInputs {
    /// Map of input name to flake reference
    pub inputs: HashMap<String, FlakeRef>,
}

/// Flake outputs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlakeOutputs {
    /// Packages by system and name
    pub packages: HashMap<String, HashMap<String, AttributePath>>,
    /// Development shells by system and name
    pub dev_shells: HashMap<String, HashMap<String, AttributePath>>,
    /// NixOS modules
    pub nixos_modules: HashMap<String, AttributePath>,
    /// Overlays
    pub overlays: HashMap<String, AttributePath>,
    /// Apps by system and name
    pub apps: HashMap<String, HashMap<String, AttributePath>>,
}

/// A Nix flake
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flake {
    /// Flake ID
    pub id: Uuid,
    /// Flake path
    pub path: PathBuf,
    /// Flake description
    pub description: String,
    /// Flake inputs
    pub inputs: FlakeInputs,
    /// Flake outputs
    pub outputs: FlakeOutputs,
}

/// A Nix expression
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixExpression {
    /// The expression text
    pub text: String,
    /// Whether this expression is pure
    pub is_pure: bool,
} 