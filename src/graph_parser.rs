//! Graph file parser for various formats
//!
//! Supports parsing graph data from:
//! - JSON graph format
//! - Nix dependency graphs
//! - Markdown document structure
//! - DOT files
//! - GraphML

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use pulldown_cmark::{Parser, Event, Tag, TagEnd, HeadingLevel};
use std::collections::HashMap;
// Node and Edge data structures for parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub id: String,
    pub label: String,
    pub position: [f32; 3],
    pub color: Option<[f32; 4]>,
    pub size: Option<f32>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeData {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub label: Option<String>,
    pub weight: f32,
    pub color: Option<[f32; 4]>,
    pub metadata: serde_json::Value,
}

/// Parse graph data from JSON
pub fn parse_json_graph(content: &str) -> Result<(Vec<NodeData>, Vec<EdgeData>)> {
    let data: Value = serde_json::from_str(content)
        .context("Failed to parse JSON")?;
    
    // Try different JSON graph formats
    if let Some(nodes) = data.get("nodes") {
        // Standard format with nodes/edges arrays
        parse_standard_json(&data)
    } else if let Some(elements) = data.get("elements") {
        // Cytoscape.js format
        parse_cytoscape_json(&data)
    } else if data.is_array() {
        // Array of nodes with relationships
        parse_array_json(&data)
    } else {
        // Try to extract graph from nested structure
        parse_nested_json(&data)
    }
}

/// Parse standard JSON graph format
fn parse_standard_json(data: &Value) -> Result<(Vec<NodeData>, Vec<EdgeData>)> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    
    // Parse nodes
    if let Some(nodes_array) = data["nodes"].as_array() {
        for node_data in nodes_array {
            let node = NodeData {
                id: if let Some(id_str) = node_data["id"].as_str() {
                    id_str.to_string()
                } else if let Some(id_num) = node_data["id"].as_i64() {
                    id_num.to_string()
                } else {
                    String::new()
                },
                label: node_data["label"]
                    .as_str()
                    .or_else(|| node_data["name"].as_str())
                    .or_else(|| node_data["title"].as_str())
                    .unwrap_or("")
                    .to_string(),
                position: parse_position(&node_data).unwrap_or([0.0, 0.0, 0.0]),
                color: parse_color(&node_data),
                size: node_data["size"]
                    .as_f64()
                    .or_else(|| node_data["radius"].as_f64())
                    .map(|s| s as f32),
                metadata: if !node_data["metadata"].is_null() {
                    node_data["metadata"].clone()
                } else if !node_data["properties"].is_null() {
                    node_data["properties"].clone()
                } else {
                    Value::Null
                },
            };
            nodes.push(node);
        }
    }
    
    // Parse edges
    if let Some(edges_array) = data["edges"].as_array() {
        for edge_data in edges_array {
            let source = edge_data["source"]
                .as_str()
                .or_else(|| edge_data["from"].as_str())
                .map(|s| s.to_string())
                .or_else(|| edge_data["source"].as_i64().map(|i| i.to_string()))
                .unwrap_or_default();
            let target = edge_data["target"]
                .as_str()
                .or_else(|| edge_data["to"].as_str())
                .map(|s| s.to_string())
                .or_else(|| edge_data["target"].as_i64().map(|i| i.to_string()))
                .unwrap_or_default();
            let edge = EdgeData {
                id: edge_data["id"].as_str().map(String::from)
                    .unwrap_or_else(|| format!("edge_{}_{}", source, target)),
                source_id: source,
                target_id: target,
                label: edge_data["label"]
                    .as_str()
                    .or_else(|| edge_data["relationship"].as_str())
                    .or_else(|| edge_data["type"].as_str())
                    .map(|s| s.to_string()),
                weight: edge_data["weight"]
                    .as_f64()
                    .or_else(|| edge_data["strength"].as_f64())
                    .map(|w| w as f32)
                    .unwrap_or(1.0),
                color: parse_edge_color(&edge_data),
                metadata: edge_data["metadata"].clone(),
            };
            edges.push(edge);
        }
    }
    
    // Also check for "links" instead of "edges"
    if edges.is_empty() {
        if let Some(links_array) = data["links"].as_array() {
            for link_data in links_array {
                let source = link_data["source"].as_str().unwrap_or("").to_string();
                let target = link_data["target"].as_str().unwrap_or("").to_string();
                let edge = EdgeData {
                    id: link_data["id"].as_str().map(String::from)
                        .unwrap_or_else(|| format!("edge_{}_{}", source, target)),
                    source_id: source,
                    target_id: target,
                    label: link_data["label"].as_str().map(|s| s.to_string()),
                    weight: link_data["weight"].as_f64().map(|w| w as f32).unwrap_or(1.0),
                    color: parse_edge_color(&link_data),
                    metadata: link_data["metadata"].clone(),
                };
                edges.push(edge);
            }
        }
    }
    
    Ok((nodes, edges))
}

/// Parse Cytoscape.js format
fn parse_cytoscape_json(data: &Value) -> Result<(Vec<NodeData>, Vec<EdgeData>)> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    
    if let Some(elements) = data["elements"].as_object() {
        // Parse nodes
        if let Some(nodes_array) = elements["nodes"].as_array() {
            for node_data in nodes_array {
                let data = &node_data["data"];
                let position = &node_data["position"];
                
                let node = NodeData {
                    id: data["id"].as_str().unwrap_or("").to_string(),
                    label: data["label"]
                        .as_str()
                        .or_else(|| data["name"].as_str())
                        .unwrap_or("")
                        .to_string(),
                    position: if position.is_object() {
                        [
                            position["x"].as_f64().unwrap_or(0.0) as f32,
                            position["y"].as_f64().unwrap_or(0.0) as f32,
                            position["z"].as_f64().unwrap_or(0.0) as f32,
                        ]
                    } else {
                        [0.0, 0.0, 0.0]
                    },
                    color: parse_color(&data),
                    size: data["size"].as_f64().map(|s| s as f32),
                    metadata: data.clone(),
                };
                nodes.push(node);
            }
        }
        
        // Parse edges
        if let Some(edges_array) = elements["edges"].as_array() {
            for edge_data in edges_array {
                let data = &edge_data["data"];
                
                let source = data["source"].as_str().unwrap_or("").to_string();
                let target = data["target"].as_str().unwrap_or("").to_string();
                let edge = EdgeData {
                    id: data["id"].as_str().map(String::from)
                        .unwrap_or_else(|| format!("edge_{}_{}", source, target)),
                    source_id: source,
                    target_id: target,
                    label: data["label"].as_str().map(|s| s.to_string()),
                    weight: data["weight"].as_f64().map(|w| w as f32).unwrap_or(1.0),
                    color: parse_edge_color(&data),
                    metadata: data.clone(),
                };
                edges.push(edge);
            }
        }
    }
    
    Ok((nodes, edges))
}

/// Parse array format (nodes with embedded relationships)
fn parse_array_json(data: &Value) -> Result<(Vec<NodeData>, Vec<EdgeData>)> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut node_index = HashMap::new();
    
    if let Some(array) = data.as_array() {
        // First pass: create nodes
        for (idx, item) in array.iter().enumerate() {
            let id = item["id"]
                .as_str()
                .or_else(|| item["name"].as_str())
                .unwrap_or(&idx.to_string())
                .to_string();
            
            node_index.insert(id.clone(), idx);
            
            let node = NodeData {
                id: id.clone(),
                label: item["label"]
                    .as_str()
                    .or_else(|| item["name"].as_str())
                    .or_else(|| item["title"].as_str())
                    .unwrap_or(&id)
                    .to_string(),
                position: parse_position(&item).unwrap_or([0.0, 0.0, 0.0]),
                color: parse_color(&item),
                size: item["size"].as_f64().map(|s| s as f32),
                metadata: item.clone(),
            };
            nodes.push(node);
        }
        
        // Second pass: create edges from relationships
        for item in array {
            let source_id = item["id"]
                .as_str()
                .or_else(|| item["name"].as_str())
                .unwrap_or("")
                .to_string();
            
            // Check for various relationship fields
            let relationship_fields = ["connections", "links", "edges", "children", "parents", "references"];
            
            for field in &relationship_fields {
                if let Some(connections) = item[field].as_array() {
                    for target in connections {
                        let target_id = target.as_str()
                            .or_else(|| target["id"].as_str())
                            .unwrap_or("")
                            .to_string();
                        
                        if !target_id.is_empty() && node_index.contains_key(&target_id) {
                            edges.push(EdgeData {
                                id: format!("edge_{}_{}", source_id, target_id),
                                source_id: source_id.clone(),
                                target_id: target_id,
                                label: Some(field.to_string()),
                                weight: 1.0,
                                color: None,
                                metadata: serde_json::Value::Null,
                            });
                        }
                    }
                }
            }
        }
    }
    
    Ok((nodes, edges))
}

/// Parse nested JSON structure
fn parse_nested_json(data: &Value) -> Result<(Vec<NodeData>, Vec<EdgeData>)> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut node_id = 0;
    
    fn traverse_json(
        value: &Value,
        parent_id: Option<String>,
        nodes: &mut Vec<NodeData>,
        edges: &mut Vec<EdgeData>,
        node_id: &mut i32,
        path: &str,
    ) {
        match value {
            Value::Object(map) => {
                // Create node for object
                let id = node_id.to_string();
                *node_id += 1;
                
                let label = map.get("name")
                    .or_else(|| map.get("title"))
                    .or_else(|| map.get("label"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(path)
                    .to_string();
                
                nodes.push(NodeData {
                    id: id.clone(),
                    label,
                    position: [0.0, 0.0, 0.0],
                    color: None,
                    size: None,
                    metadata: value.clone(),
                });
                
                // Create edge from parent
                if let Some(parent) = parent_id {
                    edges.push(EdgeData {
                        id: format!("edge_{}_{}", parent, id),
                        source_id: parent,
                        target_id: id.clone(),
                        label: Some(path.to_string()),
                        weight: 1.0,
                        color: None,
                        metadata: serde_json::Value::Null,
                    });
                }
                
                // Traverse children
                for (key, child) in map {
                    if !["name", "title", "label", "id"].contains(&key.as_str()) {
                        traverse_json(child, Some(id.clone()), nodes, edges, node_id, key);
                    }
                }
            }
            Value::Array(arr) => {
                // Create nodes for array items
                for (idx, item) in arr.iter().enumerate() {
                    let item_path = format!("{}[{}]", path, idx);
                    traverse_json(item, parent_id.clone(), nodes, edges, node_id, &item_path);
                }
            }
            _ => {
                // Leaf values - could create nodes if needed
            }
        }
    }
    
    traverse_json(data, None, &mut nodes, &mut edges, &mut node_id, "root");
    
    if nodes.is_empty() {
        anyhow::bail!("No graph structure found in JSON");
    }
    
    Ok((nodes, edges))
}

/// Parse Nix file to extract dependency graph
pub fn parse_nix_graph(content: &str) -> Result<(Vec<NodeData>, Vec<EdgeData>)> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut node_map = HashMap::new();
    
    // Simple Nix parser - looks for package definitions and dependencies
    let lines: Vec<&str> = content.lines().collect();
    let mut current_package = None;
    let mut in_deps = false;
    
    for line in lines {
        let trimmed = line.trim();
        
        // Look for package definitions
        if trimmed.ends_with(" = {") || trimmed.ends_with(" = mkDerivation {") {
            if let Some(name) = trimmed.split_whitespace().next() {
                let id = name.to_string();
                node_map.insert(id.clone(), nodes.len());
                nodes.push(NodeData {
                    id: id.clone(),
                    label: name.to_string(),
                    position: [0.0, 0.0, 0.0],
                    color: Some([0.2, 0.5, 0.8, 1.0]), // Blue for packages
                    size: None,
                    metadata: serde_json::json!({ "type": "package" }),
                });
                current_package = Some(id);
                in_deps = false;
            }
        }
        
        // Look for dependencies
        if trimmed.starts_with("buildInputs") || 
           trimmed.starts_with("propagatedBuildInputs") ||
           trimmed.starts_with("nativeBuildInputs") {
            in_deps = true;
        }
        
        // Parse dependencies
        if in_deps && current_package.is_some() {
            // Extract package names from the line
            let deps_str = trimmed.trim_start_matches(|c: char| !c.is_alphabetic());
            for dep in deps_str.split_whitespace() {
                let dep_name = dep.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
                if !dep_name.is_empty() && dep_name != "=" && dep_name != "[" && dep_name != "]" {
                    // Create node for dependency if it doesn't exist
                    if !node_map.contains_key(dep_name) {
                        node_map.insert(dep_name.to_string(), nodes.len());
                        nodes.push(NodeData {
                            id: dep_name.to_string(),
                            label: dep_name.to_string(),
                            position: [0.0, 0.0, 0.0],
                            color: Some([0.8, 0.3, 0.3, 1.0]), // Red for external deps
                            size: None,
                            metadata: serde_json::json!({ "type": "dependency" }),
                        });
                    }
                    
                    // Create edge
                    if let Some(source) = &current_package {
                        edges.push(EdgeData {
                            id: format!("edge_{}_{}", source, dep_name),
                            source_id: source.clone(),
                            target_id: dep_name.to_string(),
                            label: Some("depends on".to_string()),
                            weight: 1.0,
                            color: None,
                            metadata: serde_json::Value::Null,
                        });
                    }
                }
            }
        }
        
        // End of dependencies
        if trimmed == "];" || trimmed == "]" {
            in_deps = false;
        }
        
        // End of package
        if trimmed == "};" || trimmed == "}" {
            current_package = None;
            in_deps = false;
        }
    }
    
    if nodes.is_empty() {
        anyhow::bail!("No Nix packages found in file");
    }
    
    Ok((nodes, edges))
}

/// Parse Markdown to extract document structure as graph
pub fn parse_markdown_graph(content: &str) -> Result<(Vec<NodeData>, Vec<EdgeData>)> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut heading_stack: Vec<(HeadingLevel, String)> = Vec::new();
    let mut current_text = String::new();
    let mut node_id = 0;
    
    let parser = Parser::new(content);
    let mut in_heading = false;
    let mut current_heading_level = HeadingLevel::H1;
    
    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                current_heading_level = level;
                current_text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;
                
                // Create node for heading
                let id = format!("node_{}", node_id);
                node_id += 1;
                
                let color = match current_heading_level {
                    HeadingLevel::H1 => [0.8, 0.2, 0.2, 1.0], // Red
                    HeadingLevel::H2 => [0.2, 0.8, 0.2, 1.0], // Green
                    HeadingLevel::H3 => [0.2, 0.2, 0.8, 1.0], // Blue
                    HeadingLevel::H4 => [0.8, 0.8, 0.2, 1.0], // Yellow
                    HeadingLevel::H5 => [0.8, 0.2, 0.8, 1.0], // Magenta
                    HeadingLevel::H6 => [0.2, 0.8, 0.8, 1.0], // Cyan
                };
                
                nodes.push(NodeData {
                    id: id.clone(),
                    label: current_text.clone(),
                    position: [0.0, 0.0, 0.0],
                    color: Some(color),
                    size: Some(match current_heading_level {
                        HeadingLevel::H1 => 2.0,
                        HeadingLevel::H2 => 1.5,
                        HeadingLevel::H3 => 1.2,
                        _ => 1.0,
                    }),
                    metadata: serde_json::json!({
                        "type": "heading",
                        "level": format!("{:?}", current_heading_level),
                    }),
                });
                
                // Update heading stack and create edges
                let level_num = match current_heading_level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                
                // Pop headings of same or lower level
                while let Some((stack_level, _)) = heading_stack.last() {
                    let stack_level_num = match stack_level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };
                    if stack_level_num >= level_num {
                        heading_stack.pop();
                    } else {
                        break;
                    }
                }
                
                // Create edge to parent heading
                if let Some((_, parent_id)) = heading_stack.last() {
                    edges.push(EdgeData {
                        id: format!("edge_{}_{}", parent_id, id),
                        source_id: parent_id.clone(),
                        target_id: id.clone(),
                        label: Some("contains".to_string()),
                        weight: 1.0,
                        color: None,
                        metadata: serde_json::Value::Null,
                    });
                }
                
                heading_stack.push((current_heading_level, id));
                current_text.clear();
            }
            Event::Text(text) => {
                if in_heading {
                    current_text.push_str(&text);
                }
            }
            Event::Start(Tag::Link { dest_url, title, .. }) => {
                // Create nodes for links
                let link_id = format!("link_{}", node_id);
                node_id += 1;
                
                nodes.push(NodeData {
                    id: link_id.clone(),
                    label: if !title.is_empty() {
                        title.to_string()
                    } else {
                        dest_url.to_string()
                    },
                    position: [0.0, 0.0, 0.0],
                    color: Some([0.5, 0.5, 0.9, 1.0]), // Light blue for links
                    size: Some(0.8),
                    metadata: serde_json::json!({
                        "type": "link",
                        "url": dest_url.to_string(),
                    }),
                });
                
                // Link to current heading
                if let Some((_, heading_id)) = heading_stack.last() {
                    edges.push(EdgeData {
                        id: format!("edge_{}_{}", heading_id, link_id),
                        source_id: heading_id.clone(),
                        target_id: link_id,
                        label: Some("links to".to_string()),
                        weight: 1.0,
                        color: Some([0.5, 0.5, 0.9, 0.5]),
                        metadata: serde_json::Value::Null,
                    });
                }
            }
            _ => {}
        }
    }
    
    if nodes.is_empty() {
        // Create at least a root node
        nodes.push(NodeData {
            id: "root".to_string(),
            label: "Document".to_string(),
            position: [0.0, 0.0, 0.0],
            color: Some([0.5, 0.5, 0.5, 1.0]),
            size: Some(1.5),
            metadata: serde_json::json!({ "type": "document" }),
        });
    }
    
    Ok((nodes, edges))
}

// Helper functions

fn parse_position(data: &Value) -> Option<[f32; 3]> {
    if let Some(pos) = data.get("position") {
        if let Some(arr) = pos.as_array() {
            if arr.len() >= 2 {
                return Some([
                    arr[0].as_f64().unwrap_or(0.0) as f32,
                    arr[1].as_f64().unwrap_or(0.0) as f32,
                    arr.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                ]);
            }
        } else if pos.is_object() {
            return Some([
                pos["x"].as_f64().unwrap_or(0.0) as f32,
                pos["y"].as_f64().unwrap_or(0.0) as f32,
                pos["z"].as_f64().unwrap_or(0.0) as f32,
            ]);
        }
    } else if let (Some(x), Some(y)) = (data.get("x"), data.get("y")) {
        return Some([
            x.as_f64().unwrap_or(0.0) as f32,
            y.as_f64().unwrap_or(0.0) as f32,
            data.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
        ]);
    }
    None
}

fn parse_color(data: &Value) -> Option<[f32; 4]> {
    if let Some(color) = data.get("color") {
        if let Some(arr) = color.as_array() {
            if arr.len() >= 3 {
                return Some([
                    arr[0].as_f64().unwrap_or(0.0) as f32,
                    arr[1].as_f64().unwrap_or(0.0) as f32,
                    arr[2].as_f64().unwrap_or(0.0) as f32,
                    arr.get(3).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                ]);
            }
        } else if let Some(hex) = color.as_str() {
            // Parse hex color
            if let Ok(rgb) = hex_to_rgb(hex) {
                return Some(rgb);
            }
        } else if color.is_object() {
            return Some([
                color["r"].as_f64().unwrap_or(0.0) as f32,
                color["g"].as_f64().unwrap_or(0.0) as f32,
                color["b"].as_f64().unwrap_or(0.0) as f32,
                color["a"].as_f64().unwrap_or(1.0) as f32,
            ]);
        }
    }
    None
}

fn parse_edge_color(data: &Value) -> Option<[f32; 4]> {
    parse_color(data)
}

fn hex_to_rgb(hex: &str) -> Result<[f32; 4]> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16)? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16)? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16)? as f32 / 255.0;
        Ok([r, g, b, 1.0])
    } else {
        anyhow::bail!("Invalid hex color: {}", hex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_standard_json() {
        let json = r#"{
            "nodes": [
                {"id": "1", "label": "Node 1", "position": [0, 0, 0]},
                {"id": "2", "label": "Node 2", "position": [1, 1, 0]}
            ],
            "edges": [
                {"source": "1", "target": "2", "label": "connects"}
            ]
        }"#;
        
        let (nodes, edges) = parse_json_graph(json).unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(edges.len(), 1);
        assert_eq!(nodes[0].label, "Node 1");
        assert_eq!(edges[0].source, "1");
    }
    
    #[test]
    fn test_parse_markdown() {
        let markdown = r#"# Title
        
## Section 1
Some content

## Section 2
More content

### Subsection 2.1
Details"#;
        
        let (nodes, edges) = parse_markdown_graph(markdown).unwrap();
        assert!(nodes.len() >= 4); // At least 4 headings
        assert!(!edges.is_empty()); // Should have parent-child relationships
    }
}