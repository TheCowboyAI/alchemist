//! Git-Nix integration for content-addressed flakes and store paths
//!
//! This module provides the isomorphic mapping between Git's content-addressed
//! storage and Nix's store paths, enabling seamless integration between the two systems.

use crate::{
    value_objects::*,
    events::*,
    commands::*,
    Result, NixDomainError,
};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Mock types until cim-domain-git exports them
/// Represents a Git repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepository {
    /// Path to the repository
    pub path: PathBuf,
    /// Remote URL
    pub url: String,
}

/// Represents a Git commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    /// Commit hash
    pub hash: GitHash,
    /// Commit message
    pub message: String,
    /// Author
    pub author: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Represents a Git hash
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GitHash(String);

impl GitHash {
    /// Create from string
    pub fn from(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    
    /// Convert to string
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl std::fmt::Display for GitHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a Git reference (branch, tag, or commit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRef(String);

impl GitRef {
    /// Create from string
    pub fn from(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// Represents a Git-backed Nix flake with content-addressed properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFlake {
    /// The flake itself
    pub flake: Flake,
    /// Git repository information
    pub git_repo: GitRepository,
    /// Mapping of Git commits to Nix store paths
    pub commit_store_mapping: HashMap<GitHash, StorePath>,
    /// The current Git revision
    pub current_revision: GitHash,
    /// Whether this is a shallow clone
    pub shallow: bool,
}

/// Represents a Nix input that comes from Git
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFlakeInput {
    /// The input name
    pub name: String,
    /// Git URL (e.g., "github:owner/repo", "git+https://...")
    pub url: String,
    /// Optional Git ref (branch, tag, or commit)
    pub git_ref: Option<GitRef>,
    /// Resolved Git hash
    pub resolved_hash: Option<GitHash>,
    /// Corresponding Nix store path
    pub store_path: Option<StorePath>,
    /// Whether to follow the input's flake
    pub follows: Option<String>,
}

/// Maps between Git URLs and Nix flake references
#[derive(Debug, Clone)]
pub struct GitNixMapper {
    /// Known Git forge mappings (e.g., github, gitlab)
    forge_mappings: HashMap<String, ForgeMapping>,
    /// Cache of resolved Git URLs to store paths
    resolution_cache: HashMap<String, StorePath>,
}

#[derive(Debug, Clone)]
struct ForgeMapping {
    /// The forge name (e.g., "github")
    name: String,
    /// URL template for the forge
    url_template: String,
    /// How to construct flake refs
    flake_ref_template: String,
}

impl GitNixMapper {
    /// Create a new mapper with default forge mappings
    pub fn new() -> Self {
        let mut forge_mappings = HashMap::new();
        
        // GitHub mapping
        forge_mappings.insert("github".to_string(), ForgeMapping {
            name: "github".to_string(),
            url_template: "https://github.com/{owner}/{repo}".to_string(),
            flake_ref_template: "github:{owner}/{repo}".to_string(),
        });
        
        // GitLab mapping
        forge_mappings.insert("gitlab".to_string(), ForgeMapping {
            name: "gitlab".to_string(),
            url_template: "https://gitlab.com/{owner}/{repo}".to_string(),
            flake_ref_template: "gitlab:{owner}/{repo}".to_string(),
        });
        
        Self {
            forge_mappings,
            resolution_cache: HashMap::new(),
        }
    }
    
    /// Convert a Git URL to a Nix flake reference
    pub fn git_url_to_flake_ref(&self, git_url: &str) -> Result<FlakeRef> {
        // Check if it's a known forge
        if let Some((forge, owner, repo)) = self.parse_forge_url(git_url) {
            if let Some(mapping) = self.forge_mappings.get(&forge) {
                let flake_ref = mapping.flake_ref_template
                    .replace("{owner}", &owner)
                    .replace("{repo}", &repo);
                
                return Ok(FlakeRef::new(flake_ref));
            }
        }
        
        // Fall back to git+ URL
        Ok(FlakeRef::new(format!("git+{}", git_url)))
    }
    
    /// Parse a Git URL to extract forge, owner, and repo
    fn parse_forge_url(&self, url: &str) -> Option<(String, String, String)> {
        // Simple parser for common Git URLs
        if url.starts_with("https://github.com/") {
            let parts: Vec<&str> = url.trim_start_matches("https://github.com/")
                .trim_end_matches(".git")
                .split('/')
                .collect();
            
            if parts.len() >= 2 {
                return Some(("github".to_string(), parts[0].to_string(), parts[1].to_string()));
            }
        }
        
        if url.starts_with("https://gitlab.com/") {
            let parts: Vec<&str> = url.trim_start_matches("https://gitlab.com/")
                .trim_end_matches(".git")
                .split('/')
                .collect();
            
            if parts.len() >= 2 {
                return Some(("gitlab".to_string(), parts[0].to_string(), parts[1].to_string()));
            }
        }
        
        None
    }
    
    /// Map a Git hash to a Nix store path
    pub async fn git_hash_to_store_path(&self, git_hash: &GitHash) -> Result<StorePath> {
        // Check cache first
        if let Some(cached) = self.resolution_cache.get(&git_hash.to_string()) {
            return Ok(cached.clone());
        }
        
        // Use nix to resolve the Git hash to a store path
        let output = tokio::process::Command::new("nix")
            .args(&[
                "eval",
                "--expr",
                &format!("builtins.fetchGit {{ rev = \"{}\"; }}", git_hash),
                "--raw"
            ])
            .output()
            .await
            .map_err(|e| NixDomainError::CommandError(format!("Failed to resolve Git hash: {}", e)))?;
        
        if !output.status.success() {
            return Err(NixDomainError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        let store_path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        StorePath::parse(&store_path_str)
    }
}

/// Service for managing Git-backed Nix flakes
pub struct GitFlakeService {
    mapper: GitNixMapper,
}

impl GitFlakeService {
    /// Create a new Git flake service
    pub fn new() -> Self {
        Self {
            mapper: GitNixMapper::new(),
        }
    }
    
    /// Create a flake from a Git repository
    pub async fn create_flake_from_git(
        &self,
        git_url: &str,
        target_path: PathBuf,
        description: String,
    ) -> Result<GitFlake> {
        // Clone the Git repository
        let git_repo = self.clone_repository(git_url, &target_path).await?;
        
        // Get the current commit hash
        let current_revision = self.get_current_revision(&target_path).await?;
        
        // Create the flake
        let flake = Flake {
            id: Uuid::new_v4(),
            path: target_path.clone(),
            description,
            inputs: FlakeInputs { inputs: HashMap::new() },
            outputs: FlakeOutputs {
                packages: HashMap::new(),
                dev_shells: HashMap::new(),
                nixos_modules: HashMap::new(),
                overlays: HashMap::new(),
                apps: HashMap::new(),
            },
        };
        
        Ok(GitFlake {
            flake,
            git_repo,
            commit_store_mapping: HashMap::new(),
            current_revision,
            shallow: false,
        })
    }
    
    /// Update flake inputs from Git
    pub async fn update_git_inputs(&self, git_flake: &mut GitFlake) -> Result<Vec<GitFlakeInput>> {
        let mut updated_inputs = Vec::new();
        
        // Read flake.lock if it exists
        let lock_path = git_flake.flake.path.join("flake.lock");
        if lock_path.exists() {
            let lock_content = tokio::fs::read_to_string(&lock_path).await?;
            let lock_data: serde_json::Value = serde_json::from_str(&lock_content)?;
            
            // Parse inputs from lock file
            if let Some(nodes) = lock_data.get("nodes").and_then(|n| n.as_object()) {
                for (name, node) in nodes {
                    if name == "root" {
                        continue;
                    }
                    
                    if let Some(locked) = node.get("locked") {
                        let git_input = self.parse_locked_input(name, locked)?;
                        updated_inputs.push(git_input);
                    }
                }
            }
        }
        
        Ok(updated_inputs)
    }
    
    /// Map FHS paths to Nix store paths
    pub fn map_fhs_to_store(&self, fhs_path: &Path) -> Result<StorePath> {
        // This would implement the mapping logic from FHS to /nix/store
        // For now, we'll use a simple heuristic
        
        let path_str = fhs_path.to_string_lossy();
        
        // Check if it's already a store path
        if path_str.starts_with("/nix/store/") {
            return StorePath::parse(&path_str);
        }
        
        // Otherwise, we need to determine what package provides this path
        // This would typically involve querying Nix's database
        Err(NixDomainError::NotFound(format!("No store path mapping for FHS path: {}", path_str)))
    }
    
    /// Clone a Git repository
    async fn clone_repository(&self, git_url: &str, target_path: &Path) -> Result<GitRepository> {
        let output = tokio::process::Command::new("git")
            .args(&["clone", git_url, target_path.to_str().unwrap()])
            .output()
            .await
            .map_err(|e| NixDomainError::CommandError(format!("Failed to clone repository: {}", e)))?;
        
        if !output.status.success() {
            return Err(NixDomainError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        Ok(GitRepository {
            path: target_path.to_path_buf(),
            url: git_url.to_string(),
        })
    }
    
    /// Get the current Git revision
    async fn get_current_revision(&self, repo_path: &Path) -> Result<GitHash> {
        let output = tokio::process::Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir(repo_path)
            .output()
            .await
            .map_err(|e| NixDomainError::CommandError(format!("Failed to get revision: {}", e)))?;
        
        if !output.status.success() {
            return Err(NixDomainError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(GitHash::from(hash))
    }
    
    /// Parse a locked input from flake.lock
    fn parse_locked_input(&self, name: &str, locked: &serde_json::Value) -> Result<GitFlakeInput> {
        let input_type = locked.get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| NixDomainError::ParseError("Missing input type".to_string()))?;
        
        match input_type {
            "github" => {
                let owner = locked.get("owner").and_then(|o| o.as_str()).unwrap_or("");
                let repo = locked.get("repo").and_then(|r| r.as_str()).unwrap_or("");
                let rev = locked.get("rev").and_then(|r| r.as_str());
                
                Ok(GitFlakeInput {
                    name: name.to_string(),
                    url: format!("github:{}/{}", owner, repo),
                    git_ref: rev.map(|r| GitRef::from(r)),
                    resolved_hash: rev.map(|r| GitHash::from(r)),
                    store_path: None,
                    follows: None,
                })
            }
            "git" => {
                let url = locked.get("url").and_then(|u| u.as_str()).unwrap_or("");
                let rev = locked.get("rev").and_then(|r| r.as_str());
                
                Ok(GitFlakeInput {
                    name: name.to_string(),
                    url: url.to_string(),
                    git_ref: rev.map(|r| GitRef::from(r)),
                    resolved_hash: rev.map(|r| GitHash::from(r)),
                    store_path: None,
                    follows: None,
                })
            }
            _ => Ok(GitFlakeInput {
                name: name.to_string(),
                url: format!("{}:{}", input_type, name),
                git_ref: None,
                resolved_hash: None,
                store_path: None,
                follows: None,
            })
        }
    }
}

/// Commands for Git-Nix integration
#[derive(Debug, Clone)]
pub enum GitNixCommand {
    /// Create a flake from a Git repository
    CreateFlakeFromGit {
        git_url: String,
        target_path: PathBuf,
        description: String,
    },
    /// Update flake inputs from Git
    UpdateGitInputs {
        flake_id: Uuid,
    },
    /// Pin a flake to a specific Git revision
    PinFlakeRevision {
        flake_id: Uuid,
        git_hash: GitHash,
    },
    /// Map an FHS path to a Nix store path
    MapFhsToStore {
        fhs_path: PathBuf,
    },
}

/// Events for Git-Nix integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitNixEvent {
    /// A flake was created from a Git repository
    FlakeCreatedFromGit {
        flake_id: Uuid,
        git_url: String,
        git_hash: GitHash,
        store_path: StorePath,
        timestamp: DateTime<Utc>,
    },
    /// Git inputs were updated
    GitInputsUpdated {
        flake_id: Uuid,
        inputs: Vec<GitFlakeInput>,
        timestamp: DateTime<Utc>,
    },
    /// A flake was pinned to a Git revision
    FlakePinnedToRevision {
        flake_id: Uuid,
        git_hash: GitHash,
        store_path: StorePath,
        timestamp: DateTime<Utc>,
    },
    /// An FHS path was mapped to a store path
    FhsMappedToStore {
        fhs_path: PathBuf,
        store_path: StorePath,
        timestamp: DateTime<Utc>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_git_url_to_flake_ref() {
        let mapper = GitNixMapper::new();
        
        // Test GitHub URL
        let flake_ref = mapper.git_url_to_flake_ref("https://github.com/NixOS/nixpkgs").unwrap();
        assert_eq!(flake_ref.uri, "github:NixOS/nixpkgs");
        
        // Test GitLab URL
        let flake_ref = mapper.git_url_to_flake_ref("https://gitlab.com/example/project").unwrap();
        assert_eq!(flake_ref.uri, "gitlab:example/project");
        
        // Test generic Git URL
        let flake_ref = mapper.git_url_to_flake_ref("https://example.com/repo.git").unwrap();
        assert_eq!(flake_ref.uri, "git+https://example.com/repo.git");
    }
    
    #[test]
    fn test_parse_forge_url() {
        let mapper = GitNixMapper::new();
        
        // Test GitHub parsing
        let result = mapper.parse_forge_url("https://github.com/NixOS/nixpkgs");
        assert_eq!(result, Some(("github".to_string(), "NixOS".to_string(), "nixpkgs".to_string())));
        
        // Test with .git suffix
        let result = mapper.parse_forge_url("https://github.com/owner/repo.git");
        assert_eq!(result, Some(("github".to_string(), "owner".to_string(), "repo".to_string())));
    }
} 