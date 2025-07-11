//! Tests for the graph parser functionality

use alchemist::graph_parser::*;

#[test]
fn test_parse_json_graph_standard_format() {
    let json = r#"{
        "nodes": [
            {"id": "1", "label": "Node 1"},
            {"id": "2", "label": "Node 2"}
        ],
        "edges": [
            {"source": "1", "target": "2", "label": "connects"}
        ]
    }"#;
    
    let (nodes, edges) = parse_json_graph(json).unwrap();
    assert_eq!(nodes.len(), 2);
    assert_eq!(edges.len(), 1);
}

#[test]
fn test_parse_json_graph_cytoscape_format() {
    let json = r#"{
        "elements": {
            "nodes": [
                {"data": {"id": "a", "label": "Node A"}},
                {"data": {"id": "b", "label": "Node B"}}
            ],
            "edges": [
                {"data": {"source": "a", "target": "b"}}
            ]
        }
    }"#;
    
    let (nodes, edges) = parse_json_graph(json).unwrap();
    assert_eq!(nodes.len(), 2);
    assert_eq!(edges.len(), 1);
}

#[test]
fn test_parse_markdown_to_graph() {
    let markdown = r#"# Main Topic

## Subtopic 1
Content for subtopic 1

## Subtopic 2
Content for subtopic 2

### Sub-subtopic 2.1
Nested content"#;
    
    let (nodes, edges) = parse_markdown_to_graph(markdown).unwrap();
    assert_eq!(nodes.len(), 4); // Main + 2 subtopics + 1 sub-subtopic
    assert_eq!(edges.len(), 3); // Hierarchical connections
}

#[test]
fn test_parse_nix_graph() {
    let nix_content = r#"{
        pkgs.hello = {
            buildInputs = [ pkgs.gcc pkgs.make ];
            propagatedBuildInputs = [ pkgs.glibc ];
        };
    }"#;
    
    let (nodes, edges) = parse_nix_graph(nix_content).unwrap();
    assert!(nodes.len() >= 1); // At least the hello package
    assert!(!edges.is_empty()); // Should have dependency edges
}

#[test]
fn test_empty_inputs() {
    assert!(parse_json_graph("{}").unwrap().0.is_empty());
    assert!(parse_markdown_to_graph("").unwrap().0.is_empty());
    assert!(parse_json_graph("[]").is_ok());
}

#[test]
fn test_invalid_json() {
    assert!(parse_json_graph("not json").is_err());
    assert!(parse_json_graph("{incomplete").is_err());
}