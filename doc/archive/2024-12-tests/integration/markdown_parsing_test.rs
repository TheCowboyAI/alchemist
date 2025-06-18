//! Test markdown parsing functionality

use ia::domain::services::{GraphImportService, ImportFormat};
use std::collections::HashMap;

#[test]
fn test_parse_markdown_with_mermaid() {
    let markdown_content = r#"
# Test Document

This is a test document with a Mermaid diagram.

```mermaid
graph TD
    A[Start] --> B[Process]
    B --> C{Decision}
    C -->|Yes| D[Action 1]
    C -->|No| E[Action 2]
    D --> F[End]
    E --> F
```

More text here.
"#;

    let import_service = GraphImportService::new();

    // Import the markdown content
    let result = import_service.import_from_content(markdown_content, ImportFormat::Mermaid, None);

    assert!(
        result.is_ok(),
        "Should successfully parse markdown with Mermaid"
    );

    let graph = result.unwrap();

    // Verify nodes were created
    assert_eq!(
        graph.nodes.len(),
        6,
        "Should have 6 nodes (A, B, C, D, E, F)"
    );

    // Verify node labels
    let node_labels: Vec<&str> = graph.nodes.iter().map(|n| n.label.as_str()).collect();

    assert!(node_labels.contains(&"Start"));
    assert!(node_labels.contains(&"Process"));
    assert!(node_labels.contains(&"Decision"));
    assert!(node_labels.contains(&"Action 1"));
    assert!(node_labels.contains(&"Action 2"));
    assert!(node_labels.contains(&"End"));

    // Verify edges were created
    assert!(graph.edges.len() >= 5, "Should have at least 5 edges");
}

#[test]
fn test_parse_complex_mermaid_with_subgraphs() {
    let mermaid_content = r#"
graph TB
    subgraph Core["Core Domain"]
        A[Entity A]
        B[Entity B]
        C[Service C]
    end

    subgraph Support["Supporting Domain"]
        D[Helper D]
        E[Utility E]
    end

    A --> B
    B --> C
    C --> D
    D --> E
    A --> E
"#;

    let import_service = GraphImportService::new();

    let result = import_service.import_from_content(mermaid_content, ImportFormat::Mermaid, None);

    assert!(
        result.is_ok(),
        "Should successfully parse Mermaid with subgraphs"
    );

    let graph = result.unwrap();

    // Verify nodes
    assert_eq!(graph.nodes.len(), 5, "Should have 5 nodes");

    // Verify edges
    assert_eq!(graph.edges.len(), 5, "Should have 5 edges");

    // Verify metadata contains subgraph information
    assert!(
        graph.metadata.contains_key("subgraphs"),
        "Should have subgraph metadata"
    );
}

#[test]
fn test_parse_ddd_markdown_file() {
    // Test parsing an actual DDD markdown file
    let file_path = "assets/keco/KECO_DDD_Core_Model.md";

    if std::path::Path::new(file_path).exists() {
        let content = std::fs::read_to_string(file_path).expect("Should read markdown file");

        let import_service = GraphImportService::new();

        let result = import_service.import_from_content(&content, ImportFormat::Mermaid, None);

        assert!(
            result.is_ok(),
            "Should successfully parse DDD markdown file"
        );

        let graph = result.unwrap();

        // The file contains multiple Mermaid diagrams
        assert!(
            !graph.nodes.is_empty(),
            "Should have nodes from the diagrams"
        );
        assert!(
            !graph.edges.is_empty(),
            "Should have edges from the diagrams"
        );

        println!(
            "Parsed {} nodes and {} edges from {}",
            graph.nodes.len(),
            graph.edges.len(),
            file_path
        );
    } else {
        println!("Skipping file test - {} not found", file_path);
    }
}

#[test]
fn test_mermaid_node_types() {
    let mermaid_content = r#"
graph LR
    A[Regular Node]
    B{Decision Node}
    C((Circle Node))
    D>Asymmetric Node]
    E[/Parallelogram/]
    F[\Inverted Trapezoid\]

    A --> B
    B --> C
    C --> D
    D --> E
    E --> F
"#;

    let import_service = GraphImportService::new();

    let result = import_service.import_from_content(mermaid_content, ImportFormat::Mermaid, None);

    assert!(result.is_ok(), "Should parse various node types");

    let graph = result.unwrap();

    assert_eq!(
        graph.nodes.len(),
        6,
        "Should have 6 nodes with different shapes"
    );
    assert_eq!(graph.edges.len(), 5, "Should have 5 edges");
}
