use crate::domain::{
    commands::{GraphCommand, WorkflowCommand, ImportSource, ImportOptions},
    value_objects::{GraphId, NodeId, EdgeId, Position3D, WorkflowId, StepId, UserId},
    DomainError,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Service for importing graphs from various formats
pub struct GraphImportService;

/// Supported import formats
#[derive(Debug, Clone, PartialEq)]
pub enum ImportFormat {
    ArrowsApp,
    Cypher,
    Mermaid,
    Dot,
    ProgressJson,
    VocabularyJson,
    RssAtom,  // RSS/Atom feeds (EventStore format)
}

/// Imported graph data structure
#[derive(Debug, Clone)]
pub struct ImportedGraph {
    pub nodes: Vec<ImportedNode>,
    pub edges: Vec<ImportedEdge>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ImportedNode {
    pub id: String,
    pub node_type: String,
    pub label: String,
    pub position: Position3D,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Imported edge structure
#[derive(Debug, Clone)]
pub struct ImportedEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Arrows.app JSON format structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArrowsAppJson {
    nodes: Vec<ArrowsAppNode>,
    relationships: Vec<ArrowsAppRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArrowsAppPosition {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArrowsAppNode {
    id: String,
    position: ArrowsAppPosition,
    caption: Option<String>,
    labels: Vec<String>,
    properties: HashMap<String, serde_json::Value>,
    style: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArrowsAppRelationship {
    id: String,
    #[serde(rename = "fromId")]
    from_id: String,
    #[serde(rename = "toId")]
    to_id: String,
    #[serde(rename = "type")]
    rel_type: String,
    properties: HashMap<String, serde_json::Value>,
    style: HashMap<String, String>,
}

/// Progress.json format structures
#[derive(Debug, Deserialize)]
struct ProgressJson {
    metadata: ProgressMetadata,
    nodes: Vec<ProgressNode>,
    edges: Vec<ProgressEdge>,
}

#[derive(Debug, Deserialize)]
struct ProgressMetadata {
    name: String,
    description: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct ProgressNode {
    id: String,
    label: String,
    #[serde(rename = "type")]
    node_type: String,
    position: ProgressPosition,
    data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ProgressPosition {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Deserialize)]
struct ProgressEdge {
    id: String,
    source: String,
    target: String,
    #[serde(rename = "type")]
    edge_type: String,
    label: Option<String>,
}

/// Vocabulary graph format
#[derive(Debug, Deserialize)]
struct VocabularyJson {
    metadata: VocabularyMetadata,
    categories: Vec<VocabularyCategory>,
    terms: Vec<VocabularyTerm>,
}

#[derive(Debug, Deserialize)]
struct VocabularyMetadata {
    name: String,
    description: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct VocabularyCategory {
    id: String,
    name: String,
    description: Option<String>,
    order: i32,
    subcategories: Option<Vec<VocabularySubcategory>>,
}

#[derive(Debug, Deserialize)]
struct VocabularySubcategory {
    id: String,
    name: String,
    order: i32,
}

#[derive(Debug, Deserialize)]
struct VocabularyTerm {
    id: String,
    name: String,
    category: String,
    subcategory: Option<String>,
    #[serde(rename = "type")]
    term_type: Option<String>,
    definition: String,
    relationships: Option<HashMap<String, Vec<String>>>,
}

// RSS/Atom feed structures (EventStore format)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "feed")]
struct AtomFeed {
    #[serde(rename = "@xmlns", default)]
    xmlns: String,
    id: String,
    title: String,
    updated: String,
    author: Option<AtomAuthor>,
    #[serde(rename = "link", default)]
    links: Vec<AtomLink>,
    #[serde(rename = "entry", default)]
    entries: Vec<AtomEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AtomAuthor {
    name: String,
    uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AtomLink {
    #[serde(rename = "@rel")]
    rel: String,
    #[serde(rename = "@href")]
    href: String,
    #[serde(rename = "@type")]
    link_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AtomEntry {
    id: String,
    title: String,
    updated: String,
    author: Option<AtomAuthor>,
    summary: Option<String>,
    content: Option<AtomContent>,
    #[serde(rename = "link", default)]
    links: Vec<AtomLink>,
    // EventStore specific fields
    #[serde(rename = "eventType", default)]
    event_type: Option<String>,
    #[serde(rename = "eventNumber", default)]
    event_number: Option<i64>,
    #[serde(rename = "streamId", default)]
    stream_id: Option<String>,
    #[serde(rename = "isJson", default)]
    is_json: Option<bool>,
    #[serde(rename = "isMetaData", default)]
    is_metadata: Option<bool>,
    #[serde(rename = "isLinkMetaData", default)]
    is_link_metadata: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AtomContent {
    #[serde(rename = "@type")]
    content_type: String,
    #[serde(rename = "$value")]
    value: String,
}

// RSS 2.0 structures
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "rss")]
struct RssFeed {
    #[serde(rename = "@version", default)]
    version: String,
    channel: RssChannel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RssChannel {
    title: String,
    link: String,
    description: String,
    #[serde(rename = "lastBuildDate", default)]
    last_build_date: Option<String>,
    #[serde(rename = "pubDate", default)]
    pub_date: Option<String>,
    #[serde(rename = "item", default)]
    items: Vec<RssItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RssItem {
    title: String,
    link: Option<String>,
    description: Option<String>,
    #[serde(rename = "pubDate", default)]
    pub_date: Option<String>,
    guid: Option<String>,
    #[serde(rename = "category", default)]
    categories: Vec<String>,
    author: Option<String>,
}

/// Import mapping configuration for field and value transformations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportMapping {
    /// Field name mappings (e.g., "text" -> "content", "name" -> "label")
    pub field_mappings: HashMap<String, String>,

    /// Value transformations (e.g., uppercase, date parsing)
    pub value_transforms: HashMap<String, ValueTransform>,

    /// Relationship type mappings (e.g., "KNOWS" -> "knows", "HAS_A" -> "contains")
    pub relationship_mappings: HashMap<String, String>,

    /// Node type mappings (e.g., "Person" -> "Actor", "Company" -> "Organization")
    pub node_type_mappings: HashMap<String, String>,

    /// Layout configuration
    pub layout_config: LayoutConfig,

    /// Custom extractors for complex mappings
    pub custom_extractors: HashMap<String, ExtractorConfig>,
}

impl Default for ImportMapping {
    fn default() -> Self {
        Self {
            field_mappings: Self::default_field_mappings(),
            value_transforms: HashMap::new(),
            relationship_mappings: Self::default_relationship_mappings(),
            node_type_mappings: HashMap::new(),
            layout_config: LayoutConfig::default(),
            custom_extractors: HashMap::new(),
        }
    }
}

impl ImportMapping {
    fn default_field_mappings() -> HashMap<String, String> {
        let mut mappings = HashMap::new();
        // Common field mappings
        mappings.insert("text".to_string(), "content".to_string());
        mappings.insert("name".to_string(), "label".to_string());
        mappings.insert("id".to_string(), "node_id".to_string());
        mappings.insert("type".to_string(), "node_type".to_string());
        mappings.insert("description".to_string(), "summary".to_string());
        mappings
    }

    fn default_relationship_mappings() -> HashMap<String, String> {
        let mut mappings = HashMap::new();
        // Common relationship mappings
        mappings.insert("KNOWS".to_string(), "knows".to_string());
        mappings.insert("HAS_A".to_string(), "contains".to_string());
        mappings.insert("IS_A".to_string(), "extends".to_string());
        mappings.insert("PART_OF".to_string(), "belongs_to".to_string());
        mappings
    }

    /// Apply field mapping
    pub fn map_field(&self, field: &str) -> String {
        self.field_mappings.get(field)
            .cloned()
            .unwrap_or_else(|| field.to_string())
    }

    /// Apply value transformation
    pub fn transform_value(&self, field: &str, value: serde_json::Value) -> serde_json::Value {
        if let Some(transform) = self.value_transforms.get(field) {
            transform.apply(value)
        } else {
            value
        }
    }

    /// Map relationship type
    pub fn map_relationship(&self, rel_type: &str) -> String {
        self.relationship_mappings.get(rel_type)
            .cloned()
            .unwrap_or_else(|| rel_type.to_lowercase())
    }

    /// Map node type
    pub fn map_node_type(&self, node_type: &str) -> String {
        self.node_type_mappings.get(node_type)
            .cloned()
            .unwrap_or_else(|| node_type.to_string())
    }
}

/// Value transformation types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueTransform {
    /// Convert to uppercase
    Uppercase,
    /// Convert to lowercase
    Lowercase,
    /// Parse as date
    ParseDate { format: String },
    /// Extract number from string
    ExtractNumber,
    /// Apply regex extraction
    Regex { pattern: String, group: usize },
    /// Mathematical operation
    Math { operation: MathOperation },
    /// Custom JavaScript-like expression
    Expression { expr: String },
}

impl ValueTransform {
    fn apply(&self, value: serde_json::Value) -> serde_json::Value {
        match self {
            ValueTransform::Uppercase => {
                if let serde_json::Value::String(s) = value {
                    serde_json::Value::String(s.to_uppercase())
                } else {
                    value
                }
            }
            ValueTransform::Lowercase => {
                if let serde_json::Value::String(s) = value {
                    serde_json::Value::String(s.to_lowercase())
                } else {
                    value
                }
            }
            ValueTransform::ExtractNumber => {
                if let serde_json::Value::String(s) = &value {
                    // Extract first number from string
                    let numbers: String = s.chars()
                        .filter(|c| c.is_numeric() || *c == '.' || *c == '-')
                        .collect();
                    if let Ok(num) = numbers.parse::<f64>() {
                        serde_json::json!(num)
                    } else {
                        value
                    }
                } else {
                    value
                }
            }
            ValueTransform::Math { operation } => {
                if let serde_json::Value::Number(n) = &value {
                    if let Some(num) = n.as_f64() {
                        let result = operation.apply(num);
                        serde_json::json!(result)
                    } else {
                        value
                    }
                } else {
                    value
                }
            }
            _ => value, // TODO: Implement other transformations
        }
    }
}

/// Mathematical operations for value transformation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MathOperation {
    Add(f64),
    Multiply(f64),
    Power(f64),
    Logarithm,
    Normalize { min: f64, max: f64 },
}

impl MathOperation {
    fn apply(&self, value: f64) -> f64 {
        match self {
            MathOperation::Add(n) => value + n,
            MathOperation::Multiply(n) => value * n,
            MathOperation::Power(n) => value.powf(*n),
            MathOperation::Logarithm => value.ln(),
            MathOperation::Normalize { min, max } => {
                (value - min) / (max - min)
            }
        }
    }
}

/// Layout configuration for imported graphs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub algorithm: LayoutAlgorithm,
    pub parameters: LayoutParameters,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            algorithm: LayoutAlgorithm::ForceDirected,
            parameters: LayoutParameters::default(),
        }
    }
}

/// Layout algorithms for graph visualization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LayoutAlgorithm {
    /// Preserve original positions from import
    None,
    /// Force-directed layout with configurable forces
    ForceDirected,
    /// Circular layout
    Circular,
    /// Hierarchical layout
    Hierarchical,
    /// Grid layout
    Grid,
    /// Random layout
    Random,
    /// Custom layout based on node properties
    PropertyBased { property: String },
}

/// Parameters for layout algorithms
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutParameters {
    /// Force-directed parameters
    pub repulsion_force: f32,
    pub attraction_force: f32,
    pub center_force: f32,
    pub damping: f32,

    /// General parameters
    pub spacing: f32,
    pub margin: f32,
    pub iterations: u32,

    /// Property-based layout parameters
    pub property_scale: HashMap<String, f32>,
}

impl Default for LayoutParameters {
    fn default() -> Self {
        Self {
            repulsion_force: 100.0,
            attraction_force: 0.1,
            center_force: 0.01,
            damping: 0.9,
            spacing: 100.0,
            margin: 50.0,
            iterations: 100,
            property_scale: HashMap::new(),
        }
    }
}

/// Configuration for custom value extraction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractorConfig {
    /// Path to extract from (e.g., "data.attributes.name")
    pub path: String,
    /// Default value if extraction fails
    pub default: Option<serde_json::Value>,
    /// Post-extraction transform
    pub transform: Option<ValueTransform>,
}

impl Default for GraphImportService {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphImportService {
    pub fn new() -> Self {
        Self
    }

    /// Import a graph from various sources
    pub async fn import_from_source(
        &self,
        source: &ImportSource,
        format: ImportFormat,
    ) -> Result<ImportedGraph, DomainError> {
        match source {
            ImportSource::File { path } => {
                let content = self.read_file_content(path)?;
                self.import_from_content(&content, format, None)
            }
            ImportSource::Url { url } => {
                let content = self.fetch_url_content(url).await?;
                self.import_from_content(&content, format, None)
            }
            ImportSource::GitRepository { url, branch, path } => {
                let content = self.fetch_git_file(url, branch.as_deref(), path).await?;
                self.import_from_content(&content, format, None)
            }
            ImportSource::NixFlake { flake_ref, output } => {
                let content = self.evaluate_nix_flake(flake_ref, output).await?;
                self.import_from_content(&content, format, None)
            }
            ImportSource::InlineContent { content } => {
                self.import_from_content(content, format, None)
            }
        }
    }

    /// Import from content string based on format
    pub fn import_from_content(
        &self,
        content: &str,
        format: ImportFormat,
        mapping: Option<&ImportMapping>,
    ) -> Result<ImportedGraph, DomainError> {
        // Use provided mapping or default
        let mapping = mapping.cloned().unwrap_or_default();

        let mut imported_graph = match format {
            ImportFormat::ArrowsApp => self.import_arrows_app(content, &mapping)?,
            ImportFormat::Cypher => self.import_cypher(content, &mapping)?,
            ImportFormat::Mermaid => self.import_mermaid(content)?,
            ImportFormat::Dot => self.import_dot(content, &mapping)?,
            ImportFormat::ProgressJson => self.import_progress_json(content, &mapping)?,
            ImportFormat::VocabularyJson => self.import_vocabulary_json(content, &mapping)?,
            ImportFormat::RssAtom => self.import_rss_atom(content, &mapping)?,
        };

        // Apply layout algorithm
        self.apply_layout(&mut imported_graph, &mapping.layout_config)?;

        Ok(imported_graph)
    }

    /// Read file content from filesystem
    fn read_file_content(&self, path: &str) -> Result<String, DomainError> {
        std::fs::read_to_string(path)
            .map_err(|e| DomainError::ValidationFailed(format!("Failed to read file {path}: {e}")))
    }

    /// Fetch content from URL
    async fn fetch_url_content(&self, url: &str) -> Result<String, DomainError> {
        // In a real implementation, use reqwest or similar
        // For now, return a placeholder
        Err(DomainError::ValidationFailed(
            "URL fetching not yet implemented".to_string()
        ))
    }

    /// Fetch file from Git repository
    async fn fetch_git_file(
        &self,
        repo_url: &str,
        branch: Option<&str>,
        file_path: &str,
    ) -> Result<String, DomainError> {
        // In a real implementation, we would:
        // 1. Clone the repository to a temp directory
        // 2. Checkout the specified branch
        // 3. Read the file at the given path

        // For now, we'll use a simple git command approach
        let temp_dir = std::env::temp_dir().join(format!("graph_import_{}", Uuid::new_v4()));

        // Clone the repository
        let clone_result = std::process::Command::new("git")
            .args(["clone", "--depth", "1"])
            .args(if let Some(b) = branch {
                vec!["--branch", b]
            } else {
                vec![]
            })
            .arg(repo_url)
            .arg(&temp_dir)
            .output()
            .map_err(|e| DomainError::ValidationFailed(format!("Failed to clone repository: {e}")))?;

        if !clone_result.status.success() {
            let error = String::from_utf8_lossy(&clone_result.stderr);
            return Err(DomainError::ValidationFailed(format!("Git clone failed: {error}")));
        }

        // Read the file
        let file_path = temp_dir.join(file_path);
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| DomainError::ValidationFailed(format!("Failed to read file from repository: {e}")))?;

        // Clean up temp directory
        let _ = std::fs::remove_dir_all(&temp_dir);

        Ok(content)
    }

    /// Evaluate Nix flake to get graph content
    async fn evaluate_nix_flake(
        &self,
        flake_ref: &str,
        output: &str,
    ) -> Result<String, DomainError> {
        // Use nix eval to get the content
        let eval_result = std::process::Command::new("nix")
            .args(["eval", "--raw"])
            .arg(format!("{flake_ref}#{output}"))
            .output()
            .map_err(|e| DomainError::ValidationFailed(format!("Failed to evaluate Nix flake: {e}")))?;

        if !eval_result.status.success() {
            let error = String::from_utf8_lossy(&eval_result.stderr);
            return Err(DomainError::ValidationFailed(format!("Nix evaluation failed: {error}")));
        }

        Ok(String::from_utf8_lossy(&eval_result.stdout).to_string())
    }

    /// Apply import options to the imported graph
    pub fn apply_import_options(
        &self,
        mut graph: ImportedGraph,
        options: &ImportOptions,
    ) -> ImportedGraph {
        // Apply node ID prefix
        if let Some(prefix) = &options.id_prefix {
            for node in &mut graph.nodes {
                node.id = format!("{}-{}", prefix, node.id);
            }
            for edge in &mut graph.edges {
                edge.source = format!("{}-{}", prefix, edge.source);
                edge.target = format!("{}-{}", prefix, edge.target);
            }
        }

        // Apply position offset
        if let Some(offset) = &options.position_offset {
            for node in &mut graph.nodes {
                node.position.x += offset.x;
                node.position.y += offset.y;
                node.position.z += offset.z;
            }
        }

        graph
    }

    /// Import multiple graphs from a directory
    pub async fn import_directory(
        &self,
        dir_path: &str,
        format: ImportFormat,
        recursive: bool,
    ) -> Result<Vec<ImportedGraph>, DomainError> {
        let mut graphs = Vec::new();
        let entries = std::fs::read_dir(dir_path)
            .map_err(|e| DomainError::ValidationFailed(format!("Failed to read directory: {e}")))?;

        for entry in entries {
            let entry = entry.map_err(|e| DomainError::ValidationFailed(format!("Failed to read entry: {e}")))?;
            let path = entry.path();

            if path.is_file() && self.is_supported_file(&path, &format) {
                let content = self.read_file_content(path.to_str().unwrap())?;
                if let Ok(graph) = self.import_from_content(&content, format.clone(), None) {
                    graphs.push(graph);
                }
            } else if recursive && path.is_dir() {
                let sub_graphs = Box::pin(self.import_directory(
                    path.to_str().unwrap(),
                    format.clone(),
                    recursive,
                )).await?;
                graphs.extend(sub_graphs);
            }
        }

        Ok(graphs)
    }

    /// Check if file extension matches the format
    fn is_supported_file(&self, path: &Path, format: &ImportFormat) -> bool {
        let extension = path.extension().and_then(|e| e.to_str());

        match (format, extension) {
            (ImportFormat::ArrowsApp | ImportFormat::ProgressJson | ImportFormat::VocabularyJson, Some("json")) => true,
            (ImportFormat::Cypher, Some("cypher") | Some("cql")) => true,
            (ImportFormat::Mermaid, Some("mmd") | Some("mermaid") | Some("md")) => true,
            (ImportFormat::Dot, Some("dot") | Some("gv")) => true,
            _ => false,
        }
    }

    /// Import from a Nix flake that provides multiple graphs
    pub async fn import_from_nix_graphs_flake(
        &self,
        flake_ref: &str,
    ) -> Result<Vec<ImportedGraph>, DomainError> {
        // List available graphs in the flake
        let list_result = std::process::Command::new("nix")
            .args(["eval", "--json"])
            .arg(format!("{flake_ref}#graphs"))
            .output()
            .map_err(|e| DomainError::ValidationFailed(format!("Failed to list graphs from flake: {e}")))?;

        if !list_result.status.success() {
            let error = String::from_utf8_lossy(&list_result.stderr);
            return Err(DomainError::ValidationFailed(format!("Failed to list graphs: {error}")));
        }

        // Parse the list of available graphs
        let graphs_list: HashMap<String, serde_json::Value> = serde_json::from_slice(&list_result.stdout)
            .map_err(|e| DomainError::ValidationFailed(format!("Failed to parse graphs list: {e}")))?;

        let mut imported_graphs = Vec::new();

        // Import each graph
        for (name, _) in graphs_list {
            let graph_result = std::process::Command::new("nix")
                .args(["eval", "--json"])
                .arg(format!("{flake_ref}#graphs.{name}"))
                .output()
                .map_err(|e| DomainError::ValidationFailed(format!("Failed to evaluate graph {name}: {e}")))?;

            if graph_result.status.success() {
                let content = String::from_utf8_lossy(&graph_result.stdout);

                // Try to detect format from the content
                let format = self.detect_format(&content);

                if let Ok(mut graph) = self.import_from_content(&content, format, None) {
                    // Add flake metadata
                    graph.metadata.insert("nix_flake_ref".to_string(), serde_json::json!(flake_ref));
                    graph.metadata.insert("nix_graph_name".to_string(), serde_json::json!(name));
                    imported_graphs.push(graph);
                }
            }
        }

        Ok(imported_graphs)
    }

    /// Auto-detect format from content
    pub fn detect_format(&self, content: &str) -> ImportFormat {
        let trimmed = content.trim();

        // Check for JSON formats first
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            if content.contains("\"nodes\"") && content.contains("\"relationships\"") {
                return ImportFormat::ArrowsApp;
            }
            if content.contains("\"metadata\"") && content.contains("\"nodes\"") && content.contains("\"edges\"") {
                return ImportFormat::ProgressJson;
            }
            if content.contains("\"categories\"") && content.contains("\"terms\"") {
                return ImportFormat::VocabularyJson;
            }
        }

        // Check for XML formats (RSS/Atom)
        if trimmed.starts_with("<?xml") || trimmed.starts_with("<feed") || trimmed.starts_with("<rss") {
            if content.contains("<feed") && content.contains("xmlns") {
                return ImportFormat::RssAtom;
            }
            if content.contains("<rss") || content.contains("<channel>") {
                return ImportFormat::RssAtom;
            }
        }

        // Check for Cypher
        if content.contains("CREATE") && (content.contains("(") || content.contains(")-[")) {
            return ImportFormat::Cypher;
        }

        // Check for Mermaid
        if content.contains("graph") || content.contains("flowchart") || content.contains("stateDiagram") {
            return ImportFormat::Mermaid;
        }

        // Check for DOT
        if content.contains("digraph") || (content.contains("graph") && content.contains("{") && content.contains("}")) {
            return ImportFormat::Dot;
        }

        // Default to Cypher if unclear
        ImportFormat::Cypher
    }

    /// Import graph definitions from current Git repository
    pub async fn import_from_current_repo(
        &self,
        patterns: Vec<&str>,
    ) -> Result<Vec<ImportedGraph>, DomainError> {
        let mut graphs = Vec::new();

        // Get repository root
        let repo_root = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .map_err(|e| DomainError::ValidationFailed(format!("Failed to find git root: {e}")))?;

        if !repo_root.status.success() {
            return Err(DomainError::ValidationFailed("Not in a git repository".to_string()));
        }

        let root_path = String::from_utf8_lossy(&repo_root.stdout).trim().to_string();

        // Find files matching patterns
        for pattern in patterns {
            let find_result = std::process::Command::new("find")
                .current_dir(&root_path)
                .args([".", "-name", pattern, "-type", "f"])
                .output()
                .map_err(|e| DomainError::ValidationFailed(format!("Failed to find files: {e}")))?;

            if find_result.status.success() {
                let files = String::from_utf8_lossy(&find_result.stdout);

                for file_path in files.lines() {
                    if !file_path.is_empty() {
                        let full_path = PathBuf::from(&root_path).join(file_path);
                        if let Ok(content) = self.read_file_content(full_path.to_str().unwrap()) {
                            let format = self.detect_format(&content);

                            if let Ok(mut graph) = self.import_from_content(&content, format, None) {
                                // Add git metadata
                                graph.metadata.insert("git_path".to_string(), serde_json::json!(file_path));
                                graph.metadata.insert("git_root".to_string(), serde_json::json!(root_path));

                                // Use filename as graph name if not set
                                if graph.metadata.get("name").is_none() {
                                    if let Some(filename) = Path::new(file_path).file_stem() {
                                        graph.metadata.insert("name".to_string(), serde_json::json!(filename.to_string_lossy()));
                                    }
                                }

                                graphs.push(graph);
                            }
                        }
                    }
                }
            }
        }

        Ok(graphs)
    }

    /// Import a graph from JSON content
    pub fn import_from_json(
        &self,
        content: &str,
        format: ImportFormat,
    ) -> Result<ImportedGraph, DomainError> {
        match format {
            ImportFormat::ArrowsApp => self.import_arrows_app(content, &ImportMapping::default()),
            ImportFormat::ProgressJson => self.import_progress_json(content, &ImportMapping::default()),
            ImportFormat::VocabularyJson => self.import_vocabulary_json(content, &ImportMapping::default()),
            _ => Err(DomainError::ValidationFailed(
                "JSON import not supported for this format".to_string(),
            )),
        }
    }

    /// Import a graph from text content (Cypher, Mermaid, DOT)
    pub fn import_from_text(
        &self,
        content: &str,
        format: ImportFormat,
    ) -> Result<ImportedGraph, DomainError> {
        match format {
            ImportFormat::Cypher => self.import_cypher(content, &ImportMapping::default()),
            ImportFormat::Mermaid => self.import_mermaid(content),
            ImportFormat::Dot => self.import_dot(content, &ImportMapping::default()),
            _ => Err(DomainError::ValidationFailed(
                format!("Text import not supported for format: {:?}", format)
            ))
        }
    }

    /// Convert imported graph to domain commands
    pub fn to_graph_commands(&self, imported: &ImportedGraph) -> Vec<GraphCommand> {
        let mut commands = vec![
            GraphCommand::CreateGraph {
                id: GraphId::new(),
                name: imported.metadata.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Imported graph")
                    .to_string(),
                metadata: imported.metadata.clone(),
            },
        ];

        // Add nodes
        for node in &imported.nodes {
            commands.push(GraphCommand::AddNode {
                graph_id: GraphId::new(),
                node_id: NodeId::new(),
                node_type: node.node_type.clone(),
                position: node.position,
                content: serde_json::json!({
                    "label": node.label,
                    "properties": node.properties,
                }),
            });
        }

        // Add edges
        for edge in &imported.edges {
            // Find node IDs by original IDs
            // In real implementation, we'd maintain a mapping
            commands.push(GraphCommand::ConnectNodes {
                graph_id: GraphId::new(),
                edge_id: EdgeId::new(),
                source_id: NodeId::new(), // Would map from edge.source
                target_id: NodeId::new(), // Would map from edge.target
                edge_type: edge.edge_type.clone(),
                properties: edge.properties.clone(),
            });
        }

        commands
    }

    /// Convert imported graph to workflow commands if it represents a workflow
    pub fn to_workflow_commands(&self, imported: &ImportedGraph) -> Option<Vec<WorkflowCommand>> {
        // Check if this is a workflow graph
        let is_workflow = imported.nodes.iter().any(|n| {
            matches!(n.node_type.as_str(), "WorkflowStep" | "Decision" | "Start" | "End" | "FlowchartNode")
        });

        if !is_workflow {
            return None;
        }

        let workflow_id = WorkflowId::new();
        let mut commands = vec![
            WorkflowCommand::CreateWorkflow(crate::domain::commands::workflow::CreateWorkflow {
                workflow_id,
                name: imported.metadata.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Imported workflow")
                    .to_string(),
                description: imported.metadata.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Imported workflow")
                    .to_string(),
                created_by: UserId::new(), // Would need to pass this in
                tags: vec!["imported".to_string()],
            }),
        ];

        // Map node IDs to step IDs
        let mut step_map = HashMap::new();

        // Add workflow steps
        for node in &imported.nodes {
            let step_id = StepId::new();
            step_map.insert(node.id.clone(), step_id);

            let step_type = match node.node_type.as_str() {
                "Decision" => crate::domain::aggregates::workflow::StepType::Decision {
                    conditions: vec![],
                },
                "Start" | "End" => crate::domain::aggregates::workflow::StepType::UserTask,
                _ => crate::domain::aggregates::workflow::StepType::ServiceTask {
                    service: "default".to_string(),
                    operation: "execute".to_string(),
                },
            };

            let step = crate::domain::aggregates::workflow::WorkflowStep {
                id: step_id,
                name: node.label.clone(),
                step_type,
                node_id: NodeId::new(),
                inputs: vec![],
                outputs: vec![],
                timeout_ms: None,
                retry_policy: None,
            };

            commands.push(WorkflowCommand::AddStep(crate::domain::commands::workflow::AddStep {
                workflow_id,
                step,
            }));
        }

        // Connect steps based on edges
        for edge in &imported.edges {
            if let (Some(&source_step), Some(&target_step)) =
                (step_map.get(&edge.source), step_map.get(&edge.target)) {

                let condition = edge.properties.get("condition")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                commands.push(WorkflowCommand::ConnectSteps(crate::domain::commands::workflow::ConnectSteps {
                    workflow_id,
                    from_step: source_step,
                    to_step: target_step,
                    edge_id: EdgeId::new(),
                    condition,
                }));
            }
        }

        Some(commands)
    }

    /// Import from Arrows.app JSON format
    fn import_arrows_app(&self, content: &str, mapping: &ImportMapping) -> Result<ImportedGraph, DomainError> {
        let arrows_data: ArrowsAppJson = serde_json::from_str(content)
            .map_err(|e| DomainError::ValidationFailed(format!("Invalid Arrows.app JSON: {e}")))?;

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Import nodes
        for arrow_node in arrows_data.nodes {
            let mut properties = HashMap::new();

            // Apply field mappings
            for (key, value) in arrow_node.properties {
                let mapped_key = mapping.map_field(&key);
                let transformed_value = mapping.transform_value(&mapped_key, value);
                properties.insert(mapped_key, transformed_value);
            }

            // Handle caption specially
            if let Some(caption) = arrow_node.caption {
                let caption_key = mapping.map_field("caption");
                let caption_value = mapping.transform_value(&caption_key, serde_json::json!(caption));
                properties.insert(caption_key, caption_value);
            }

            nodes.push(ImportedNode {
                id: arrow_node.id,
                label: properties.get(&mapping.map_field("caption"))
                    .or_else(|| properties.get(&mapping.map_field("name")))
                    .or_else(|| properties.get(&mapping.map_field("label")))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Node")
                    .to_string(),
                node_type: mapping.map_node_type(arrow_node.style.get("node-type").unwrap_or(&"default".to_string())),
                position: Position3D {
                    x: arrow_node.position.x,
                    y: arrow_node.position.y,
                    z: 0.0,
                },
                properties,
            });
        }

        // Import relationships
        for arrow_rel in arrows_data.relationships {
            let mut properties = HashMap::new();

            // Apply field mappings to relationship properties
            for (key, value) in arrow_rel.properties {
                let mapped_key = mapping.map_field(&key);
                let transformed_value = mapping.transform_value(&mapped_key, value);
                properties.insert(mapped_key, transformed_value);
            }

            edges.push(ImportedEdge {
                id: arrow_rel.id,
                source: arrow_rel.from_id,
                target: arrow_rel.to_id,
                edge_type: mapping.map_relationship(&arrow_rel.rel_type),
                properties,
            });
        }

        Ok(ImportedGraph {
            nodes,
            edges,
            metadata: HashMap::new(),
        })
    }

    /// Import from progress.json format
    fn import_progress_json(&self, content: &str, mapping: &ImportMapping) -> Result<ImportedGraph, DomainError> {
        let progress_data: ProgressJson = serde_json::from_str(content)
            .map_err(|e| DomainError::ValidationFailed(format!("Invalid progress.json: {e}")))?;

        let mut nodes = Vec::new();
        for progress_node in progress_data.nodes {
            nodes.push(ImportedNode {
                id: progress_node.id.clone(),
                node_type: progress_node.node_type,
                label: progress_node.label,
                position: Position3D {
                    x: progress_node.position.x,
                    y: progress_node.position.y,
                    z: progress_node.position.z,
                },
                properties: progress_node.data,
            });
        }

        let mut edges = Vec::new();
        for progress_edge in progress_data.edges {
            let mut properties = HashMap::new();
            if let Some(label) = progress_edge.label {
                properties.insert("label".to_string(), serde_json::json!(label));
            }

            edges.push(ImportedEdge {
                id: progress_edge.id,
                source: progress_edge.source,
                target: progress_edge.target,
                edge_type: progress_edge.edge_type,
                properties,
            });
        }

        Ok(ImportedGraph {
            nodes,
            edges,
            metadata: {
                let mut map = HashMap::new();
                map.insert("name".to_string(), serde_json::json!(progress_data.metadata.name));
                map.insert("description".to_string(), serde_json::json!(progress_data.metadata.description));
                map.insert("version".to_string(), serde_json::json!(progress_data.metadata.version));
                map
            },
        })
    }

    /// Import from vocabulary.json format
    fn import_vocabulary_json(&self, content: &str, mapping: &ImportMapping) -> Result<ImportedGraph, DomainError> {
        let vocab_data: VocabularyJson = serde_json::from_str(content)
            .map_err(|e| DomainError::ValidationFailed(format!("Invalid vocabulary.json: {e}")))?;

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut y_offset = 0.0;

        // Create nodes for categories
        for category in vocab_data.categories {
            let category_id = format!("category-{}", category.id);
            nodes.push(ImportedNode {
                id: category_id.clone(),
                node_type: "Category".to_string(),
                label: category.name,
                position: Position3D {
                    x: -200.0,
                    y: y_offset,
                    z: 0.0,
                },
                properties: {
                    let mut props = HashMap::new();
                    props.insert("order".to_string(), serde_json::json!(category.order));
                    if let Some(desc) = &category.description {
                        props.insert("description".to_string(), serde_json::json!(desc));
                    }
                    props
                },
            });
            y_offset += 100.0;

            // Create subcategory nodes
            if let Some(subcategories) = category.subcategories {
                for subcat in subcategories {
                    let subcat_id = format!("subcategory-{}", subcat.id);
                    nodes.push(ImportedNode {
                        id: subcat_id.clone(),
                        node_type: "Subcategory".to_string(),
                        label: subcat.name,
                        position: Position3D {
                            x: 0.0,
                            y: y_offset,
                            z: 0.0,
                        },
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("order".to_string(), serde_json::json!(subcat.order));
                            props
                        },
                    });

                    // Connect category to subcategory
                    edges.push(ImportedEdge {
                        id: format!("edge-{category_id}-{subcat_id}"),
                        source: category_id.clone(),
                        target: subcat_id,
                        edge_type: "contains".to_string(),
                        properties: HashMap::new(),
                    });

                    y_offset += 50.0;
                }
            }
        }

        // Create nodes for terms
        let mut x_offset = 200.0;
        for (i, term) in vocab_data.terms.iter().enumerate() {
            let term_id = format!("term-{}", term.id);
            nodes.push(ImportedNode {
                id: term_id.clone(),
                node_type: term.term_type.clone().unwrap_or_else(|| "Term".to_string()),
                label: term.name.clone(),
                position: Position3D {
                    x: x_offset,
                    y: (i as f32) * 50.0,
                    z: 0.0,
                },
                properties: {
                    let mut props = HashMap::new();
                    props.insert("definition".to_string(), serde_json::json!(term.definition));
                    props.insert("category".to_string(), serde_json::json!(term.category));
                    if let Some(subcat) = &term.subcategory {
                        props.insert("subcategory".to_string(), serde_json::json!(subcat));
                    }
                    props
                },
            });

            // Connect term to its category
            let category_id = format!("category-{}", term.category);
            edges.push(ImportedEdge {
                id: format!("edge-{term_id}-{category_id}"),
                source: category_id,
                target: term_id.clone(),
                edge_type: "categorizes".to_string(),
                properties: HashMap::new(),
            });

            // Create relationship edges
            if let Some(relationships) = &term.relationships {
                for (rel_type, targets) in relationships {
                    for target in targets {
                        let target_id = format!("term-{target}");
                        edges.push(ImportedEdge {
                            id: format!("edge-{term_id}-{rel_type}-{target_id}"),
                            source: term_id.clone(),
                            target: target_id,
                            edge_type: rel_type.clone(),
                            properties: HashMap::new(),
                        });
                    }
                }
            }

            if (i + 1) % 10 == 0 {
                x_offset += 200.0;
            }
        }

        Ok(ImportedGraph {
            nodes,
            edges,
            metadata: {
                let mut map = HashMap::new();
                map.insert("name".to_string(), serde_json::json!(vocab_data.metadata.name));
                map.insert("description".to_string(), serde_json::json!(vocab_data.metadata.description));
                map.insert("version".to_string(), serde_json::json!(vocab_data.metadata.version));
                map
            },
        })
    }

    /// Import from Cypher query language
    fn import_cypher(&self, content: &str, mapping: &ImportMapping) -> Result<ImportedGraph, DomainError> {
        // Basic Cypher parser - in production, use a proper parser
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_counter = 0;
        let mut edge_counter = 0;

        // Simple pattern matching for CREATE statements
        let lines: Vec<&str> = content.lines().collect();

        for line in lines {
            let line = line.trim();

            // Match node creation: CREATE (n:Label {prop: value})
            if line.starts_with("CREATE") && line.contains("(") && line.contains(")") {
                if let Some(node_match) = extract_cypher_node(line) {
                    nodes.push(ImportedNode {
                        id: format!("cypher-node-{node_counter}"),
                        node_type: node_match.label,
                        label: node_match.properties.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Node")
                            .to_string(),
                        position: Position3D {
                            x: (node_counter as f32) * 100.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        properties: node_match.properties,
                    });
                    node_counter += 1;
                }
            }

            // Match relationship creation: CREATE (a)-[:TYPE]->(b)
            if line.contains(")-[") && line.contains("]->(") {
                if let Some(rel_match) = extract_cypher_relationship(line) {
                    edges.push(ImportedEdge {
                        id: format!("cypher-edge-{edge_counter}"),
                        source: rel_match.source,
                        target: rel_match.target,
                        edge_type: rel_match.rel_type,
                        properties: rel_match.properties,
                    });
                    edge_counter += 1;
                }
            }
        }

        Ok(ImportedGraph {
            nodes,
            edges,
            metadata: {
                let mut map = HashMap::new();
                map.insert("name".to_string(), serde_json::json!("Imported from Cypher"));
                map
            },
        })
    }

    /// Import from Mermaid diagram
    pub fn import_mermaid(&self, mermaid_content: &str) -> Result<ImportedGraph, DomainError> {
        // User Story: US9 - Import/Export
        // Acceptance Criteria: Can import graphs from Mermaid diagram format
        // Test Purpose: Validates that Mermaid diagrams are correctly parsed into ImportedGraph
        // Expected Behavior: Nodes and edges are created from Mermaid syntax

        use pulldown_cmark::{Parser, Event, Tag, TagEnd, CodeBlockKind};

        // First, check if this is markdown with mermaid code blocks
        let parser = Parser::new(mermaid_content);
        let mut mermaid_blocks = Vec::new();
        let mut in_mermaid_block = false;
        let mut current_block = String::new();
        let mut current_heading = String::new();
        let mut in_heading = false;
        let mut heading_level = 0;

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    in_heading = true;
                    heading_level = level as usize;
                    current_heading.clear();
                }
                Event::End(TagEnd::Heading(_)) => {
                    in_heading = false;
                }
                Event::Text(text) => {
                    if in_heading {
                        current_heading.push_str(&text);
                    } else if in_mermaid_block {
                        current_block.push_str(&text);
                    }
                }
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                    if lang.as_ref() == "mermaid" {
                        in_mermaid_block = true;
                        current_block.clear();
                    }
                }
                Event::End(TagEnd::CodeBlock) => {
                    if in_mermaid_block {
                        mermaid_blocks.push((current_heading.clone(), current_block.clone()));
                        in_mermaid_block = false;
                    }
                }
                _ => {}
            }
        }

        // If we found mermaid blocks in markdown, create subgraphs
        if !mermaid_blocks.is_empty() {
            let mut all_nodes = Vec::new();
            let mut all_edges = Vec::new();
            let mut metadata = HashMap::new();
            let mut subgraph_info = Vec::new();

            // Process each mermaid block as a subgraph
            for (idx, (heading, diagram)) in mermaid_blocks.iter().enumerate() {
                let subgraph_name = if heading.is_empty() {
                    format!("Subgraph {}", idx + 1)
                } else {
                    heading.clone()
                };

                // Parse this mermaid diagram
                match self.parse_mermaid_diagram(diagram) {
                    Ok(mut subgraph) => {
                        // Track which nodes belong to this subgraph
                        let node_ids: Vec<String> = subgraph.nodes.iter()
                            .map(|n| n.id.clone())
                            .collect();

                        // Add subgraph prefix to node IDs to avoid conflicts
                        let prefix = format!("sg{}_", idx);
                        for node in &mut subgraph.nodes {
                            node.id = format!("{}{}", prefix, node.id);
                            // Add subgraph metadata
                            node.properties.insert("subgraph".to_string(), serde_json::json!(subgraph_name.clone()));
                            node.properties.insert("subgraph_id".to_string(), serde_json::json!(idx));

                            // Offset positions for each subgraph
                            node.position.x += (idx as f32) * 400.0;
                            node.position.y += (idx as f32) * 100.0;
                        }

                        // Update edge IDs and references
                        for edge in &mut subgraph.edges {
                            edge.id = format!("{}{}", prefix, edge.id);
                            edge.source = format!("{}{}", prefix, edge.source);
                            edge.target = format!("{}{}", prefix, edge.target);
                            // Add subgraph metadata
                            edge.properties.insert("subgraph".to_string(), serde_json::json!(subgraph_name.clone()));
                            edge.properties.insert("subgraph_id".to_string(), serde_json::json!(idx));
                        }

                        // Store subgraph information
                        subgraph_info.push(serde_json::json!({
                            "id": idx,
                            "name": subgraph_name,
                            "node_count": subgraph.nodes.len(),
                            "edge_count": subgraph.edges.len(),
                            "node_ids": node_ids.iter().map(|id| format!("{}{}", prefix, id)).collect::<Vec<_>>(),
                        }));

                        all_nodes.extend(subgraph.nodes);
                        all_edges.extend(subgraph.edges);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse subgraph '{}': {}", subgraph_name, e);
                    }
                }
            }

            if all_nodes.is_empty() {
                return Err(DomainError::ValidationError("No valid nodes found in any Mermaid diagram".to_string()));
            }

            metadata.insert("subgraphs".to_string(), serde_json::json!(subgraph_info));
            metadata.insert("subgraph_count".to_string(), serde_json::json!(mermaid_blocks.len()));

            Ok(ImportedGraph {
                nodes: all_nodes,
                edges: all_edges,
                metadata,
            })
        } else {
            // Single mermaid diagram without markdown
            self.parse_mermaid_diagram(mermaid_content)
        }
    }

    fn parse_mermaid_diagram(&self, diagram: &str) -> Result<ImportedGraph, DomainError> {
        use nom::{
            IResult,
            branch::alt,
            bytes::complete::{tag, take_until, take_while1, is_not},
            character::complete::{char, multispace0, multispace1, alphanumeric1},
            combinator::{opt, map, recognize},
            multi::{many0, separated_list0},
            sequence::{tuple, delimited, preceded, terminated},
        };

        // Parse node ID (can be alphanumeric)
        fn node_id(input: &str) -> IResult<&str, &str> {
            recognize(take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-'))(input)
        }

        // Parse node label in square brackets [Label] or curly braces {Label}
        fn node_label(input: &str) -> IResult<&str, &str> {
            alt((
                delimited(
                    char('['),
                    take_until("]"),
                    char(']')
                ),
                delimited(
                    char('{'),
                    take_until("}"),
                    char('}')
                ),
                delimited(
                    char('('),
                    take_until(")"),
                    char(')')
                ),
            ))(input)
        }

        // Parse node with optional label: A[Start]
        fn node_with_label(input: &str) -> IResult<&str, (&str, Option<&str>)> {
            tuple((
                node_id,
                opt(node_label)
            ))(input)
        }

        // Parse arrow types
        fn arrow(input: &str) -> IResult<&str, &str> {
            alt((
                tag("-->"),
                tag("->"),
                tag("---"),
                tag("-.->"),
                tag("==>"),
                tag("--"),
            ))(input)
        }

        // Parse a single edge: A --> B
        fn edge(input: &str) -> IResult<&str, ((&str, Option<&str>), (&str, Option<&str>))> {
            tuple((
                terminated(node_with_label, multispace0),
                terminated(arrow, multispace0),
                node_with_label
            ))(input)
            .map(|(rest, (source, _, target))| (rest, (source, target)))
        }

        // Parse graph type declaration
        fn graph_declaration(input: &str) -> IResult<&str, ()> {
            map(
                tuple((
                    alt((tag("graph"), tag("flowchart"))),
                    multispace1,
                    alt((tag("TD"), tag("LR"), tag("TB"), tag("RL"), tag("BT")))
                )),
                |_| ()
            )(input)
        }

        // Main parser
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();
        let mut node_counter = 0;

        // Split into lines and process each
        let lines: Vec<&str> = diagram.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("%%"))
            .collect();

        // Skip the graph declaration if present
        let start_index = if !lines.is_empty() {
            if let Ok(_) = graph_declaration(lines[0]) {
                1
            } else {
                0
            }
        } else {
            0
        };

        // Process each line
        for line in &lines[start_index..] {
            // Try to parse as edge
            if let Ok((_, ((source_id, source_label), (target_id, target_label)))) = edge(line) {
                // Add source node if not exists
                if !nodes.contains_key(source_id) {
                    let label = source_label.unwrap_or(source_id).to_string();
                    nodes.insert(source_id.to_string(), ImportedNode {
                        id: source_id.to_string(),
                        node_type: "Node".to_string(),
                        label,
                        position: Position3D {
                            x: (node_counter % 3) as f32 * 200.0,
                            y: (node_counter / 3) as f32 * 150.0,
                            z: 0.0,
                        },
                        properties: HashMap::new(),
                    });
                    node_counter += 1;
                }

                // Add target node if not exists
                if !nodes.contains_key(target_id) {
                    let label = target_label.unwrap_or(target_id).to_string();
                    nodes.insert(target_id.to_string(), ImportedNode {
                        id: target_id.to_string(),
                        node_type: "Node".to_string(),
                        label,
                        position: Position3D {
                            x: (node_counter % 3) as f32 * 200.0,
                            y: (node_counter / 3) as f32 * 150.0,
                            z: 0.0,
                        },
                        properties: HashMap::new(),
                    });
                    node_counter += 1;
                }

                // Add edge
                edges.push(ImportedEdge {
                    id: format!("{}-{}", source_id, target_id),
                    source: source_id.to_string(),
                    target: target_id.to_string(),
                    edge_type: "-->".to_string(),
                    properties: HashMap::new(),
                });
            }
            // Try to parse as standalone node
            else if let Ok((_, (node_id_str, label))) = node_with_label(line) {
                if !nodes.contains_key(node_id_str) {
                    let label = label.unwrap_or(node_id_str).to_string();
                    nodes.insert(node_id_str.to_string(), ImportedNode {
                        id: node_id_str.to_string(),
                        node_type: "Node".to_string(),
                        label,
                        position: Position3D {
                            x: (node_counter % 3) as f32 * 200.0,
                            y: (node_counter / 3) as f32 * 150.0,
                            z: 0.0,
                        },
                        properties: HashMap::new(),
                    });
                    node_counter += 1;
                }
            }
        }

        if nodes.is_empty() {
            return Err(DomainError::ValidationError("No valid nodes found in Mermaid diagram".to_string()));
        }

        Ok(ImportedGraph {
            nodes: nodes.into_values().collect(),
            edges,
            metadata: HashMap::new(),
        })
    }

    /// Import from DOT/Graphviz format
    fn import_dot(&self, content: &str, mapping: &ImportMapping) -> Result<ImportedGraph, DomainError> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_map = HashMap::new();

        // Basic DOT parser - in production, use a proper parser
        let lines: Vec<&str> = content.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        let graph_name = lines.first()
            .and_then(|l| {
                if l.starts_with("digraph") || l.starts_with("graph") {
                    l.split_whitespace().nth(1)
                } else {
                    None
                }
            })
            .unwrap_or("graph");

        let is_directed = lines.first()
            .map(|l| l.starts_with("digraph"))
            .unwrap_or(false);

        for line in lines.iter().skip(1) {
            // Skip opening/closing braces
            if *line == "{" || *line == "}" {
                continue;
            }

            // Parse node definitions: node [label="Label"];
            if let Some(node_match) = extract_dot_node(line) {
                let node_id = format!("dot-{}", node_match.id);
                node_map.insert(node_match.id.clone(), node_id.clone());

                nodes.push(ImportedNode {
                    id: node_id,
                    node_type: "DotNode".to_string(),
                    label: node_match.label.unwrap_or(node_match.id),
                    position: Position3D {
                        x: (nodes.len() as f32) * 150.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    properties: node_match.attributes,
                });
            }

            // Parse edge definitions: a -> b or a -- b
            if let Some(edge_match) = extract_dot_edge(line, is_directed) {
                let source_id = node_map.get(&edge_match.source)
                    .cloned()
                    .unwrap_or_else(|| {
                        let id = format!("dot-{}", edge_match.source);
                        node_map.insert(edge_match.source.clone(), id.clone());
                        nodes.push(ImportedNode {
                            id: id.clone(),
                            node_type: "DotNode".to_string(),
                            label: edge_match.source.clone(),
                            position: Position3D {
                                x: (nodes.len() as f32) * 150.0,
                                y: -100.0,
                                z: 0.0,
                            },
                            properties: HashMap::new(),
                        });
                        id
                    });

                let target_id = node_map.get(&edge_match.target)
                    .cloned()
                    .unwrap_or_else(|| {
                        let id = format!("dot-{}", edge_match.target);
                        node_map.insert(edge_match.target.clone(), id.clone());
                        nodes.push(ImportedNode {
                            id: id.clone(),
                            node_type: "DotNode".to_string(),
                            label: edge_match.target.clone(),
                            position: Position3D {
                                x: (nodes.len() as f32) * 150.0,
                                y: 100.0,
                                z: 0.0,
                            },
                            properties: HashMap::new(),
                        });
                        id
                    });

                edges.push(ImportedEdge {
                    id: format!("edge-{source_id}-{target_id}"),
                    source: source_id,
                    target: target_id,
                    edge_type: if is_directed { "directed" } else { "undirected" }.to_string(),
                    properties: edge_match.attributes,
                });
            }
        }

        // Apply circular layout for better visualization
        apply_circular_layout(&mut nodes);

        Ok(ImportedGraph {
            nodes,
            edges,
            metadata: {
                let mut map = HashMap::new();
                map.insert("name".to_string(), serde_json::json!(format!("Imported from DOT: {}", graph_name)));
                map.insert("graph_type".to_string(), serde_json::json!(if is_directed { "digraph" } else { "graph" }));
                map.insert("original_name".to_string(), serde_json::json!(graph_name));
                map
            },
        })
    }

    /// Import from RSS/Atom feed
    fn import_rss_atom(&self, content: &str, mapping: &ImportMapping) -> Result<ImportedGraph, DomainError> {
        // Try to parse as Atom first (EventStore format)
        if content.contains("<feed") && content.contains("xmlns") {
            return self.import_atom_feed(content, mapping);
        }

        // Try RSS 2.0
        if content.contains("<rss") || content.contains("<channel>") {
            return self.import_rss_feed(content, mapping);
        }

        Err(DomainError::ValidationFailed("Content is not a valid RSS or Atom feed".to_string()))
    }

    /// Import from Atom feed (EventStore format)
    fn import_atom_feed(&self, content: &str, mapping: &ImportMapping) -> Result<ImportedGraph, DomainError> {
        // For EventStore Atom feeds, we'll use quick-xml for parsing
        // For now, we'll use a simplified approach
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Extract feed title
        let title = extract_xml_value(content, "title")
            .unwrap_or_else(|| "Imported Atom Feed".to_string());

        // Create a root node for the feed/stream
        let feed_id = extract_xml_value(content, "id")
            .unwrap_or_else(|| "atom-feed".to_string());

        nodes.push(ImportedNode {
            id: format!("feed-{}", sanitize_id(&feed_id)),
            node_type: "EventStream".to_string(),
            label: title.clone(),
            position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
            properties: {
                let mut props = HashMap::new();
                if let Some(updated) = extract_xml_value(content, "updated") {
                    props.insert("updated".to_string(), serde_json::json!(updated));
                }
                props
            },
        });

        // Parse entries
        let entries = extract_xml_sections(content, "entry");
        let mut y_offset = 100.0;

        for (i, entry) in entries.iter().enumerate() {
            let entry_id = extract_xml_value(entry, "id")
                .unwrap_or_else(|| format!("entry-{i}"));
            let entry_title = extract_xml_value(entry, "title")
                .unwrap_or_else(|| "Event".to_string());

            // Extract EventStore specific fields
            let event_type = extract_xml_value(entry, "eventType");
            let event_number = extract_xml_value(entry, "eventNumber")
                .and_then(|n| n.parse::<i64>().ok());
            let stream_id = extract_xml_value(entry, "streamId");

            // Extract content
            let content_value = extract_xml_content(entry, "content");

            let node_id = format!("event-{}", sanitize_id(&entry_id));

            nodes.push(ImportedNode {
                id: node_id.clone(),
                node_type: event_type.clone().unwrap_or_else(|| "Event".to_string()),
                label: entry_title,
                position: Position3D {
                    x: 200.0,
                    y: y_offset,
                    z: 0.0,
                },
                properties: {
                    let mut props = HashMap::new();
                    if let Some(et) = event_type {
                        props.insert("eventType".to_string(), serde_json::json!(et));
                    }
                    if let Some(en) = event_number {
                        props.insert("eventNumber".to_string(), serde_json::json!(en));
                    }
                    if let Some(sid) = stream_id {
                        props.insert("streamId".to_string(), serde_json::json!(sid));
                    }
                    if let Some(updated) = extract_xml_value(entry, "updated") {
                        props.insert("timestamp".to_string(), serde_json::json!(updated));
                    }
                    if let Some(summary) = extract_xml_value(entry, "summary") {
                        props.insert("summary".to_string(), serde_json::json!(summary));
                    }
                    if let Some(content) = content_value {
                        // Try to parse as JSON if it's marked as JSON
                        if entry.contains("isJson=\"true\"") {
                            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                                props.insert("data".to_string(), json_value);
                            } else {
                                props.insert("content".to_string(), serde_json::json!(content));
                            }
                        } else {
                            props.insert("content".to_string(), serde_json::json!(content));
                        }
                    }
                    props
                },
            });

            // Connect event to stream
            edges.push(ImportedEdge {
                id: format!("edge-stream-{i}"),
                source: format!("feed-{}", sanitize_id(&feed_id)),
                target: node_id.clone(),
                edge_type: "contains".to_string(),
                properties: {
                    let mut props = HashMap::new();
                    if let Some(en) = event_number {
                        props.insert("sequence".to_string(), serde_json::json!(en));
                    }
                    props
                },
            });

            // If this is not the first event, connect to previous event
            if i > 0 {
                let prev_id = format!("event-{}", sanitize_id(entries[i-1]
                    .split("<id>").nth(1)
                    .and_then(|s| s.split("</id>").next())
                    .unwrap_or(&format!("entry-{}", i-1))));

                edges.push(ImportedEdge {
                    id: format!("edge-sequence-{i}"),
                    source: prev_id,
                    target: node_id,
                    edge_type: "followed_by".to_string(),
                    properties: HashMap::new(),
                });
            }

            y_offset += 100.0;
        }

        Ok(ImportedGraph {
            nodes,
            edges,
            metadata: {
                let mut map = HashMap::new();
                map.insert("name".to_string(), serde_json::json!(title));
                map.insert("format".to_string(), serde_json::json!("atom"));
                map.insert("feed_id".to_string(), serde_json::json!(feed_id));
                map
            },
        })
    }

    /// Import from RSS 2.0 feed
    fn import_rss_feed(&self, content: &str, mapping: &ImportMapping) -> Result<ImportedGraph, DomainError> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Extract channel information
        let channel_title = extract_xml_value(content, "title")
            .unwrap_or_else(|| "RSS Feed".to_string());
        let channel_desc = extract_xml_value(content, "description");
        let channel_link = extract_xml_value(content, "link");

        // Create root node for the channel
        let channel_id = "rss-channel";
        nodes.push(ImportedNode {
            id: channel_id.to_string(),
            node_type: "RSSChannel".to_string(),
            label: channel_title.clone(),
            position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
            properties: {
                let mut props = HashMap::new();
                if let Some(desc) = channel_desc {
                    props.insert("description".to_string(), serde_json::json!(desc));
                }
                if let Some(link) = channel_link {
                    props.insert("link".to_string(), serde_json::json!(link));
                }
                if let Some(pub_date) = extract_xml_value(content, "pubDate") {
                    props.insert("pubDate".to_string(), serde_json::json!(pub_date));
                }
                props
            },
        });

        // Parse items
        let items = extract_xml_sections(content, "item");
        let mut y_offset = 100.0;

        for (i, item) in items.iter().enumerate() {
            let item_title = extract_xml_value(item, "title")
                .unwrap_or_else(|| format!("Item {}", i + 1));
            let item_guid = extract_xml_value(item, "guid")
                .unwrap_or_else(|| format!("item-{i}"));

            let node_id = format!("item-{}", sanitize_id(&item_guid));

            nodes.push(ImportedNode {
                id: node_id.clone(),
                node_type: "RSSItem".to_string(),
                label: item_title,
                position: Position3D {
                    x: 200.0,
                    y: y_offset,
                    z: 0.0,
                },
                properties: {
                    let mut props = HashMap::new();
                    if let Some(desc) = extract_xml_value(item, "description") {
                        props.insert("description".to_string(), serde_json::json!(desc));
                    }
                    if let Some(link) = extract_xml_value(item, "link") {
                        props.insert("link".to_string(), serde_json::json!(link));
                    }
                    if let Some(pub_date) = extract_xml_value(item, "pubDate") {
                        props.insert("pubDate".to_string(), serde_json::json!(pub_date));
                    }
                    if let Some(author) = extract_xml_value(item, "author") {
                        props.insert("author".to_string(), serde_json::json!(author));
                    }

                    // Extract categories
                    let categories = extract_xml_values(item, "category");
                    if !categories.is_empty() {
                        props.insert("categories".to_string(), serde_json::json!(categories));
                    }

                    props.insert("guid".to_string(), serde_json::json!(item_guid));
                    props
                },
            });

            // Connect item to channel
            edges.push(ImportedEdge {
                id: format!("edge-channel-item-{i}"),
                source: channel_id.to_string(),
                target: node_id,
                edge_type: "publishes".to_string(),
                properties: HashMap::new(),
            });

            y_offset += 100.0;
        }

        Ok(ImportedGraph {
            nodes,
            edges,
            metadata: {
                let mut map = HashMap::new();
                map.insert("name".to_string(), serde_json::json!(channel_title));
                map.insert("format".to_string(), serde_json::json!("rss"));
                map
            },
        })
    }

    /// Apply layout algorithm to imported graph
    fn apply_layout(
        &self,
        graph: &mut ImportedGraph,
        config: &LayoutConfig,
    ) -> Result<(), DomainError> {
        match &config.algorithm {
            LayoutAlgorithm::None => Ok(()),
            LayoutAlgorithm::ForceDirected => {
                self.apply_force_directed_layout(graph, &config.parameters)
            }
            LayoutAlgorithm::Circular => {
                self.apply_circular_layout(graph, &config.parameters)
            }
            LayoutAlgorithm::Hierarchical => {
                self.apply_hierarchical_layout(graph, &config.parameters)
            }
            LayoutAlgorithm::Grid => {
                self.apply_grid_layout(graph, &config.parameters)
            }
            LayoutAlgorithm::Random => {
                self.apply_random_layout(graph, &config.parameters)
            }
            LayoutAlgorithm::PropertyBased { property } => {
                self.apply_property_based_layout(graph, property, &config.parameters)
            }
        }
    }

    /// Apply force-directed layout
    fn apply_force_directed_layout(
        &self,
        graph: &mut ImportedGraph,
        params: &LayoutParameters,
    ) -> Result<(), DomainError> {
        let node_count = graph.nodes.len();
        if node_count == 0 {
            return Ok(());
        }

        // Initialize positions if not set
        for (i, node) in graph.nodes.iter_mut().enumerate() {
            if node.position.x == 0.0 && node.position.y == 0.0 && node.position.z == 0.0 {
                // Initial circular placement
                let angle = 2.0 * std::f32::consts::PI * i as f32 / node_count as f32;
                node.position = Position3D {
                    x: angle.cos() * 200.0,
                    y: angle.sin() * 200.0,
                    z: 0.0,
                };
            }
        }

        // Run force-directed simulation
        for _ in 0..params.iterations {
            // Calculate forces
            let mut forces: Vec<(f32, f32, f32)> = vec![(0.0, 0.0, 0.0); node_count];

            // Repulsion between all nodes
            for i in 0..node_count {
                for j in (i + 1)..node_count {
                    let dx = graph.nodes[j].position.x - graph.nodes[i].position.x;
                    let dy = graph.nodes[j].position.y - graph.nodes[i].position.y;
                    let dz = graph.nodes[j].position.z - graph.nodes[i].position.z;

                    let dist_sq = dx * dx + dy * dy + dz * dz + 0.01; // Avoid division by zero
                    let dist = dist_sq.sqrt();

                    let force = params.repulsion_force / dist_sq;
                    let fx = force * dx / dist;
                    let fy = force * dy / dist;
                    let fz = force * dz / dist;

                    forces[i].0 -= fx;
                    forces[i].1 -= fy;
                    forces[i].2 -= fz;
                    forces[j].0 += fx;
                    forces[j].1 += fy;
                    forces[j].2 += fz;
                }
            }

            // Attraction along edges
            for edge in &graph.edges {
                let source_idx = graph.nodes.iter().position(|n| n.id == edge.source);
                let target_idx = graph.nodes.iter().position(|n| n.id == edge.target);

                if let (Some(i), Some(j)) = (source_idx, target_idx) {
                    let dx = graph.nodes[j].position.x - graph.nodes[i].position.x;
                    let dy = graph.nodes[j].position.y - graph.nodes[i].position.y;
                    let dz = graph.nodes[j].position.z - graph.nodes[i].position.z;

                    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                    if dist > 0.0 {
                        let force = params.attraction_force * dist;
                        let fx = force * dx / dist;
                        let fy = force * dy / dist;
                        let fz = force * dz / dist;

                        forces[i].0 += fx;
                        forces[i].1 += fy;
                        forces[i].2 += fz;
                        forces[j].0 -= fx;
                        forces[j].1 -= fy;
                        forces[j].2 -= fz;
                    }
                }
            }

            // Center force
            for i in 0..node_count {
                forces[i].0 -= graph.nodes[i].position.x * params.center_force;
                forces[i].1 -= graph.nodes[i].position.y * params.center_force;
                forces[i].2 -= graph.nodes[i].position.z * params.center_force;
            }

            // Apply forces with damping
            for i in 0..node_count {
                graph.nodes[i].position.x += forces[i].0 * params.damping;
                graph.nodes[i].position.y += forces[i].1 * params.damping;
                graph.nodes[i].position.z += forces[i].2 * params.damping;
            }
        }

        Ok(())
    }

    /// Apply circular layout
    fn apply_circular_layout(
        &self,
        graph: &mut ImportedGraph,
        params: &LayoutParameters,
    ) -> Result<(), DomainError> {
        let node_count = graph.nodes.len();
        if node_count == 0 {
            return Ok(());
        }

        let radius = params.spacing * node_count as f32 / (2.0 * std::f32::consts::PI);

        for (i, node) in graph.nodes.iter_mut().enumerate() {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / node_count as f32;
            node.position = Position3D {
                x: angle.cos() * radius,
                y: angle.sin() * radius,
                z: 0.0,
            };
        }

        Ok(())
    }

    /// Apply hierarchical layout
    fn apply_hierarchical_layout(
        &self,
        graph: &mut ImportedGraph,
        params: &LayoutParameters,
    ) -> Result<(), DomainError> {
        // Simple hierarchical layout based on node connections
        let mut levels: HashMap<String, u32> = HashMap::new();
        let mut visited: HashSet<String> = HashSet::new();

        // Find root nodes (no incoming edges)
        let mut root_nodes: Vec<String> = graph.nodes.iter()
            .map(|n| n.id.clone())
            .filter(|id| !graph.edges.iter().any(|e| &e.target == id))
            .collect();

        if root_nodes.is_empty() && !graph.nodes.is_empty() {
            // If no root nodes, start with first node
            root_nodes.push(graph.nodes[0].id.clone());
        }

        // Assign levels using BFS
        for root in root_nodes {
            let mut queue = VecDeque::new();
            queue.push_back((root.clone(), 0));

            while let Some((node_id, level)) = queue.pop_front() {
                if visited.contains(&node_id) {
                    continue;
                }

                visited.insert(node_id.clone());
                levels.insert(node_id.clone(), level);

                // Find children
                for edge in &graph.edges {
                    if edge.source == node_id {
                        queue.push_back((edge.target.clone(), level + 1));
                    }
                }
            }
        }

        // Group nodes by level
        let mut level_groups: HashMap<u32, Vec<String>> = HashMap::new();
        for (node_id, level) in &levels {
            level_groups.entry(*level).or_default().push(node_id.clone());
        }

        // Position nodes
        let max_level = level_groups.keys().max().copied().unwrap_or(0);
        for node in &mut graph.nodes {
            if let Some(&level) = levels.get(&node.id) {
                let nodes_at_level = level_groups.get(&level).map(|v| v.len()).unwrap_or(1);
                let index_at_level = level_groups.get(&level)
                    .and_then(|v| v.iter().position(|id| id == &node.id))
                    .unwrap_or(0);

                let x = (index_at_level as f32 - (nodes_at_level as f32 - 1.0) / 2.0) * params.spacing;
                let y = -(level as f32) * params.spacing;

                node.position = Position3D { x, y, z: 0.0 };
            }
        }

        Ok(())
    }

    /// Apply grid layout
    fn apply_grid_layout(
        &self,
        graph: &mut ImportedGraph,
        params: &LayoutParameters,
    ) -> Result<(), DomainError> {
        let node_count = graph.nodes.len();
        if node_count == 0 {
            return Ok(());
        }

        let cols = (node_count as f32).sqrt().ceil() as usize;

        for (i, node) in graph.nodes.iter_mut().enumerate() {
            let row = i / cols;
            let col = i % cols;

            node.position = Position3D {
                x: col as f32 * params.spacing - (cols as f32 - 1.0) * params.spacing / 2.0,
                y: -(row as f32 * params.spacing),
                z: 0.0,
            };
        }

        Ok(())
    }

    /// Apply random layout
    fn apply_random_layout(
        &self,
        graph: &mut ImportedGraph,
        params: &LayoutParameters,
    ) -> Result<(), DomainError> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let range = params.spacing * (graph.nodes.len() as f32).sqrt();

        for node in &mut graph.nodes {
            node.position = Position3D {
                x: rng.gen_range(-range..range),
                y: rng.gen_range(-range..range),
                z: 0.0,
            };
        }

        Ok(())
    }

    /// Apply property-based layout
    fn apply_property_based_layout(
        &self,
        graph: &mut ImportedGraph,
        property: &str,
        params: &LayoutParameters,
    ) -> Result<(), DomainError> {
        // Extract property values
        let mut property_values: Vec<(usize, f32)> = Vec::new();

        for (i, node) in graph.nodes.iter().enumerate() {
            if let Some(value) = node.properties.get(property) {
                if let Some(num) = value.as_f64() {
                    property_values.push((i, num as f32));
                } else if let Some(s) = value.as_str() {
                    // Try to parse string as number
                    if let Ok(num) = s.parse::<f32>() {
                        property_values.push((i, num));
                    }
                }
            }
        }

        if property_values.is_empty() {
            // Fall back to grid layout if no property values found
            return self.apply_grid_layout(graph, params);
        }

        // Normalize property values
        let min_val = property_values.iter().map(|(_, v)| *v).fold(f32::INFINITY, f32::min);
        let max_val = property_values.iter().map(|(_, v)| *v).fold(f32::NEG_INFINITY, f32::max);
        let range = max_val - min_val;

        if range > 0.0 {
            // Position nodes based on property value
            let scale = params.property_scale.get(property).copied().unwrap_or(1.0);

            for (idx, value) in property_values {
                let normalized = (value - min_val) / range;
                let angle = normalized * 2.0 * std::f32::consts::PI;
                let radius = params.spacing * scale * (1.0 + normalized);

                graph.nodes[idx].position = Position3D {
                    x: angle.cos() * radius,
                    y: angle.sin() * radius,
                    z: normalized * params.spacing * 0.5, // Use Z for property value
                };
            }
        }

        Ok(())
    }
}

// Helper structures for parsing
struct CypherNode {
    label: String,
    properties: HashMap<String, serde_json::Value>,
}

struct CypherRelationship {
    source: String,
    target: String,
    rel_type: String,
    properties: HashMap<String, serde_json::Value>,
}

struct MermaidNode {
    id: String,
    label: String,
}

struct MermaidEdge {
    source: String,
    target: String,
    edge_type: String,
    label: Option<String>,
}

struct DotNode {
    id: String,
    label: Option<String>,
    attributes: HashMap<String, serde_json::Value>,
}

struct DotEdge {
    source: String,
    target: String,
    attributes: HashMap<String, serde_json::Value>,
}

// Helper functions for parsing (simplified implementations)
fn extract_cypher_node(line: &str) -> Option<CypherNode> {
    // Very basic Cypher node extraction
    // In production, use a proper Cypher parser
    if let Some(start) = line.find("(") {
        if let Some(end) = line.find(")") {
            let node_def = &line[start + 1..end];
            let parts: Vec<&str> = node_def.split(':').collect();

            if parts.len() >= 2 {
                let label = parts[1].split_whitespace().next()?.to_string();
                let properties = HashMap::new(); // Would parse properties here

                return Some(CypherNode { label, properties });
            }
        }
    }
    None
}

fn extract_cypher_relationship(line: &str) -> Option<CypherRelationship> {
    // Very basic relationship extraction
    None // Would implement proper parsing
}

fn extract_mermaid_node(line: &str) -> Option<MermaidNode> {
    // Match patterns like: A[Label] or A(Label) or A{Label}
    let patterns = [('[', ']'), ('(', ')'), ('{', '}')];

    for (open, close) in patterns {
        if let Some(open_idx) = line.find(open) {
            if let Some(close_idx) = line.find(close) {
                if open_idx > 0 && close_idx > open_idx {
                    let id = line[..open_idx].trim().to_string();
                    let label = line[open_idx + 1..close_idx].trim().to_string();
                    return Some(MermaidNode { id, label });
                }
            }
        }
    }

    None
}

fn extract_mermaid_edge(line: &str) -> Option<MermaidEdge> {
    // Match patterns like: A --> B or A -.-> B or A ==> B
    let edge_patterns = ["-->", "---", "-.->", "==>", "-..-", "=="];

    for pattern in edge_patterns {
        if let Some(idx) = line.find(pattern) {
            let source = line[..idx].trim();
            let rest = &line[idx + pattern.len()..];

            // Check for label: A -->|Label| B
            let (target, label) = if let Some(label_start) = rest.find('|') {
                if let Some(label_end) = rest[label_start + 1..].find('|') {
                    let label = rest[label_start + 1..label_start + 1 + label_end].trim().to_string();
                    let target = rest[label_start + label_end + 2..].trim();
                    (target, Some(label))
                } else {
                    (rest.trim(), None)
                }
            } else {
                (rest.trim(), None)
            };

            return Some(MermaidEdge {
                source: source.to_string(),
                target: target.to_string(),
                edge_type: pattern.to_string(),
                label,
            });
        }
    }

    None
}

fn extract_dot_node(line: &str) -> Option<DotNode> {
    // Match pattern: node [attributes];
    if line.contains('[') && line.contains(']') && line.ends_with(';') {
        let parts: Vec<&str> = line.split('[').collect();
        if parts.len() >= 2 {
            let id = parts[0].trim().to_string();
            let attr_part = parts[1].split(']').next()?;

            // Parse attributes (simplified)
            let attributes = HashMap::new();
            if attr_part.contains("label=") {
                if let Some(label_start) = attr_part.find("\"") {
                    if let Some(label_end) = attr_part[label_start + 1..].find("\"") {
                        let label = attr_part[label_start + 1..label_start + 1 + label_end].to_string();
                        return Some(DotNode {
                            id,
                            label: Some(label),
                            attributes,
                        });
                    }
                }
            }

            return Some(DotNode {
                id,
                label: None,
                attributes,
            });
        }
    }
    None
}

fn extract_dot_edge(line: &str, is_directed: bool) -> Option<DotEdge> {
    let edge_op = if is_directed { "->" } else { "--" };

    if line.contains(edge_op) {
        let parts: Vec<&str> = line.split(edge_op).collect();
        if parts.len() >= 2 {
            let source = parts[0].trim().to_string();
            let target_part = parts[1].trim();

            // Extract target and attributes
            let (target, attributes) = if let Some(attr_idx) = target_part.find('[') {
                let target = target_part[..attr_idx].trim().to_string();
                (target, HashMap::new()) // Would parse attributes
            } else {
                let target = target_part.trim_end_matches(';').to_string();
                (target, HashMap::new())
            };

            return Some(DotEdge {
                source,
                target,
                attributes,
            });
        }
    }
    None
}

// Layout helper functions
fn apply_simple_layout(nodes: &mut [ImportedNode]) {
    // Simple grid layout
    let cols = ((nodes.len() as f32).sqrt().ceil()) as usize;

    for (i, node) in nodes.iter_mut().enumerate() {
        let row = i / cols;
        let col = i % cols;

        node.position = Position3D {
            x: (col as f32) * 200.0 - 400.0,
            y: (row as f32) * 200.0 - 400.0,
            z: 0.0,
        };
    }
}

fn apply_circular_layout(nodes: &mut [ImportedNode]) {
    let count = nodes.len();
    if count == 0 {
        return;
    }

    let radius = 300.0;
    let angle_step = 2.0 * std::f32::consts::PI / count as f32;

    for (i, node) in nodes.iter_mut().enumerate() {
        let angle = i as f32 * angle_step;
        node.position = Position3D {
            x: radius * angle.cos(),
            y: radius * angle.sin(),
            z: 0.0,
        };
    }
}

// XML parsing helper functions
fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");

    xml.find(&start_tag)
        .and_then(|start| {
            let content_start = start + start_tag.len();
            xml[content_start..].find(&end_tag)
                .map(|end| xml[content_start..content_start + end].to_string())
        })
        .map(|s| s.trim().to_string())
}

fn extract_xml_values(xml: &str, tag: &str) -> Vec<String> {
    let mut values = Vec::new();
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");

    let mut search_from = 0;
    while let Some(start) = xml[search_from..].find(&start_tag) {
        let absolute_start = search_from + start + start_tag.len();
        if let Some(end) = xml[absolute_start..].find(&end_tag) {
            let value = xml[absolute_start..absolute_start + end].trim().to_string();
            values.push(value);
            search_from = absolute_start + end + end_tag.len();
        } else {
            break;
        }
    }

    values
}

fn extract_xml_sections(xml: &str, tag: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let start_tag = format!("<{tag}"); // Don't include > to handle attributes
    let end_tag = format!("</{tag}>");

    let mut search_from = 0;
    while let Some(start) = xml[search_from..].find(&start_tag) {
        let absolute_start = search_from + start;
        if let Some(end) = xml[absolute_start..].find(&end_tag) {
            let section = xml[absolute_start..absolute_start + end + end_tag.len()].to_string();
            sections.push(section);
            search_from = absolute_start + end + end_tag.len();
        } else {
            break;
        }
    }

    sections
}

fn extract_xml_content(xml: &str, tag: &str) -> Option<String> {
    // Handle content tag with attributes like <content type="application/json">
    let start_pattern = format!("<{tag}");

    if let Some(start_pos) = xml.find(&start_pattern) {
        // Find the closing > of the opening tag
        if let Some(tag_end) = xml[start_pos..].find('>') {
            let content_start = start_pos + tag_end + 1;
            let end_tag = format!("</{tag}>");

            if let Some(end_pos) = xml[content_start..].find(&end_tag) {
                return Some(xml[content_start..content_start + end_pos].trim().to_string());
            }
        }
    }

    None
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_arrows_app() {
        // User Story: US9 - Import/Export
        // Acceptance Criteria: Can import graphs from Arrows.app JSON format
        // Test Purpose: Validates that Arrows.app JSON is correctly parsed into ImportedGraph
        // Expected Behavior: Nodes and edges are created with proper properties and positions

        // Given
        let service = GraphImportService::new();
        let json = r#"{
            "nodes": [{
                "id": "n1",
                "position": {"x": 0, "y": 0},
                "caption": "Node 1",
                "labels": ["Person"],
                "properties": {"name": "Alice"},
                "style": {}
            }, {
                "id": "n2",
                "position": {"x": 100, "y": 100},
                "caption": "Node 2",
                "labels": ["Person"],
                "properties": {"name": "Bob"},
                "style": {}
            }],
            "relationships": [{
                "id": "r1",
                "fromId": "n1",
                "toId": "n2",
                "type": "KNOWS",
                "properties": {"since": "2020"},
                "style": {}
            }]
        }"#;

        // When
        let result = service.import_from_json(json, ImportFormat::ArrowsApp);

        // Then
        assert!(result.is_ok(), "Import should succeed");

        let graph = result.unwrap();
        assert_eq!(graph.nodes.len(), 2, "Should have 2 nodes");
        assert_eq!(graph.edges.len(), 1, "Should have 1 edge");

        // Verify first node
        let node1 = &graph.nodes[0];
        assert_eq!(node1.id, "n1");
        assert_eq!(node1.label, "Node 1");
        assert_eq!(node1.position.x, 0.0);
        assert_eq!(node1.position.y, 0.0);

        // Verify edge
        let edge = &graph.edges[0];
        assert_eq!(edge.source, "n1");
        assert_eq!(edge.target, "n2");
        assert_eq!(edge.edge_type, "knows"); // Should be mapped to lowercase
    }

    #[test]
    fn test_import_mermaid() {
        // User Story: US9 - Import/Export
        // Acceptance Criteria: Can import graphs from Mermaid diagram format
        // Test Purpose: Validates that Mermaid diagrams are correctly parsed into ImportedGraph
        // Expected Behavior: Nodes and edges are created from Mermaid syntax

        // Given
        let service = GraphImportService::new();
        let mermaid = r#"
graph TD
    A[Start] --> B{Decision}
    B --> C[Option 1]
    B --> D[Option 2]
"#;

        // When
        let result = service.import_mermaid(mermaid);

        // Then
        assert!(result.is_ok());
        let graph = result.unwrap();
        assert_eq!(graph.nodes.len(), 4); // A, B, C, D
        assert_eq!(graph.edges.len(), 3); // A->B, B->C, B->D

        // Verify nodes have correct labels
        let node_labels: Vec<_> = graph.nodes.iter()
            .map(|n| &n.label)
            .collect();
        assert!(node_labels.contains(&&"Start".to_string()));
        assert!(node_labels.contains(&&"Decision".to_string()));
        assert!(node_labels.contains(&&"Option 1".to_string()));
        assert!(node_labels.contains(&&"Option 2".to_string()));
    }

    #[test]
    fn test_import_mermaid_from_markdown() {
        // User Story: US9 - Import/Export
        // Acceptance Criteria: Can extract and import Mermaid diagrams from Markdown
        // Test Purpose: Validates that Mermaid diagrams embedded in Markdown are correctly extracted and parsed
        // Expected Behavior: Mermaid code block is extracted from Markdown and parsed

        // Given
        let service = GraphImportService::new();
        let markdown = r#"
# My Document

Here's a diagram:

```mermaid
graph LR
    A[Input] --> B[Process]
    B --> C[Output]
```

Some more text.
"#;

        // When
        let result = service.import_mermaid(markdown);

        // Then
        assert!(result.is_ok());
        let graph = result.unwrap();
        assert_eq!(graph.nodes.len(), 3); // A, B, C
        assert_eq!(graph.edges.len(), 2); // A->B, B->C
    }
}
